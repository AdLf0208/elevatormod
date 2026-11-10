# elevatorid (Horizon port)

A Horizon port of [elevatorid](VsnGamer/ElevatorMod) — MIT.

**Status: first slices.** What is below is what the port actually does today.

## Implemented so far

- Origin**: from the feet block, scan down at most `ACTIVATION_RANGE` (6)
- cells; the first elevator is the origin; if the cell above it suffocates the
- attempt aborts (upstream does not scan deeper).
- Target**: step one cell at a time in the travel direction, stop at
- `RANGE` (384, inclusive); the first elevator whose above-cell does not
- suffocate wins; blocked cells are skipped, and with `sameColor` a different
- dye is skipped.
- Teleport**: destination is the target block's top, x/z centred

## Not yet ported

- Teleport motion** (documented deviation): the native helper `/tp`s as a
- command does — the player's motion is lost (upstream keeps horizontal
- velocity) and no upstream sound/particles are sent. The console-command path
- (level-4 permission, command feedback) is gone since slice 2.
- Input**: raw key edges unavailable; jump inferred from the body (see above),
- with `player_jump` as the exact edge at this pin.
- An issue-shaped request: give `native_player_tick` the player's input or
- `on_ground`/jump flag.

## Install

Build the bundle with a native-mod capable launcher, or add the file from the
registry once it is listed. Pinned to Horizon `1d63e0a` and horizon-mixins
`5a64af93` for game 26.3.

## Build from source

```sh
cargo test
python3 tools/horizon.py check --horizon <horizon checkout>
python3 tools/horizon.py build --horizon <horizon checkout>
python3 tools/package.py
```

## Credits and licence

Original mod: VsnGamer/ElevatorMod. Port: AdLf0208. Licence: MIT — see [LICENSE](LICENSE);
the original notices are preserved.
