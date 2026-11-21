# ElevatorMod for Horizon

Sixteen coloured elevator blocks and the ride between them, for Horizon (Minecraft 26.3) as one
flat SDK package: `horizon-mod.toml` + `Cargo.toml` + `src/` + `assets/` + `data/`.

Ported from **ElevatorMod 1.21.1-1.11.4** (`VsnGamer/ElevatorMod`, MIT) — a clean-room Rust
rewrite from the upstream source read for behaviour only; the sixteen blocks, their recipes,
loot, tags, models, textures and translations are carried from the jar unchanged.

## What it does

- **Jump on an elevator** to ride the column up to the nearest elevator **of the same colour**
  above you; **sneak** to ride it down to the nearest one below. Another colour's landing in the
  column is skipped, so a public and a private floor can share a column.
- The destination is the top of the target block, x/z centred on it, and your view is kept.
- A target whose cell above suffocates is skipped and the scan continues past it, up to the
  upstream default range of 384 blocks; the origin is the first elevator at most 6 blocks below
  your feet.
- Both edges come from the server's own reports (`player_jump`, `player_toggle_sneak`), so a jump
  clipped by a low ceiling still rides, and holding jump does not climb repeatedly (a 10-tick
  cooldown).
- Spectators do not ride.

## What it does not do yet

Camouflage (a block applied to an elevator), the per-block options screen and its
directional/arrow state, the XP cost, mob spawning, the teleport sound and particles, and the
`serverconfig` file (the defaults above are constants). The block template is opaque, so an
elevator looks right but is not `noOcclusion`. See `notes.md` for the full gap list and the
upstream-defect rulings.

## Build and verify

The pinned SDK is fetched into `.horizon/sdk/` by any `cargo horizon-mod` command (gitignored),
and the manifest pins the game build, Horizon and mixins commits.

```sh
cargo test                                          # 15 tests: the planner and the travel queue
cargo clippy --all-targets -- -D warnings
cargo fmt --check                                   # not --all: that formats the fetched SDK
rm -f Cargo.lock                                    # a packed lock changes the artifact
cargo horizon-mod build                             # -> target/horizon-mod/elevatorid.hmod
cargo horizon-mod check elevatorid.hmod
cargo horizon-mod fits elevatorid.hmod --build nightly
```

A game carrying the mod is built by the launcher, which compiles `src/` against the pinned
build: `horizon-launcher native-build ports/elevatorid --instance <i> --trust-native-code`.
`tools/e2e.sh` then drives the jump/sneak scenario with `GAME=` pointing at that build (and
`CONTROL_GAME=` at an unmodified build for the differential). It needs the prepared 26.3
resources and `jq`.

## Layout

| Path | What it is |
|---|---|
| `horizon-mod.toml` | the only manifest: id, version, `api = "0.2"`, side, capabilities, `[native]` pins |
| `Cargo.toml` | one crate; the only dependency is `horizon-mod-api` from `.horizon/sdk/` |
| `src/lib.rs` | the one `#[mod_api::horizon_mod] impl Hooks` |
| `src/support.rs` | the glue: the travel queue, the column reads, the teleport |
| `src/runtime.rs` | the pure planner and its tests (byte-identical to the retired bundle's) |
| `assets/elevatorid/`, `data/elevatorid/` | the carried content: 16 blocks, items, models, textures, recipes, loot, tags, lang |
| `tools/e2e.sh` | the in-game scenario |
| `notes.md`, `VALIDATION.md`, `REPIN-AUDIT.md` | the port's history, evidence and gaps |
| `LICENSE`, `LICENSE-NOTICE.md` | MIT text with the original copyright, and where it was read |

## Licence

MIT. `LICENSE` keeps the original `Copyright (c) 2016 Vasco (ElevatorMod)` beside the port's own.
Local-only until the maintainer approves publishing (ARCHITECTURE.md §8).
