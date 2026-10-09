"""Closed hosted synthetic-PDF diagnostic; never application or worker authority."""
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import threading
import time

CASE = "attachment_processing::tests::synthetic_pdf_retains_all_pages_in_order_and_rejects_over_limit"
CLASSIFICATION = "hosted-synthetic-pdf-library-diagnostic-not-worker-qualification"
PREFIX = b"ASTER_PDF_DIAG_V1|"
HARNESS = re.compile(r"aster_desktop_lib-[0-9a-f]{16}\.exe")
CAP = 16384
CHUNK = 1024
LINE_CAP = 128
MARKER_CAP = 128
ENTRY_CAP = 4096
CONTEXT_CAP = 4096
DECISION_SECONDS = 30.0
STOP_SECONDS = 25.0
CLEANUP_SECONDS = 5.0
RECEIPT_KEYS = frozenset((
    "schema_version", "classification", "source_revision", "workflow_run_id",
    "workflow_run_attempt", "test_case", "status", "reason", "exit_class",
    "checkpoints", "last_checkpoint", "trace_complete", "single_test_reported",
    "cleanup_complete", "poisoned", "captured_bytes", "elapsed_ms",
))
FAILURES = frozenset((
    "artifact_identity_invalid", "launch_failure", "cleanup_failure",
    "output_limit", "timeout", "marker_invalid", "access_violation",
    "nonzero_exit", "test_not_reported",
))
# Keep uncertain owned references until interpreter/ephemeral-runner teardown.
# This is not proof that an unresponsive native process or pipe was released.
_OWNED = []


def expected_trace():
    result = ["ASTER_PDF_DIAG_V1|valid_begin"]
    for phase, pages in (("valid", 2), ("limit", 0)):
        if phase == "limit":
            result.extend(("ASTER_PDF_DIAG_V1|valid_end",
                           "ASTER_PDF_DIAG_V1|assertions_done",
                           "ASTER_PDF_DIAG_V1|limit_begin"))
        for code in range(64, 73):
            result.extend(f"ASTER_PDF_DIAG_V1|{phase}|{code}|{edge}"
                          for edge in ("enter", "leave"))
        for _ in range(pages):
            for code in range(73, 82):
                result.extend(f"ASTER_PDF_DIAG_V1|{phase}|{code}|{edge}"
                              for edge in ("enter", "leave"))
        if phase == "valid":
            result.append("ASTER_PDF_DIAG_V1|valid|body_complete")
        result.extend(f"ASTER_PDF_DIAG_V1|{phase}|{point}"
                      for point in ("uninit_enter", "uninit_leave"))
    result.extend(("ASTER_PDF_DIAG_V1|limit_end", "ASTER_PDF_DIAG_V1|complete"))
    return tuple(result)


EXPECTED = expected_trace()
SUMMARY = re.compile(
    rb"test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; "
    rb"[0-9]{1,8} filtered out; finished in [0-9]{1,8}\.[0-9]{1,8}s"
)
CASE_OK = ("test " + CASE + " ... ok").encode("ascii")


def parse_capture(stdout, stderr):
    """Retain only admitted static tokens and three closed harness observations."""
    checkpoints = []
    malformed = False
    limited = False
    observations = [0, 0, 0]
    marker_lines = 0
    for stream_index, data in enumerate((stdout, stderr)):
        position = 0
        while position < len(data):
            newline = data.find(b"\n", position)
            terminated = newline >= 0
            end = newline if terminated else len(data)
            content_end = end - 1 if end > position and data[end - 1] == 13 else end
            length = content_end - position
            if data.startswith(PREFIX, position, content_end):
                marker_lines += 1
                if length > LINE_CAP or marker_lines > MARKER_CAP:
                    limited = True
                elif not terminated or stream_index != 1:
                    malformed = True
                else:
                    try:
                        token = data[position:content_end].decode("ascii")
                    except UnicodeDecodeError:
                        malformed = True
                    else:
                        index = len(checkpoints)
                        if index < len(EXPECTED) and token == EXPECTED[index]:
                            checkpoints.append(EXPECTED[index])
                        else:
                            malformed = True
            elif stream_index == 0 and terminated and length <= 256:
                line = data[position:content_end]
                if line == b"running 1 test":
                    observations[0] += 1
                if line == CASE_OK:
                    observations[1] += 1
                if SUMMARY.fullmatch(line):
                    observations[2] += 1
            position = end + 1 if terminated else len(data)
    return {
        "checkpoints": checkpoints,
        "trace_complete": tuple(checkpoints) == EXPECTED,
        "single_test_reported": observations == [1, 1, 1],
        "marker_invalid": malformed,
        "output_limit": limited,
    }


def classify_exit(code):
    if code is None:
        return "unobserved"
    if code == 0:
        return "zero"
    if code & 0xFFFFFFFF == 0xC0000005:
        return "access_violation"
    return "other_nonzero"


def decide(exit_class, parsed, cleanup_complete, output_limit, timed_out):
    if not cleanup_complete:
        return "cleanup_failure"
    if output_limit or parsed["output_limit"]:
        return "output_limit"
    if timed_out:
        return "timeout"
    if parsed["marker_invalid"]:
        return "marker_invalid"
    if exit_class == "access_violation":
        return "access_violation"
    if exit_class in ("other_nonzero", "unobserved"):
        return "nonzero_exit"
    if not parsed["single_test_reported"]:
        return "test_not_reported"
    if not parsed["trace_complete"]:
        return "marker_invalid"
    return "complete"


def binding_valid(context, expected):
    return (
        type(context) is dict
        and context.get("classification") == "hosted-source-fixtures-not-owner-session"
        and type(context.get("source_revision")) is str
        and re.fullmatch(r"[0-9a-f]{40}", context["source_revision"]) is not None
        and type(context.get("workflow_run_id")) is str
        and re.fullmatch(r"[1-9][0-9]{0,19}", context["workflow_run_id"]) is not None
        and type(context.get("workflow_run_attempt")) is str
        and re.fullmatch(r"[1-9][0-9]{0,3}", context["workflow_run_attempt"]) is not None
        and all(context[key] == expected.get(key) for key in
                ("source_revision", "workflow_run_id", "workflow_run_attempt"))
    )


def receipt_valid(receipt):
    if type(receipt) is not dict or set(receipt) != RECEIPT_KEYS:
        return False
    if type(receipt["schema_version"]) is not int or receipt["schema_version"] != 1:
        return False
    if receipt["classification"] != CLASSIFICATION or receipt["test_case"] != CASE:
        return False
    context = dict(receipt, classification="hosted-source-fixtures-not-owner-session")
    if not binding_valid(context, receipt):
        return False
    if receipt["status"] not in ("PASS", "FAIL") or receipt["reason"] not in FAILURES | {"complete"}:
        return False
    if receipt["exit_class"] not in ("zero", "access_violation", "other_nonzero", "unobserved"):
        return False
    for key in ("trace_complete", "single_test_reported", "cleanup_complete", "poisoned"):
        if type(receipt[key]) is not bool:
            return False
    if receipt["poisoned"] != (not receipt["cleanup_complete"]):
        return False
    if type(receipt["captured_bytes"]) is not int or not 0 <= receipt["captured_bytes"] <= CAP:
        return False
    if type(receipt["elapsed_ms"]) is not int or not 0 <= receipt["elapsed_ms"] <= 9007199254740991:
        return False
    tokens = receipt["checkpoints"]
    if type(tokens) is not list or len(tokens) > MARKER_CAP or tuple(tokens) != EXPECTED[:len(tokens)]:
        return False
    if receipt["last_checkpoint"] != (tokens[-1] if tokens else None):
        return False
    if receipt["trace_complete"] != (tuple(tokens) == EXPECTED):
        return False
    if (receipt["status"] == "PASS") != (receipt["reason"] == "complete"):
        return False
    if receipt["status"] == "PASS" and not (
        receipt["exit_class"] == "zero" and receipt["trace_complete"]
        and receipt["single_test_reported"] and receipt["cleanup_complete"]
        and receipt["elapsed_ms"] <= 30000
    ):
        return False
    return True


def remaining(deadline):
    return max(0.0, deadline - time.monotonic())


def assert_plain(path):
    """Reject reparse/symlink ancestors before canonical path admission."""
    for item in (path, *path.parents):
        information = os.lstat(item)
        if stat.S_ISLNK(information.st_mode) or (
            getattr(information, "st_file_attributes", 0)
            & getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0x400)
        ):
            raise ValueError("artifact_identity_invalid")


def select_harness(root, stop_deadline):
    directory = root / "src-tauri" / "target" / "debug" / "deps"
    assert_plain(directory)
    canonical = directory.resolve(strict=True)
    candidate = None
    count = 0
    with os.scandir(directory) as entries:
        for entry in entries:
            count += 1
            if count > ENTRY_CAP or remaining(stop_deadline) <= 0:
                raise ValueError("artifact_identity_invalid")
            if HARNESS.fullmatch(entry.name):
                if candidate is not None or not entry.is_file(follow_symlinks=False):
                    raise ValueError("artifact_identity_invalid")
                chosen = Path(entry.path)
                assert_plain(chosen)
                if chosen.resolve(strict=True).parent != canonical:
                    raise ValueError("artifact_identity_invalid")
                candidate = chosen
    if candidate is None or remaining(stop_deadline) <= 0:
        raise ValueError("artifact_identity_invalid")
    return candidate


def collect(process, stop_deadline, final_deadline):
    lock = threading.Lock()
    wake = threading.Event()
    reader_error = threading.Event()
    output_limit = threading.Event()
    buffers = [bytearray(), bytearray()]
    eof = [False, False]
    total = [0]

    def read_pipe(index, pipe):
        try:
            while True:
                chunk = pipe.read(CHUNK)
                if not lock.acquire(timeout=remaining(final_deadline)):
                    reader_error.set()
                    wake.set()
                    return
                try:
                    if not chunk:
                        eof[index] = True
                        return
                    if len(chunk) > CHUNK or total[0] + len(chunk) > CAP:
                        output_limit.set()
                    elif not output_limit.is_set():
                        buffers[index].extend(chunk)
                        total[0] += len(chunk)
                finally:
                    lock.release()
                    wake.set()
                if remaining(final_deadline) <= 0:
                    reader_error.set()
                    return
        except Exception:
            reader_error.set()
            wake.set()

    threads = [
        threading.Thread(target=read_pipe, args=(0, process.stdout), daemon=True),
        threading.Thread(target=read_pipe, args=(1, process.stderr), daemon=True),
    ]
    _OWNED.append((process, threads, buffers))
    for thread in threads:
        try:
            thread.start()
        except Exception:
            reader_error.set()
    timed_out = False
    while True:
        budget = remaining(stop_deadline)
        if budget <= 0:
            timed_out = True
            break
        if output_limit.is_set() or reader_error.is_set():
            break
        if process.poll() is not None and all(not thread.is_alive() for thread in threads):
            break
        wake.wait(min(0.01, budget))
        wake.clear()

    cleanup_deadline = min(final_deadline, time.monotonic() + CLEANUP_SECONDS)
    if process.poll() is None:
        try:
            process.kill()
        except Exception:
            reader_error.set()
    while remaining(cleanup_deadline) > 0:
        if process.poll() is not None and all(not thread.is_alive() for thread in threads):
            break
        wake.wait(min(0.01, remaining(cleanup_deadline)))
        wake.clear()
    for thread in threads:
        if thread.ident is not None:
            thread.join(timeout=remaining(cleanup_deadline))
    code = process.poll()
    cleanup = (
        code is not None and all(not thread.is_alive() for thread in threads)
        and all(eof) and not reader_error.is_set()
    )
    if not lock.acquire(timeout=remaining(cleanup_deadline)):
        cleanup = False
        parsed = parse_capture(bytearray(), bytearray())
        captured = total[0]
    else:
        try:
            # Parsing only completed readers avoids racing mutable buffers.
            if all(not thread.is_alive() for thread in threads):
                parsed = parse_capture(buffers[0], buffers[1])
            else:
                parsed = parse_capture(bytearray(), bytearray())
            captured = total[0]
        finally:
            lock.release()
    if cleanup and remaining(cleanup_deadline) > 0:
        try:
            process.stdout.close()
            process.stderr.close()
        except Exception:
            cleanup = False
    if remaining(final_deadline) <= 0:
        timed_out = True
    return classify_exit(code), parsed, cleanup, output_limit.is_set(), timed_out, captured


def run():
    started = time.monotonic()
    stop_deadline = started + STOP_SECONDS
    final_deadline = started + DECISION_SECONDS
    root = Path(__file__).resolve(strict=True).parents[2]
    if sys.platform != "win32":
        print("artifact_identity_invalid")
        return 1
    try:
        context_path = root / "reports" / "windows-execution-context.json"
        assert_plain(context_path)
        if not stat.S_ISREG(os.lstat(context_path).st_mode):
            raise ValueError("artifact_identity_invalid")
        with context_path.open("rb") as stream:
            raw_context = stream.read(CONTEXT_CAP + 1)
        if len(raw_context) > CONTEXT_CAP:
            raise ValueError("artifact_identity_invalid")
        context = json.loads(raw_context)
        expected = {
            "source_revision": os.environ.get("GITHUB_SHA"),
            "workflow_run_id": os.environ.get("GITHUB_RUN_ID"),
            "workflow_run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT"),
        }
        if not binding_valid(context, expected):
            raise ValueError("artifact_identity_invalid")
    except Exception:
        print("artifact_identity_invalid")
        return 1
    reason = None
    parsed = parse_capture(bytearray(), bytearray())
    exit_class = "unobserved"
    cleanup = True
    captured = 0
    try:
        executable = select_harness(root, stop_deadline)
    except Exception:
        reason = "artifact_identity_invalid"
    if reason is None:
        try:
            process = subprocess.Popen(
                [str(executable), CASE, "--exact", "--nocapture", "--test-threads=1"],
                shell=False, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                stderr=subprocess.PIPE, bufsize=0, close_fds=True, cwd=str(root),
            )
        except Exception:
            reason = "launch_failure"
        else:
            try:
                exit_class, parsed, cleanup, limited, timed_out, captured = collect(
                    process, stop_deadline, final_deadline
                )
                reason = decide(exit_class, parsed, cleanup, limited, timed_out)
            except Exception:
                # Unexpected reader/thread/OS API failure cannot masquerade
                # as released ownership or replay native exception content.
                _OWNED.append(process)
                try:
                    process.kill()
                except Exception:
                    pass
                cleanup = False
                reason = "cleanup_failure"
    elapsed = max(0, int((time.monotonic() - started) * 1000))
    if reason == "complete" and elapsed > 30000:
        reason = "timeout"
    receipt = {
        "schema_version": 1, "classification": CLASSIFICATION,
        "source_revision": context["source_revision"],
        "workflow_run_id": context["workflow_run_id"],
        "workflow_run_attempt": context["workflow_run_attempt"],
        "test_case": CASE, "status": "PASS" if reason == "complete" else "FAIL",
        "reason": reason, "exit_class": exit_class,
        "checkpoints": parsed["checkpoints"],
        "last_checkpoint": parsed["checkpoints"][-1] if parsed["checkpoints"] else None,
        "trace_complete": parsed["trace_complete"],
        "single_test_reported": parsed["single_test_reported"],
        "cleanup_complete": cleanup, "poisoned": not cleanup,
        "captured_bytes": captured, "elapsed_ms": elapsed,
    }
    if not receipt_valid(receipt):
        print("artifact_identity_invalid")
        return 1
    try:
        report = root / "reports" / "pdf-library-crash-diagnostic.json"
        if report.exists():
            raise ValueError("artifact_identity_invalid")
        with report.open("x", encoding="utf-8", newline="\n") as stream:
            json.dump(receipt, stream, ensure_ascii=True, indent=2)
            stream.write("\n")
        print(json.dumps(receipt, ensure_ascii=True, indent=2))
    except Exception:
        print("artifact_identity_invalid")
        return 1
    return 0 if receipt["status"] == "PASS" else 1


def self_test():
    import io
    from types import SimpleNamespace
    from unittest.mock import patch

    count = 0

    def check(value):
        nonlocal count
        if not value:
            raise AssertionError("pure_fixture_failed")
        count += 1

    stdout = b"running 1 test\n" + CASE_OK + b"\n" + (
        b"test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; "
        b"230 filtered out; finished in 0.02s\n"
    )
    trace = "\n".join(EXPECTED).encode("ascii") + b"\n"
    good = parse_capture(stdout, trace)
    check(len(EXPECTED) == 83)
    check(good["trace_complete"] and good["single_test_reported"])
    check(decide("zero", good, True, False, False) == "complete")
    check(decide("access_violation", good, True, False, False) == "access_violation")
    check(decide("other_nonzero", good, True, False, False) == "nonzero_exit")
    partial = parse_capture(b"", b"\n".join(token.encode("ascii") for token in EXPECTED[:9]) + b"\n")
    check(not partial["marker_invalid"] and not partial["trace_complete"])
    check(decide("access_violation", partial, True, False, False) == "access_violation")
    check(decide("zero", partial, True, False, False) == "test_not_reported")
    check(decide("zero", good, False, True, True) == "cleanup_failure")
    check(decide("access_violation", good, True, True, True) == "output_limit")
    check(decide("access_violation", good, True, False, True) == "timeout")
    unknown = parse_capture(stdout, b"ASTER_PDF_DIAG_V1|unknown\n")
    check(decide("access_violation", unknown, True, False, False) == "marker_invalid")
    check(parse_capture(stdout, trace + EXPECTED[-1].encode("ascii") + b"\n")["marker_invalid"])
    check(parse_capture(stdout, EXPECTED[1].encode("ascii") + b"\n")["marker_invalid"])
    check(parse_capture(stdout, EXPECTED[0].encode("ascii"))["marker_invalid"])
    check(parse_capture(stdout, PREFIX + b"x" * 120 + b"\n")["output_limit"])
    check(parse_capture(stdout, (EXPECTED[0].encode("ascii") + b"\n") * 129)["output_limit"])
    check(parse_capture(stdout + EXPECTED[0].encode("ascii") + b"\n", trace)["marker_invalid"])
    check(not parse_capture(b"running 0 tests\n", trace)["single_test_reported"])
    check(not parse_capture(stdout + stdout, trace)["single_test_reported"])
    check(not parse_capture(stdout.replace(CASE_OK, b"test another_case ... ok"), trace)["single_test_reported"])
    check(not parse_capture(stdout, trace[:-1])["trace_complete"])
    with_stdout = parse_capture(stdout, b"\n".join(token.encode("ascii") for token in EXPECTED[:9]) + b"\n")
    check(decide("zero", with_stdout, True, False, False) == "marker_invalid")
    for name in ("aster_desktop_lib-0123456789abcdef.exe",):
        check(HARNESS.fullmatch(name) is not None)
    for name in ("other.exe", "aster_desktop_lib-0123456789ABCDEF.exe", "../aster_desktop_lib-0123456789abcdef.exe",
                 "aster_desktop_lib-0123456789abcdef.exe.extra", "aster_desktop_lib-1.exe"):
        check(HARNESS.fullmatch(name) is None)
    expected = {"source_revision": "a" * 40, "workflow_run_id": "12", "workflow_run_attempt": "1"}
    context = dict(expected, classification="hosted-source-fixtures-not-owner-session")
    check(binding_valid(context, expected))
    for key, value in (("source_revision", "A" * 40), ("workflow_run_id", "0"),
                       ("workflow_run_attempt", "0001"), ("classification", "owner-session")):
        check(not binding_valid(dict(context, **{key: value}), expected))
    synthetic = {
        "schema_version": 1, "classification": CLASSIFICATION, **expected,
        "test_case": CASE, "status": "PASS", "reason": "complete", "exit_class": "zero",
        "checkpoints": list(EXPECTED), "last_checkpoint": EXPECTED[-1],
        "trace_complete": True, "single_test_reported": True, "cleanup_complete": True,
        "poisoned": False, "captured_bytes": len(stdout) + len(trace), "elapsed_ms": 100,
    }
    check(receipt_valid(synthetic))
    for key, value in (("schema_version", True), ("captured_bytes", CAP + 1),
                       ("elapsed_ms", 30001), ("poisoned", True), ("last_checkpoint", None),
                       ("exit_class", "access_violation"), ("trace_complete", False)):
        check(not receipt_valid(dict(synthetic, **{key: value})))
    check(not receipt_valid(dict(synthetic, extra=True)))
    check(receipt_valid(dict(synthetic, status="FAIL", reason="cleanup_failure",
                             cleanup_complete=False, poisoned=True, elapsed_ms=31000)))
    check(STOP_SECONDS + CLEANUP_SECONDS == DECISION_SECONDS)
    check(min(30.0, 9.0 + CLEANUP_SECONDS) == 14.0)
    class FakeProcess:
        def __init__(self, out, err, code):
            self.stdout = io.BytesIO(out)
            self.stderr = io.BytesIO(err)
            self.code = code
            self.kill_count = 0

        def poll(self):
            return self.code

        def kill(self):
            self.kill_count += 1
            self.code = 1

    # Memory-only owned fake processes: no Popen, OS child, file or network.
    for out, err, code, wanted in (
        (stdout, trace, 0, "complete"),
        (b"", b"\n".join(token.encode("ascii") for token in EXPECTED[:9]) + b"\n",
         0xC0000005, "access_violation"),
        (stdout, trace, 1, "nonzero_exit"),
        (b"x" * (CAP + 1), b"", 0, "output_limit"),
    ):
        now = time.monotonic()
        observed = collect(FakeProcess(out, err, code), now + 1.0, now + 2.0)
        observed_exit, observed_parsed, observed_cleanup, observed_limit, observed_timeout, size = observed
        check(decide(observed_exit, observed_parsed, observed_cleanup,
                     observed_limit, observed_timeout) == wanted)
        check(0 <= size <= CAP and observed_cleanup)
    class BrokenPipe:
        def read(self, _):
            raise OSError("synthetic")

    broken = FakeProcess(b"", b"", 0)
    broken.stderr = BrokenPipe()
    now = time.monotonic()
    observed = collect(broken, now + 1.0, now + 2.0)
    check(not observed[2])
    check(decide(observed[0], observed[1], observed[2], observed[3], observed[4]) == "cleanup_failure")
    now = time.monotonic()
    completed_late = collect(FakeProcess(stdout, trace, 0), now - 1.0, now + 2.0)
    check(completed_late[4])
    check(decide(completed_late[0], completed_late[1], completed_late[2],
                 completed_late[3], completed_late[4]) == "timeout")
    pending = FakeProcess(b"", b"", None)
    now = time.monotonic()
    stopped = collect(pending, now - 1.0, now + 2.0)
    check(pending.kill_count == 1 and stopped[2] and stopped[4])
    check(decide(stopped[0], stopped[1], stopped[2], stopped[3], stopped[4]) == "timeout")
    class FailedKill(FakeProcess):
        def kill(self):
            self.kill_count += 1
            raise OSError("synthetic")

    unkillable = FailedKill(b"", b"", None)
    now = time.monotonic()
    failed_kill = collect(unkillable, now - 1.0, now + 0.03)
    check(unkillable.kill_count == 1 and not failed_kill[2])
    check(decide(failed_kill[0], failed_kill[1], failed_kill[2],
                 failed_kill[3], failed_kill[4]) == "cleanup_failure")
    release_reader = threading.Event()

    class DelayedEof:
        def read(self, _):
            release_reader.wait(timeout=0.2)
            return b""

    unread_eof = FakeProcess(b"", b"", 0)
    unread_eof.stderr = DelayedEof()
    now = time.monotonic()
    missing_eof = collect(unread_eof, now - 1.0, now + 0.02)
    check(not missing_eof[2])
    check(decide(missing_eof[0], missing_eof[1], missing_eof[2],
                 missing_eof[3], missing_eof[4]) == "cleanup_failure")
    release_reader.set()
    for thread in _OWNED[-1][1]:
        thread.join(timeout=0.3)
    check(all(not thread.is_alive() for thread in _OWNED[-1][1]))
    poisoned_receipt = dict(synthetic, status="FAIL", reason="cleanup_failure",
                            exit_class="unobserved", cleanup_complete=False, poisoned=True)
    check(receipt_valid(poisoned_receipt))
    check(not receipt_valid(dict(poisoned_receipt, poisoned=False)))

    # Discovery uses mocked directory entries, canonical paths and metadata:
    # this pure mode neither opens the repository nor starts an executable.
    fake_root = Path("synthetic_root")
    fake_directory = fake_root / "src-tauri" / "target" / "debug" / "deps"
    admitted_name = "aster_desktop_lib-0123456789abcdef.exe"

    class FakeEntry:
        def __init__(self, name, regular=True):
            self.name = name
            self.path = str(fake_directory / name)
            self.regular = regular

        def is_file(self, *, follow_symlinks):
            check(follow_symlinks is False)
            return self.regular

    class FakeScan:
        def __init__(self, entries):
            self.entries = entries

        def __enter__(self):
            return iter(self.entries)

        def __exit__(self, *_):
            return False

    def discovery(entries, *, reject_plain=False, escape=False, expired=False, final_expiry=False):
        def plain(path):
            if reject_plain and path.name == admitted_name:
                raise ValueError("artifact_identity_invalid")

        def canonical(path, *, strict):
            check(strict is True)
            return Path("synthetic_escape") / path.name if escape and path.name == admitted_name else path

        with patch.object(os, "scandir", return_value=FakeScan(entries)), \
                patch.object(Path, "resolve", canonical), \
                patch(__name__ + ".assert_plain", plain):
            try:
                if final_expiry:
                    with patch(__name__ + ".remaining", side_effect=(1.0, 0.0)):
                        result = select_harness(fake_root, time.monotonic() + 1.0)
                else:
                    deadline = time.monotonic() - 1.0 if expired else time.monotonic() + 1.0
                    result = select_harness(fake_root, deadline)
            except ValueError as error:
                check(str(error) == "artifact_identity_invalid")
                return None
            return result

    check(discovery([FakeEntry(admitted_name)]) == fake_directory / admitted_name)
    check(discovery([]) is None)
    check(discovery([FakeEntry(admitted_name),
                     FakeEntry("aster_desktop_lib-fedcba9876543210.exe")]) is None)
    check(discovery([FakeEntry(admitted_name, regular=False)]) is None)
    check(discovery([FakeEntry(admitted_name)], reject_plain=True) is None)
    check(discovery([FakeEntry(admitted_name)], escape=True) is None)
    check(discovery([FakeEntry(admitted_name)], expired=True) is None)
    check(discovery([FakeEntry(admitted_name)], final_expiry=True) is None)
    check(discovery([FakeEntry(admitted_name)]
                    + [FakeEntry("synthetic_other") for _ in range(ENTRY_CAP - 1)]) is not None)
    check(discovery([FakeEntry(admitted_name)]
                    + [FakeEntry("synthetic_other") for _ in range(ENTRY_CAP)]) is None)
    for mode, attributes, reject in ((stat.S_IFREG, 0, False),
                                     (stat.S_IFLNK, 0, True),
                                     (stat.S_IFDIR, 0x400, True)):
        with patch.object(os, "lstat", return_value=SimpleNamespace(
                st_mode=mode, st_file_attributes=attributes)):
            rejected = False
            try:
                assert_plain(fake_directory)
            except ValueError as error:
                rejected = str(error) == "artifact_identity_invalid"
            check(rejected == reject)
    print(json.dumps({"classification": "pure-diagnostic-fixtures-not-native-execution",
                      "status": "PASS", "fixtures": count}, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        if sys.argv[1:] == ["--self-test"]:
            sys.exit(self_test())
        if sys.argv[1:] == ["--run"]:
            sys.exit(run())
        print("artifact_identity_invalid")
        sys.exit(1)
    except Exception:
        print("pure_fixture_failed" if sys.argv[1:] == ["--self-test"] else "cleanup_failure")
        sys.exit(1)
