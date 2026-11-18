# ElevatorMod — a Horizon port

**ElevatorMod** is a Minecraft mod. Port of the elevator from OpenBlocks for Minecraft 1.8+. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [VsnGamer/ElevatorMod](https://github.com/VsnGamer/ElevatorMod) — licence **MIT**.

| | |
|---|---|
| Upstream mod | ElevatorMod ([source](https://github.com/VsnGamer/ElevatorMod)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `elevatorid.hmod` (install it with the Horizon launcher) |
| Licence | **MIT** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

- **Origin**: from the feet block, scan down at most `ACTIVATION_RANGE` (6)
- **Target**: step one cell at a time in the travel direction, stop at
- **Teleport**: destination is the target block's top, x/z centred
- 16 coloured elevator blocks (white base: 8 wool in a ring + 1 ender pearl;
- Stand on an elevator and **jump → teleport up** the same column;
- A candidate elevator is valid only with ≥1 non-solid block directly above.
- Config (`serverconfig/elevatorid-server.toml`, NeoForge 1.21.1 defaults):
- Extras not in scope for the first slice: per-block options GUI

## Not yet ported

- **Teleport motion** (documented deviation): the native helper `/tp`s as a
- **Input**: raw key edges unavailable; jump inferred from the body (see above),
- **Suffocation**: approximated with a full-collision block
- **Not carried**: camouflage, options GUI, directional model/arrow/yaw,
- The 16 blocks use the `stone` template (opaque, light-dampening 15), so an
- **The two fallback edges are gone.** `player_tick`'s body predicate and the window's Space edge
- **Suffocation now means "a collision box covers the whole cell" from the level's own shapes**
- **The cooldown tick comes from the context**, not a self-counted tick (`TickId::get()`); same

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `elevatorid.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

ElevatorMod was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **MIT**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/VsnGamer/ElevatorMod

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run elevatorid` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/elevatorid/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
