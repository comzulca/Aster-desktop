# Aster strict JPEG entropy completeness patch

This directory preserves the published `zune-jpeg` 0.5.15 package, its version,
Cargo metadata and MIT OR Apache-2.0 OR Zlib licences. The verified upstream
crate archive SHA256 is
`27bc9d5b815bc103f142aa054f561d9187d191692ec7c2d1e2b4737f8dbd7296`.
Its recorded upstream VCS revision is
`31d81fed7551c8ccea456d9d8e2b1fd8bebb6995`, path `crates/zune-jpeg`.
The complete file inventory and original/modified hashes are recorded in
`ASTER-PROVENANCE.json`; the local Cargo extraction stamp is not package source.

## Why the patch exists

Hosted Windows run 37200835238, job 111432154845, rejected the engineering gate
at 2026-10-04T12:20:30Z: the app test
`complete_jpeg_refuses_missing_eoi_partial_entropy_and_suffixes` admitted its
case 2, complete JPEG headers followed by zero entropy and a forged EOI. The
seven negative cases remain unchanged. Upstream marker refill synthesises zero
bits even with strict decoder options; fast bit consumption can silently
saturate/wrap rather than prove that decoded values came from input entropy.

## Narrow changed-source contract

- `src/bitstream.rs`: strict mode uses fallible entropy byte reads and never
  manufactures bits after a marker. Actual bit overconsumption records sticky
  failure across scan/restart resets, including fast AC, amplitudes and
  progressive refinement. Speculative Huffman lookahead may still inspect
  zero-padded lookahead but cannot consume more bits than truly remain. At a
  completed segment, only at most seven one-valued JPEG padding bits may remain.
- `src/mcu.rs`: strict mode requires every declared scan row/component before
  success, does not turn termination into a grey/default-filled image, defers
  lookahead markers until actual scan completion, and uses the component data
  grid for single-component subsampled scans. Partial multi-component scans use
  their interleaved sampling loops. Restart reset cannot conceal sticky failure
  or an EOB run crossing the restart segment.
- `src/mcu_prog.rs`: strict mode tracks completed initial DC scans for every
  component; all scanned blocks/segments must complete with real entropy. Legal
  EOB runs still cover their declared blocks without unnecessary bit reads.
  Nonfinal restart intervals require restart markers; an EOB run may not exceed
  the scan. At a known strict boundary, marker extraction permits FF fill bytes
  but no search over arbitrary bytes that could conceal unconsumed entropy.

The non-strict codec recovery behaviour is preserved. The app explicitly uses
strict mode, RGB output, bounded dimensions, a terminal EOI and exact reader
consumption; it retains EXIF orientation and normalises derived RGBA8 pixels to
metadata-free PNG. This is not original-file/pixel-bit-depth equivalence and does
not authenticate the creator's intended picture: a fully valid independently
encoded JPEG is judged on its own declared image.

## Feature matrix and evidence limits

The app and exact-source codec harness pin 0.5.15 with default features disabled
and `std` enabled; registry `zune-core` remains 0.5.3. Upstream default/x86/neon,
portable SIMD and logging metadata/code remain unchanged; they have not been
separately qualified by this patch. No new format, decoder, resize or permission
is introduced. The app's seven partial/trailing JPEG negatives are retained and
its inline fixtures additionally cover 1x1/3x2/32x32 baseline JPEG and synthetic
3x2/16x16 progressive JPEG with pixel-content witnesses and forged no-entropy EOI.
Internal strict bitstream tests cover sticky reset failure and a legitimate
short tail, but dependency unit tests are not implied by app test execution.

At initial patch freeze, Rust formatting and source review are complete; codec
execution, hosted compilation/tests, native worker/resource qualification and
the final engineering artifact remain NOT RUN for this changed source. No local
codec/helper execution was attempted because Windows Application Control had
blocked the local build helper. Later CI evidence must refer to its exact source
revision and must not overwrite the historical hosted failure with an inferred
PASS. This source patch does not establish a general RCE/no-network sandbox.
