"""Independently validate the bounded parent-authored CPU containment receipt.

Supplementary to, never a substitute for, the existing native exit, all fifteen
codec/six resource checks and source/package guards. No process is launched.
"""

from __future__ import annotations

import argparse
import copy
import json
from pathlib import Path

CPU_TICKS = 100_000_000
WALL_MILLIS = 15_000
CPU_EXIT = 2_776_498_177
MAX_RECEIPT_BYTES = 64 * 1024
MAX_I64 = (1 << 63) - 1
MAX_U64 = (1 << 64) - 1
MAX_U32 = (1 << 32) - 1
CODEC_CODES = (
    "native-text", "native-png", "native-jpeg", "native-static-gif",
    "native-two-page-pdf", "deny-malformed-png", "deny-animated-gif",
    "deny-pixel-overflow", "deny-dimension-overflow", "deny-pdf-page-overflow",
    "deny-malformed-pdf", "deny-renamed-pdf", "deny-invalid-utf-eight",
    "deny-text-overflow", "deny-source-overflow",
)
RESOURCE_CODES = (
    "cpu-kill", "memory-denial", "wall-kill", "active-process-denial",
    "kill-on-close", "cancel-kill",
)
PROOF_BOOLEANS = (
    "samplingValid", "liveChildObserved", "priorityRechecked",
    "exactJobDispatched", "physicalExpectedExit", "cleanupComplete",
    "terminalDenied", "exactMarker", "statsValid", "naturalExit",
    "jobClosed", "workerError", "fallbackUsed",
)
PROOF_KEYS = frozenset(("mechanism", "triggerUser100ns", "triggerWallMillis", *PROOF_BOOLEANS))
RESOURCE_KEYS = frozenset((
    "code", "status", "reason", "elapsedMillis", "exitCode", "cpuUser100ns",
    "peakMemoryBytes", "limitTerminations", "boundaryAvailable", "cpuProof",
))


class InvalidReceipt(ValueError):
    """Closed, content-free failure: do not echo receipt bytes or paths."""


def require(condition: bool) -> None:
    if not condition:
        raise InvalidReceipt("invalid bounded CPU receipt")


def keys(value: object, expected: frozenset[str]) -> dict:
    require(type(value) is dict and set(value) == expected)
    return value


def integer(value: object, maximum: int) -> int:
    require(type(value) is int and 0 <= value <= maximum)
    return value


def reject_duplicates(pairs: list[tuple[str, object]]) -> dict:
    result = {}
    for key, value in pairs:
        require(key not in result)
        result[key] = value
    return result


def invalid_constant(_: str) -> None:
    raise InvalidReceipt("non-finite JSON")


def decode(raw: bytes) -> dict:
    require(0 < len(raw) <= MAX_RECEIPT_BYTES)
    # No BOM, duplicate key, non-finite numeric constant or permissive decoding.
    return json.loads(raw.decode("utf-8"), object_pairs_hook=reject_duplicates,
                      parse_constant=invalid_constant)


def validate_cpu(cpu: dict) -> str:
    proof = keys(cpu["cpuProof"], PROOF_KEYS)
    require(type(proof["mechanism"]) is str)
    trigger = integer(proof["triggerUser100ns"], MAX_I64)
    wall = integer(proof["triggerWallMillis"], MAX_U64)
    for name in PROOF_BOOLEANS:
        require(type(proof[name]) is bool)
    for name in ("samplingValid", "cleanupComplete", "terminalDenied", "exactMarker", "statsValid", "workerError"):
        require(proof[name] is True)
    require(proof["jobClosed"] is False and proof["fallbackUsed"] is False)
    require(cpu["boundaryAvailable"] is True and cpu["cpuUser100ns"] >= CPU_TICKS)
    if proof["mechanism"] == "native-job-cpu-limit":
        require(proof["naturalExit"] is True and cpu["exitCode"] != 0
                and cpu["limitTerminations"] > 0 and trigger == 0 and wall == 0)
        for name in ("liveChildObserved", "priorityRechecked", "exactJobDispatched", "physicalExpectedExit"):
            require(proof[name] is False)
    elif proof["mechanism"] == "parent-user-cpu-watchdog":
        require(proof["naturalExit"] is False and trigger >= CPU_TICKS
                and wall < WALL_MILLIS and cpu["cpuUser100ns"] >= trigger
                and cpu["elapsedMillis"] >= wall and cpu["exitCode"] == CPU_EXIT)
        for name in ("liveChildObserved", "priorityRechecked", "exactJobDispatched", "physicalExpectedExit"):
            require(proof[name] is True)
    else:
        # Unknown and unproved/none are never a CPU PASS.
        raise InvalidReceipt("unproved CPU mechanism")
    return proof["mechanism"]


def validate(report: object) -> str:
    report = keys(report, frozenset(("schemaVersion", "classification", "allPassed",
                                  "qualificationComplete", "cases", "resourceKillChecks")))
    require(type(report["schemaVersion"]) is int and report["schemaVersion"] == 1)
    require(report["classification"] == "synthetic-byte-worker-fixture")
    require(report["allPassed"] is True and report["qualificationComplete"] is False)
    require(type(report["cases"]) is list and len(report["cases"]) == 15)
    require(type(report["resourceKillChecks"]) is list and len(report["resourceKillChecks"]) == 6)
    for expected, case in zip(CODEC_CODES, report["cases"], strict=True):
        case = keys(case, frozenset(("code", "status", "reason")))
        require(case["code"] == expected and case["status"] == "PASS")
        require(case["reason"] == ("exact-synthetic-content-and-boundary"
                if expected.startswith("native-") else "whole-operation-denied"))
    for expected, resource in zip(RESOURCE_CODES, report["resourceKillChecks"], strict=True):
        resource = keys(resource, RESOURCE_KEYS)
        require(resource["code"] == expected and resource["status"] == "PASS"
                and resource["reason"] == "observed-synthetic-resource-containment")
        integer(resource["elapsedMillis"], MAX_U64)
        integer(resource["exitCode"], MAX_U32)
        integer(resource["cpuUser100ns"], MAX_I64)
        integer(resource["peakMemoryBytes"], MAX_U64)
        integer(resource["limitTerminations"], MAX_U32)
        require(type(resource["boundaryAvailable"]) is bool and resource["boundaryAvailable"] is True)
        if expected != "cpu-kill":
            require(resource["cpuProof"] is None)
    return validate_cpu(report["resourceKillChecks"][0])


def fixture(mechanism: str) -> dict:
    parent = mechanism == "parent-user-cpu-watchdog"
    resources = []
    for code in RESOURCE_CODES:
        resources.append(dict(code=code, status="PASS", reason="observed-synthetic-resource-containment",
                              elapsedMillis=10_150, exitCode=CPU_EXIT if parent else 1,
                              cpuUser100ns=CPU_TICKS, peakMemoryBytes=1_638_400,
                              limitTerminations=0 if parent else 1, boundaryAvailable=True, cpuProof=None))
    resources[0]["cpuProof"] = dict(
        mechanism=mechanism, triggerUser100ns=CPU_TICKS if parent else 0,
        triggerWallMillis=10_100 if parent else 0, samplingValid=True,
        liveChildObserved=parent, priorityRechecked=parent, exactJobDispatched=parent,
        physicalExpectedExit=parent, cleanupComplete=True, terminalDenied=True,
        exactMarker=True, statsValid=True, naturalExit=not parent, jobClosed=False,
        workerError=True, fallbackUsed=False,
    )
    return dict(schemaVersion=1, classification="synthetic-byte-worker-fixture",
                allPassed=True, qualificationComplete=False,
                cases=[dict(code=code, status="PASS", reason="exact-synthetic-content-and-boundary"
                       if code.startswith("native-") else "whole-operation-denied") for code in CODEC_CODES],
                resourceKillChecks=resources)


def self_test() -> int:
    negatives = 0

    def reject(report: object) -> None:
        nonlocal negatives
        try:
            validate(report)
        except (InvalidReceipt, ValueError, TypeError):
            negatives += 1
        else:
            raise AssertionError("negative CPU control admitted")

    for mechanism in ("parent-user-cpu-watchdog", "native-job-cpu-limit"):
        good = fixture(mechanism)
        require(validate(decode(json.dumps(good).encode())) == mechanism)
        proof = good["resourceKillChecks"][0]["cpuProof"]
        # Every closed proof key is independently removed and forged; unknown
        # fields and wrong types fail instead of being silently ignored.
        for name in PROOF_KEYS:
            changed = copy.deepcopy(good)
            del changed["resourceKillChecks"][0]["cpuProof"][name]
            reject(changed)
            changed = copy.deepcopy(good)
            changed["resourceKillChecks"][0]["cpuProof"][name] = None
            reject(changed)
        for name in PROOF_BOOLEANS:
            changed = copy.deepcopy(good)
            changed["resourceKillChecks"][0]["cpuProof"][name] = not proof[name]
            reject(changed)
            changed = copy.deepcopy(good)
            changed["resourceKillChecks"][0]["cpuProof"][name] = int(proof[name])
            reject(changed)
        for name, value in (("mechanism", "none"), ("mechanism", "wall-kill"),
                            ("triggerUser100ns", -1), ("triggerUser100ns", MAX_I64 + 1),
                            ("triggerUser100ns", True), ("triggerWallMillis", True),
                            ("triggerWallMillis", MAX_U64 + 1), ("triggerWallMillis", -1)):
            changed = copy.deepcopy(good)
            changed["resourceKillChecks"][0]["cpuProof"][name] = value
            reject(changed)
        for name, value in (("exitCode", 0), ("exitCode", None), ("exitCode", True),
                            ("exitCode", MAX_U32 + 1), ("cpuUser100ns", CPU_TICKS - 1),
                            ("cpuUser100ns", True), ("limitTerminations", True),
                            ("elapsedMillis", True), ("peakMemoryBytes", True),
                            ("boundaryAvailable", False), ("boundaryAvailable", 1)):
            changed = copy.deepcopy(good)
            changed["resourceKillChecks"][0][name] = value
            reject(changed)
        changed = copy.deepcopy(good)
        changed["resourceKillChecks"][0]["cpuProof"]["unknown"] = True
        reject(changed)
        changed = copy.deepcopy(good)
        changed["resourceKillChecks"][0]["status"] = "FAIL"
        reject(changed)
        changed = copy.deepcopy(good)
        changed["resourceKillChecks"][0]["reason"] = "timeout"
        reject(changed)
        for name, value in (("schemaVersion", True), ("allPassed", 1), ("qualificationComplete", True)):
            changed = copy.deepcopy(good)
            changed[name] = value
            reject(changed)
    good = fixture("parent-user-cpu-watchdog")
    for name, value in (("triggerUser100ns", CPU_TICKS - 1), ("triggerUser100ns", CPU_TICKS + 1),
                        ("triggerWallMillis", WALL_MILLIS)):
        changed = copy.deepcopy(good)
        changed["resourceKillChecks"][0]["cpuProof"][name] = value
        reject(changed)
    changed = copy.deepcopy(good)
    changed["resourceKillChecks"][0]["elapsedMillis"] = 10_099
    reject(changed)
    for exit_code in (1, 3, CPU_EXIT - 1):
        changed = copy.deepcopy(good)
        changed["resourceKillChecks"][0]["exitCode"] = exit_code
        reject(changed)
    good = fixture("native-job-cpu-limit")
    changed = copy.deepcopy(good)
    changed["resourceKillChecks"][0]["limitTerminations"] = 0
    reject(changed)
    # The two real historical failures had final CPU above budget but generic
    # exit1 / zero native terminations / no parent causal proof. Never promote.
    for historical_ticks in (147_656_250, 148_437_500):
        changed = copy.deepcopy(good)
        changed["resourceKillChecks"][0].update(cpuUser100ns=historical_ticks, exitCode=1,
                                               limitTerminations=0, elapsedMillis=15_029)
        del changed["resourceKillChecks"][0]["cpuProof"]
        reject(changed)
    for raw in (b'{"a":1,"a":2}', b'{"a":NaN}', b'{"a":Infinity}',
                b'{"a":-Infinity}', b'\xef\xbb\xbf{}', b'\xff', b'x' * (MAX_RECEIPT_BYTES + 1),
                b'[' * 2000 + b'0' + b']' * 2000):
        try:
            validate(decode(raw))
        except (InvalidReceipt, ValueError, UnicodeError, RecursionError):
            negatives += 1
        else:
            raise AssertionError("malformed JSON admitted")
    return negatives


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--self-test", action="store_true")
    mode.add_argument("--receipt", type=Path)
    args = parser.parse_args()
    try:
        if args.self_test:
            count = self_test()
            print(f"CPU receipt verifier self-test PASS ({count} negative controls).")
        else:
            with args.receipt.open("rb") as stream:
                raw = stream.read(MAX_RECEIPT_BYTES + 1)
            mechanism = validate(decode(raw))
            print(f"CPU receipt verifier PASS ({mechanism}); not product qualification.")
    except (InvalidReceipt, ValueError, TypeError, OSError, UnicodeError, AssertionError, RecursionError):
        print("CPU receipt verifier FAIL; package promotion denied.")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
