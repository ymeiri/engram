# Native stale v3 implementation baseline — 2026-09-10

Status: provider-free baseline evidence; no provider, credential, deletion, installation, or pilot
execution authority

## Source identity

~~~text
git HEAD                                4e2b4c1e0047633bbc0a640c1966e60774925f82
Cargo.lock                              0eb924bb9008bb417cb9990081df8429520131aa6b3f7367866c5b805ec474de
engram-eval/src/native_stale.rs         37ef2031c6e2207cac5b27eefe83f5f4c2ff0e5fd1fdabc76869499953d555e4
engram-eval/src/native_pilot.rs         79db7c622fca2357e4cbb915ecc3533638f5fa21e893df98a17956da7dce67cc
engram-eval/src/native_runner.rs        e59faf8301ccbf36a33cd23b7a4f6aba76e0a0e2acc6e7281cf6f1c4e76fb5b5
~~~

The worktree was already dirty and remains user-owned. Repository `target/debug` was absent before
and after this check.

## Build prerequisite

The first cold build stopped before tests because `ort-sys 2.0.0-rc.9` attempted its direct Pyke
download and received HTTP 403. The existing Engram procedure was followed using Microsoft's
official ONNX Runtime v1.20.0 macOS arm64 release asset:

~~~text
archive /private/tmp/engram-onnxruntime-1.20.0.suTH2W/onnxruntime-osx-arm64-1.20.0.tgz
sha256  2bcfaafa9ff0a3a94f78e3af2f135ffde5bb2d79b08e83a50dbc450b0d20ddae
bytes   7875127

dylib   /private/tmp/engram-onnxruntime-1.20.0.suTH2W/extracted/onnxruntime-osx-arm64-1.20.0/lib/libonnxruntime.1.20.0.dylib
sha256  d8be733cb8dd097cfe2b21e069a7462b5ff561625141d9c4b98d866f15bfb852
bytes   25477000
~~~

The hashes match the previously recorded accepted build prerequisite. No credential or provider
material was accessed.

## Baseline command and result

~~~sh
env \
  ORT_LIB_LOCATION=/private/tmp/engram-onnxruntime-1.20.0.suTH2W/extracted/onnxruntime-osx-arm64-1.20.0 \
  ORT_PREFER_DYNAMIC_LINK=1 \
  DYLD_LIBRARY_PATH=/private/tmp/engram-onnxruntime-1.20.0.suTH2W/extracted/onnxruntime-osx-arm64-1.20.0/lib \
  CARGO_TARGET_DIR=/private/tmp/engram-native-stale-target-20260910 \
  cargo test --offline -p engram-eval native_stale
~~~

Result: **PASS**. The `engram-eval` library ran 24 matching tests, with 24 passed, zero failed,
zero ignored, and 393 filtered out. Other compiled test binaries selected zero matching tests. This
proves the pre-change stale-family baseline only; it does not validate family v3 or a provider
outcome.

## Candidate validation failure

Running the current evaluator against
`protocol-native-stale-safety-v3-base-schema-10-direct-host-file-cache.json` stops before family-v3
validation:

~~~text
native pilot abstention case case-4f2a6d1c9b830e57 requires a non-empty
condition_evidence_output_contains
~~~

Source inspection confirms that the missing-source case replaced the base field
`evaluation_prerequisite_state: "missing"` with an unknown prose field, so deserialization defaults
the base state to `unchanged`. The candidate is therefore non-executable and requires a
forward-only corrected protocol plus family-v3 implementation.
