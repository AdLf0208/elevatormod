# Validation

Validated on Linux with the SDK nightly **`0b14f8acfd97fa2c750e42795249dcc981178ca4`**
(`package_format` 6, `content_api` "0.2", mixins `e59da909…`, game 26.3) on
2026-11-18, for the flat SDK package `elevatorid.hmod` 0.2.0. The retired `.hnmod` ladder
(`tools/horizon.py stage/check/test`, `tools/test_template.py`, `tools/test_stage.py`) belongs
to the previous format and is no longer run; its evidence is in
`../_shared/evidence/elevatorid/retired-hnmod-0.1.0/` and the old logs.

| Check | Result | Evidence |
|---|---|---|
| `cargo test --offline` | 15 passed, 0 failed (10 `runtime::`, 5 `support::`) | `evidence/elevatorid/sdk-test.log` |
| `cargo test` before any glue | 10 passed (the carried planner alone) | `sdk-test-runtime-only.log` |
| `cargo test` with the glue stubbed `todo!()` | 5 failed, as required | `sdk-test-glue-red.log` |
| `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings | `sdk-clippy.log` |
| `cargo fmt --check` | exit 0, no diff | `sdk-fmt.log` |
| `cargo horizon-mod build` | `target/horizon-mod/elevatorid.hmod`, EXIT 0 | `sdk-build.log` |
| second build, `cmp` | byte-identical | `sdk-build-second.log`, `sdk-reproducible.log` |
| `cargo horizon-mod check elevatorid.hmod` | `elevatorid 0.2.0 (native-mod, format native-1/0b14f8a…/e59da90…, for 26.3)`, 134,756 B, sha256 `4cc6fb2d…`, EXIT 0 | `sdk-check.log` |
| `cargo horizon-mod fits elevatorid.hmod --build nightly` | `0b14f8a made for`, EXIT 0 | `sdk-fits.log` |
| `runtime.rs` carry | `ee6aa4ed…3025` in `src/` and in the package = the retired file | `sdk-runtime-in-package.sha256` |
| carried content | 187 files, identical size + sha256 to the retired built content archive | this document, `notes.md` |

The package artifact: **134,756 B**, SHA-256
`4cc6fb2d5edf756349209544b43dfe977e4a795a9b0b865da69580153f499c4b`, **196 entries**
(5 root documents, 187 carried `assets/`+`data/` files, `src/lib.rs`, `src/runtime.rs`,
`src/support.rs`, `uses.json`), no `Cargo.lock`.

**Not validated here: an in-game run.** A native mod is compiled into `horizon-client` by
`horizon-launcher native-build … --trust-native-code`, which is not installed in this
environment, and no game build exists at the pinned SDK. `tools/e2e.sh` is updated for that
build (`GAME=`, `CONTROL_GAME=` differential, `SELFTEST=1` must fail) and has not been run;
the 0.1.0 in-game logs under `../_shared/evidence/elevatorid/` are historical evidence of the
retired bundle's identical behaviour, not of this package. That is gap 1 in `notes.md`.

No Minecraft resources, Mojang files or player data were read or changed by these checks.
