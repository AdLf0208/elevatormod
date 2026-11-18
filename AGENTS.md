# Working on this Horizon native mod

This is a **flat SDK package** (mod API 0.2): `horizon-mod.toml` + `Cargo.toml` + `src/lib.rs`
(`impl mod_api::Hooks`) + `src/runtime.rs` (the pure rules and their tests) + `assets/` + `data/`.
Horizon retired the `.hnmod` container in the mod-API merge (#16941), so there is no
`compatibility.json`, `mixins/<id>/`, `horizon-mixin.toml`, plan/injection schema or
`rust-toolchain.toml` here, and none of them should be recreated. The retired tree is at
`../_shared/evidence/elevatorid/retired-hnmod-0.1.0/`.

> **Build slots (v2):** every build/test goes through the campaign runner
> (`ports/tools/build-slot.sh` from the campaign root). It **blocks in the kernel** until a slot is
> free, so **never poll and never background a build**; a timeout exits **75**. It does **not**
> `cd` into the port — wrap the command (`bash -c 'cd ports/elevatorid && …'`, or guard the `cd`
> so a forwarded run that already starts in the port still works) and never pass `--target-dir`.
> Read `../_shared/build-slots-usage.md` before running anything.

```sh
ports/tools/build-slot.sh run --job elevatorid-sdk --port ports/elevatorid --step test \
    -- bash -c 'if [ -d ports/elevatorid ]; then cd ports/elevatorid; fi; cargo test --offline'
ports/tools/build-slot.sh run --job elevatorid-sdk --port ports/elevatorid --step build \
    -- bash -c 'if [ -d ports/elevatorid ]; then cd ports/elevatorid; fi; rm -f Cargo.lock && cargo horizon-mod build'
```

Rules that bite:

- **`src/runtime.rs` is byte-identical** to the retired bundle's file and must stay so; the port's
  new logic belongs in `src/support.rs` (glue) or in a new pure module, never in `runtime.rs`.
- **`rm -f Cargo.lock` before every `cargo horizon-mod build`** — a packed lock changes the
  artifact's bytes. `.gitignore` keeps `/Cargo.lock` out of the tree.
- **`cargo fmt --check`, never `--all`**: `--all` formats the fetched SDK under `.horizon/sdk`.
- **The package root holds package paths only.** Keep the retired layout out (it is in the
  evidence folder) and do not add `tools/horizon.py`, `package.py` or an audit crate.
- **A launcher builds the game**: `horizon-launcher native-build . --instance <i>
  --trust-native-code`; `cargo horizon-mod build` only packs. `tools/e2e.sh` takes `GAME=` for
  that build and prints `EXIT:<status>`.
- **Nothing is published without the maintainer** (ARCHITECTURE.md §8); the port is local-only.
