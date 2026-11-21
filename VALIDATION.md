# Validation

Validated on Linux with the SDK nightly
**`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (`package_format` 6, `content_api` "0.2",
mixins `e59da909…`, game 26.3) on 2026-11-21, for the flat SDK package `elevatorid.hmod`
**0.2.2**. The retired `.hnmod` ladder (`tools/horizon.py stage/check/test`) belongs to the
previous format and is no longer run; its evidence is in
`../_shared/evidence/elevatorid/retired-hnmod-0.1.0/` and the old logs.

Every command ran in a `ports/tools/build-slot.sh` lease with `CARGO_INCREMENTAL=0`; the package
steps used `BUILD_LOCAL=1` (the second machine has no `cargo-horizon-mod`).

| Check | Result | Evidence |
|---|---|---|
| `cargo test` before the arrival rule existed | 16 passed, **4 failed** `not yet implemented` (`EXIT:101`) | `evidence/elevatorid/sdk-test-arrival-red.log` |
| `cargo test --offline` | **20 passed**, 0 failed (10 `runtime::`, 6 `support::`, 4 `arrival::`) | `sdk-test.log` |
| `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings | `sdk-clippy.log` |
| `cargo fmt --check` | exit 0, no diff | `sdk-fmt.log` |
| `cargo horizon-mod build` (×2, `rm -f Cargo.lock` before each) | `SDK 0a875da … in ./.horizon/sdk`, byte-identical | `sdk-build.log`, `sdk-build-second.log`, `sdk-reproducible.log` |
| `cargo horizon-mod check elevatorid.hmod` | `elevatorid 0.2.2 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, 139,378 B, sha256 `f17dcbc7…`, EXIT 0 | `sdk-check.log` |
| `cargo horizon-mod fits elevatorid.hmod --build nightly` | `0a875da  made for`, EXIT 0 (the pin: a tautology, kept for the record) | `sdk-fits.log` |
| `cargo horizon-mod fits elevatorid.hmod --build 0b14f8a…` | `0b14f8a  fits`, EXIT 0 | `sdk-fits-other.log` |
| `runtime.rs` carry | `ee6aa4ed…3025` in `src/` and inside the package (byte-identical to the retired file) | `sdk-entries.log` |
| in-game leg (`launcher-game.sh build`, `--offscreen`) | `PASS` + `EXIT:0` — travel, colour, arrival view, 16/16 recipes | `evidence/elevatorid-e2e/e2e-launcher.log` |
| control (same assertions, unmodified Nightly) | `FAIL: expected y=67 after jump, got 64.0` + `EXIT:1` | `e2e-launcher-control.log` |
| selftest, blocked target | `FAIL: expected y=64 …, got 70.0` + `EXIT:1` | `e2e-selftest.log` |
| selftest, this slice's family | `FAIL: expected yaw=33 …, got 270.0` + `EXIT:1` | `e2e-selftest-look.log` |
| `check-e2e-logs.py elevatorid-e2e --strict` | 4/4 ok | — |
| `verify-sdk-package.py elevatorid` | 1/1 | — |

The package artifact: **139,378 B**, SHA-256
`f17dcbc7f3f4df8aec1392fef3cb40af93bff7d4ba1e80df8317ea3a7048de0e`, **197 entries**
(5 root documents, 187 carried `assets/`+`data/` files, `src/lib.rs`, `src/arrival.rs`,
`src/runtime.rs`, `src/support.rs`, `uses.json`), **no `Cargo.lock`**.

The in-game leg launches `--offscreen` (not `--headless`): the recipe phase drives a crafting
table with `ctl click-slot`, which needs a renderer. The build was made from this tree and the
log shows `Compiling elevatorid (1 of 1)`; the packaged `src/*.rs` are byte-identical to the tree.

No Minecraft resources, Mojang files or player data were read or changed by these checks.
