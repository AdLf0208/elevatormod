# ElevatorMod — port notes (content / logic, core, P0)

Ported from **ElevatorMod 1.21.1-1.11.4** (`VsnGamer/ElevatorMod`, MIT) for
Minecraft Java 1.21.1 NeoForge. Local-only port.

Status: `ported-partial` — **0.2.2, one flat SDK package** (mod API 0.2, nightly
`0a875da`), migrated 2026-11-18 from the retired `.hnmod` container; see
[`## Migration to the SDK format`](#migration-to-the-sdk-format-020-2026-11-18)
and the 0.2.2 slice at the end ([`## Slice 3 — the arrival view`](#slice-3--the-arrival-view-directional-elevators-022-2026-11-21)).
Slice 1: all 16 coloured elevators, jump-up / sneak-down travel between blocks
in the same column, nearest target, range 384, activation range 6,
sameColor=false, precision target (x/z centred); slice 2 (`1d63e0a`): the
transport is the pin's native player teleport; slice 3 (`0a875da`): the
`facing`/`directional` block states and the arrival view (a directional target
turns you and levels the pitch), plus a real in-game craft of all 16 recipes.
Out of scope: camouflage, options GUI, the arrow overlay, XP, sound/particles,
mob spawning, config file.
The package **is** run in game since 0.2.1 (`horizon-launcher native-build`,
`ports/_shared/evidence/elevatorid-e2e/`); the 0.1.0 runs under
`_shared/evidence/elevatorid/` are historical evidence of the same behaviour.


## Behaviour (slice 1)

- **Origin**: from the feet block, scan down at most `ACTIVATION_RANGE` (6)
  cells; the first elevator is the origin; if the cell above it suffocates the
  attempt aborts (upstream does not scan deeper).
- **Target**: step one cell at a time in the travel direction, stop at
  `RANGE` (384, inclusive); the first elevator whose above-cell does not
  suffocate wins; blocked cells are skipped, and with `sameColor` a different
  dye is skipped.
- **Teleport**: destination is the target block's top, x/z centred
  (`precisionTarget`). Slice 2 (`1d63e0a`) queues the travel and moves the
  player with `native_teleport_player(uuid, dimension, pos, yaw, pitch)` from
  `server_tick`; the slice-1 path was
  `execute in <dimension> run tp <uuid> <x> <y> <z>` through
  `native_run_command` (see the last section).

## Input detection

Upstream is client-driven (raw jump/shift key edges). At the pin:
- **sneak** = `PlayerMotion::shift_key_down` rising edge in `player_tick`
  (reliable, client-fed);
- **jump** = the **window's** `tick` hook reads the raw Space key edge
  (`Game::keys`) and queues it by player name; the server never sees raw input,
  and `player_tick` runs after movement (post-collision velocity) with its
  `history` already drained by the block-damage path, so neither is usable.
  Server-side `velocity.y > 0.05` remains a fallback for unobstructed take-offs
  (e.g. LAN guests).
A 10-tick per-player cooldown prevents a held jump from climbing repeatedly.
Known limit: a guest's ceiling-clipped jump is not detected (only the host's
raw Space is visible); an input/on-ground API on `native_player_tick` would
close it.

## Hooks (0.1.0, retired `.hnmod` format, `1d63e0a`) — superseded by the migration section

| Hook | Helper | What it does |
|---|---|---|
| `player_jump` (v1, API 344) | `jump.rs` | The exact jump edge from accepted movement (a ceiling-clipped jump is still seen); queue up. |
| `player_toggle_sneak` (v1, API 351) | `sneak.rs` | The sneak press edge; queue down. |
| `player_tick` (v1, API 31) | `player.rs` | Body-predicate fallback for a jump take-off (`history` upward move); queue. |
| `tick` (v1, API 4) | `tick.rs` | The window's raw Space edge (host); queue by player name. |
| `server_tick` (v1, API 4) | `server.rs` | At End of a running tick: scan the column, then `native_teleport_player`. |

## Gaps

- **Teleport motion** (documented deviation): the native helper `/tp`s as a
  command does — the player's motion is lost (upstream keeps horizontal
  velocity) and no upstream sound/particles are sent. The console-command path
  (level-4 permission, command feedback) is gone since slice 2.
- **Input**: raw key edges unavailable; jump inferred from the body (see above),
  with `player_jump` as the exact edge at this pin.
  An issue-shaped request: give `native_player_tick` the player's input or
  `on_ground`/jump flag.
- **Suffocation**: approximated with a full-collision block
  (`facts.full_collision && !air`); glass/leaves corner cases differ.
- **Not carried**: camouflage, options GUI, directional model/arrow/yaw,
  `resetPitch*`, XP cost, mob spawning, config (`config/elevatorid-server.toml`)
  — defaults are constants. Content declares no `facing`/`directional`/
  `show_arrow` states yet.
- The 16 blocks use the `stone` template (opaque, light-dampening 15), so an
  un-camouflaged elevator looks right but is not `noOcclusion`.

## In-game check

`tools/e2e.sh`: The End floor with elevators at y=63/66/69 in one column —
`ctl jump` lands at y=67 (nearest 66), `hold shift 0.2` returns to y=64,
a plain-floor jump stays put (no travel), a lone elevator gives no target, and
a stone block above the middle elevator makes the jump skip to 69 (y=70).

# Elevator ID / ElevatorMod — port notes (P3 logic tier)

Ported from **ElevatorMod** (Modrinth id `hi2dSXTu`, mod id `elevatorid`, MIT;
repo `VsnGamer/ElevatorMod`), file `elevatorid-neoforge-1.21.1-1.11.4`.
Status: **scoped, not implemented** — needs a data carry (block/item/assets)
plus a player-teleport path.

## Behaviour (public docs only)

- 16 coloured elevator blocks (white base: 8 wool in a ring + 1 ender pearl;
  re-dye shapeless with any dye).
- Stand on an elevator and **jump → teleport up** the same column;
  **sneak → teleport down**. X/Z never change.
- A candidate elevator is valid only with ≥1 non-solid block directly above.
- Config (`serverconfig/elevatorid-server.toml`, NeoForge 1.21.1 defaults):
  `range=384`, `sameColor=false`, `precisionTarget=true`, `activationRange=6`,
  `mobSpawn=false`, `resetPitchNormal=false`, `resetPitchDirectional=true`,
  `useXP=false`, `XPPointsAmount=1`.
- Extras not in scope for the first slice: per-block options GUI
  (directional/hide arrow), camouflage with any solid block, particles tinted
  by the camouflage, XP cost, mob spawning.
- Not documented: nearest vs farthest target choice, obstruction rules,
  jump/sneak edge cases, fall-damage cancellation, exact sounds.

## Data carry (required first)

`content/` must gain, from the original jar (Modrinth CDN by SHA-1):
- 16 blocks + block items: blockstates/models (directional facing variants),
  item models, textures (wool-tinted), lang keys.
- Recipes: the white elevator recipe and 15 dye recipes.
- Block declaration through `data/elevatorid/horizon/block/*.json` using a
  vanilla template (solid, full cube, non-conductive), one state with
  `facing` (N/E/S/W) if directional rendering is carried.
No worldgen/loot beyond block drops (self).

## Logic design (second)

- Runtime engine (pure, unit-tested): `select_target(column: &[(y, color)],
  from_y, direction, range, same_color) -> Option<y>`; nearest target in the
  travel direction within `range`; ignores invalid (blocked-above) candidates;
  colour filter per `sameColor`.
- Hook: `player_tick` (API 31) — fires for the acting player each tick.
  Reads (in-crate field access, no lifecycle read scope): player position,
  on-ground/jump state, sneak key, block under the feet.
- Detect: standing on a member of our elevator block set, and either
  `velocity.y > 0` (jump) or sneak pressed with a downward target.
- Teleport: **no direct player-teleport API**, but a viable deferred path was
  found in the pin's source: `native_run_command(line) -> bool` (API 21, console
  level 4) exists and is refused *only while a player's state is swapped in*;
  it works from `native_tick`/`native_menu_clicked`. So a `player_tick` hook can
  queue `tp <player> x y z` and `native_tick` can execute it. This is untested.
  Ruling updated: status `pending(deferred-command)` rather than blocked; the
  experiment to run is exactly that queue+execute path (plus fall damage/pitch
  handling), with a player-position hook still the cleaner long-term API.
- Config: `Game::native_config_dir()` (API 26) + a TOML file in the port's
  schema; defaults as above.

## Open questions to settle before coding

1. `native_perform_command` signature/contract at the pin, and whether a
   native mod may run it without operator permission.
2. Whether `native_player_tick` fires for the singleplayer host (server-thread
   host) as well as LAN guests, and which `Game` state is swapped in.
3. Whether the carried block state can expose `facing` (for the directional
   model) through the content-mod block templates, or whether the first slice
   ships non-directional models.
4. How block interactions (camo right-click / options GUI) map to
   `right_click_block` + `screen` hooks; deferred out of the first slice.

## Licence

MIT — derivative ports are allowed; still local-only until the maintainer
approves publishing (ARCHITECTURE.md §8).

## Upstream source

Analysed 2026-11-03. Clone: `ports/_shared/upstream/elevatorid/` (git-ignored
scratch, delete when done).

- **Repo / commit:** <https://github.com/VsnGamer/ElevatorMod>, branch
  `1.21.1/input_patch`, commit
  `129614f8e54de6ddca3c5db757fa8ebc33f40a9d` (2025-02-16, "Improve controller
  compatibility"). `gradle.properties` says `mod_version = 1.21.1-1.11.4`
  (the pack's jar). Modrinth only publishes 1.11.3 for NeoForge 1.21.1; the
  extra commits on this branch are controller/GUI input wiring, not elevator
  logic. MIT (`LICENCE`).
- **Files read:** `ElevatorHandler.java`, `network/TeleportPacket.java`,
  `Config.java`, `ElevatorBlockBase.java`, `level/ElevatorBlockEntityBase.java`,
  `ElevatorMod.java`, `network/ClientPacketSender.java`; NeoForge
  `ElevatorModNeoForge.java`, `init/Registry.java`, `ElevatorBlock.java`,
  `network/NetworkHandler.java`, `tile/ElevatorBlockEntity.java`; resources
  (16 blockstates/models, recipes, tags, sounds, lang); `neoforge.mods.toml`,
  `gradle.properties`, `README.md`. Full spec with line numbers:
  [`ports/_shared/upstream/elevatorid/FINDINGS.md`](../_shared/upstream/elevatorid/FINDINGS.md).

### Behaviour extracted (not translated)

- The mod is **client-driven**: `ElevatorModNeoForge` runs
  `ElevatorHandler.handleInput` on `ClientTickEvent.Post`; it reads
  `player.input.shiftKeyDown` / `player.input.jumping` and fires on their
  **rising edges** (`ElevatorHandler.java:17-37`), then sends
  `TeleportPacket(from,to)`; the server validates and teleports
  (`TeleportPacket.java:46-127`). No cooldown, no server-side movement code.
- Origin = first elevator found scanning **down** from the feet block for at
  most `activationRange` cells; if that elevator's cell above is suffocating,
  abort (do not scan deeper) (`ElevatorHandler.java:76-91`).
- Target = **nearest** elevator in the travel direction, same x/z column,
  within `range` (inclusive, 384), outside-build-height stops the scan;
  candidates with a suffocating block above are skipped and the scan
  continues; with `sameColor` a different dye is skipped and the scan
  continues (`ElevatorHandler.java:54-67`).
- `isValidPos` is `!above.isSuffocating(above)` (`TeleportPacket.java:133-135`)
  — "at least one non-solid block above" in the README is imprecise.
- Teleport (`TeleportPacket.java:71-93`): yaw = target's `facing` only for
  directional elevators; pitch = 0 if (`directional && resetPitchDirectional`)
  or (`!directional && resetPitchNormal`); x/z centred on the block when
  `precisionTarget`; feet at the block's support-shape top (full cube → `y+1`);
  vertical delta zeroed, horizontal kept; fall distance reset; plays
  `elevatorid:teleport` at the target.
- Config (`serverconfig`-style `elevatorid-server.toml`, `[General]`):
  `sameColor=false`, `range=384` (3..4064), `activationRange=6` (1..64),
  `precisionTarget=true`, `mobSpawn=false`, `resetPitchNormal=false`,
  `resetPitchDirectional=true`, `useXP=false`, `XPPointsAmount=1`.
- Block ids are exactly `elevatorid:elevator_<dye>` for the 16 dye names
  (`Registry.java:31-34`), incl. `light_blue`, `light_gray`, `gray`; block
  states `facing/directional/show_arrow`, defaults N/false/true; the shipped
  blockstate JSON is a single `""` variant, so facing never selects a model.
  Wool sound, strength 0.8, noOcclusion, full cube, redstone-signal source
  delegating to the camo state; all content/recipes/tags already carried.

### Pin facts settled (pin `c9d474d`)

1. **`player_tick` state.** `native_player_tick` fires only from
   `Game::tick_guests` (`lan_actions.rs:3436`), i.e. for hub **guests** — and
   in singleplayer the owner joins the integrated server as a memory guest
   (`integrated.rs:754,769`), so it fires for them too. The acting guest's
   `PlayerMotion` is swapped into `self.player` (`enter_guest`,
   `lan_actions.rs:371`): `position`, `velocity`, `ground.on_ground`,
   `history` (this tick's moves), `shift_key_down` (set from the crouch flag,
   `lan_actions.rs:3853`) and `pose` are there. `PlayerMotion::jumping` is
   **not** reliable (only local/replay ticks set it), and the guest's raw key
   bits (`Guest::input`, jump 16 / shift 32) are **not reachable from this
   hook**: `tick_lan` has taken the hub out of `self.lan` while
   `tick_guests(&mut host)` runs (`game/lan.rs:2162,2331`). So the port listens
   on the swapped body: sneak = `shift_key_down` rising edge; jump = the
   take-off predicate the server itself uses (`last_on_ground &&
   !on_ground && velocity.y > 0 && history has an upward move`), which
   `jump_from_ground`/`history` make observable (`lan_actions.rs:3820-3830`).
   Exact key-edge semantics would need an API exposing the input flags (gap).
2. **`server_tick` on singleplayer.** Yes: `native_lifecycle_authority` maps
   `ExecutionRole::IntegratedServer` to the integrated authority
   (`native_lifecycle.rs:218-230`) and both phases call the hook
   (`:264-276`). `native_run_command` is usable there — the guard is only
   `if self.lan_acting.is_some() || self.lan.is_none() { return false; }`
   (`native_server.rs:480-483`), and the integrated server always creates a
   hub (`integrated.rs:754` → `game/lan.rs:1416`, `HubConfig { dedicated:
   true, integrated: true }` at `:715`; the hub is back in `self.lan` between
   level ticks). `native_server_block(context, dimension, pos)`
   (`native_world_read.rs:95`) is in scope for the column reads. Not usable:
   inside `player_tick` (state swapped in), and the deprecated `--in-frame`
   path (no hub).
3. **Acting player identity.** In `player_tick`,
   `native_acting_player()` (`native_server.rs:459-478`) gives `uuid: u128`,
   `name`, `dimension`, `position`, `game_mode`, `health`, `yaw`, `pitch`
   (built from `lan_acting_uuid/_name`, `self.dim.id()`). Queue **key = uuid**;
   the executor re-reads fresh name/dimension/position by uuid from
   `native_server_players()` (`native_server.rs:257-278`) and runs
   `execute in <dimension> run tp <uuid> <x> <y> <z>` through
   `native_run_command` on `ServerTickEnd` (command feedback swallowed with
   `chat_capture`).

### Differences from the rewrite (declared)

- No direct player-position write at the pin → deferred `/tp` (one tick of
  latency, all velocity zeroed, console level 4, sound deferred because
  `native_play_sound` is on `GameLevel`, not reachable from `server_tick`).
- The acting player's key bits are not exposed to `player_tick` (the hub is
  detached from `self.lan` while the hook runs), so the rewrite infers the jump
  from the swapped body; gap: "`native_player_tick` should receive the player's
  input flags (jump/shift) and `on_ground`", which would give the exact
  one-teleport-per-press edge upstream has.
- `isSuffocating(above)` approximated with `facts.full_collision` (air + full
  cubes), a corner-case difference for glass/leaves.
- Held jump can auto-jump on landing at the target and climb again (upstream
  is one teleport per press); a short per-uuid cooldown is the planned
  mitigation.
- Camo, options GUI, directional yaw/arrow, tinted particles, XP cost, mob
  spawning and the teleport sound are deferred; all our elevators are
  non-directional full cubes, so yaw/pitch are kept and feet land at `y+1`.
- Content already carried: 16 blocks/items, 32 recipes, tags (`c:` dye tags
  included), sounds, lang; the scaffold's mixin still targets the template pin
  and `example_mod` and must be rewritten to `c9d474d` with the `player_tick` +
  `server_tick` injections.

## Fix applied after the first in-game run

The first run failed only the ceiling-clipped jump: `native_player_tick` fires
after movement, so `velocity.y` is post-collision (a jump blocked by a low
ceiling reports a fall). The port now uses the **server's own take-off
predicate** (`lan_actions.rs`): the player left the ground on an upward move
this tick, read from `PlayerMotion::history` (`records()`; `to.y - from.y >
0.0`). Retested after the fix.

## Verification results (in-game, final)

`tools/e2e.sh` on the rebuilt fast binary (The End, `--dev-mod content`):
jump **64→67**, sneak **67→64**, plain-floor jump/sneak stay **64**, lone
elevator jump stays **64**, stone above the middle elevator makes the jump
skip to **70**. Exit 0, `PASS: jump 67, sneak 64, plain floor stayed, lone
elevator stayed, blocked target skipped to 70`.

The deferred path was proven first with a manual run: `ctl jump` replied
`{"from_y":64.0,"position":[0.5,67.0,0.5]}` — `player_tick` queued,
`server_tick` ran `execute in minecraft:the_end run tp <uuid> 0.5 67 0.5`.

## Earlier results (superseded)

Run of 2026-11-03 at pin `c9d474dd9116e84c243f8992d75c11314df56f06`
(`horizon-mixins` at its pinned revision), The End, `--dev-mod content`.
Commands: `python3 tools/horizon.py build --horizon <campaign>/horizon`
(exit 0), `cargo build -p horizon-client --bin horizon-ctl` in
`target/mixins/modded` (exit 0), `timeout 600 tools/e2e.sh` (exit 1).

Log: `/tmp/opencode/elevatorid-e2e.log`. e2e.sh was **not** modified.

Observed feet positions (`ctl state` → `.player.position[1]`):

| Step | Expected | Observed |
|---|---|---|
| jump from 64, elevators 63/66/69 | 67 | **67.0** |
| sneak down from 67 | 64 | **64.0** |
| plain floor jump / sneak | 64 / 64 | **64.0 / 64.0** |
| lone elevator (63 only) jump | 64 | **64.0** |
| stone at 67, jump from 64 (skip 66 → 69) | 70 | **64.0 → FAIL** |

```
after jump: y=67.0
after sneak: y=64.0
plain-floor jump: y=64.0
plain-floor sneak: y=64.0
lone-elevator jump: y=64.0
blocked-target jump: y=64.0
FAIL: expected y=70 (skipping the blocked 66), got 64.0
```

### Port bug found: jump-up is not detected under the usual 2-block clearance

The jump is inferred from the body, but `native_player_tick` fires at the
**end** of `ServerPlayer.tick`, after that tick's movement, so
`motion.velocity.y` has already been zeroed by the ceiling collision
(`mixins/elevatorid/support.rs:55-64`: `jump = !ground && velocity.y > 0.0`).
With the next elevator at `feet+2` the head has only 0.2 of clearance, the
jump is clipped at `feet+0.2`, and the hook sees `vy = -0.0784`:

```
player ground=false vy=-0.0784 y=64.200 pose=Standing h=1.80 jump_edge=false
```

The first jump only travels by accident: the freshly teleported body is still
in the `Swimming` pose (h=0.60), so the jump is not clipped and the hook sees
`vy=+0.3332`:

```
player ground=false vy=0.3332 y=64.420 pose=Swimming h=0.60 jump_edge=true
server ... up=true feet=64.420 target=67 line=execute in ... tp ... 0.5 67 0.5 ok=true
```

That is why the first assertion passes and the obstruction one fails: it is
the *second* jump, in the normal standing pose. Positive control (elevators at
63 and 69 only, so the jump is not clipped) detects the take-off and lands at
70 — the deferred teleport itself works (`native_run_command` → ok, one tick
later, exactly as designed). The sneak path is unaffected (key-edge based).

Evidence: temporary `eprintln`-to-file tracing added **only** to the generated
`target/mixins/modded/crates/horizon-client/src/game.rs` (build output), then
restored (`diff` clean, `horizon.py stage` reports `stage_changed: false`). No
mixins or content were edited. `ctl logs --lines 120` shows no command
errors and no warning; the mod never issues a `/tp` for the failing jump.

Suggested fix (not applied here; mixins are read-only for this run): detect
the take-off with an upward move in `motion.history` this tick (the design note
already listed that term) instead of the post-collision `velocity.y`, or expose
the jump input flag to `player_tick`. Until then jump-up works only when the
cell above the head has ≥ ~0.4 free (i.e. no elevator 2 above), which is not
the normal elevator column.

## Re-pin to `83ad384f` + the jump-edge fix (2026-11-05/06)

**Class/tier:** content / **logic** (`ports/ledger.csv`; approach per
`horizon/docs/modpacks.md` §Tiers: content + behaviour hooks, not a data-only
carry).

**Migrated:** Horizon `83ad384f48d0742746d518737511882680aab5b7` (427 hooks /
159 helpers), horizon-mixins `e8cd0aa38cffef49cb2e088ef716baa107148c02`.
`compatibility.json`, the mixin manifest's `[compatibility]`, and
`tools/horizon.py` (embedded tests now `--lib`) updated; the mixins pin is
copied to `target/horizon-mixins` for the CLI.

**Feature (the recorded failing gap):** the jump trigger no longer infers a
take-off from the post-collision body. Two stamped hooks do it directly:

| Hook | API | Use |
|---|---|---|
| `player_jump` | 344 | accepted movement jumped the server player from the ground, after simulation: the exact up edge, even when a low ceiling clips the jump |
| `player_toggle_sneak` | 351 | the guest's sneak flag changed; only the press queues a down travel |

The body predicate in `player_tick` and the window's Space path remain as
fallbacks; the queue is a UUID-keyed map, so a duplicate edge is one entry.

**Suffocation:** `ServerColumn::suffocating` now asks
`native_block_collision` for the cell's real collision boxes when the player's
dimension is the level in place (the helper reads that level); otherwise it
keeps the content-facts (`full_collision`) fallback. A cell is suffocating when
a box covers the whole block.

**e2e:** the blocked-target phase (stone above the middle elevator, expect
feet 70) was the assertion that failed at the old pin (observed 64, the clipped
jump). `SELFTEST=1` keeps the old 64 expectation and must fail, proving the
scenario is fail-capable; the plain-floor and lone-elevator controls stay.

Results are appended below after the run.

### Results on the new pin (2026-11-06)

| Step | Command | Result |
|---|---|---|
| unit tests | `cargo test --locked --offline` (runtime lib) | 10 passed, 0 failed |
| staged check | `python3 tools/horizon.py check --horizon …/horizon-83ad384f` | `cargo_exit: 0`, **0 warnings** (`target/horizon-check.log`) |
| staged test | `python3 tools/horizon.py test …` | `cargo_exit: 0`, **10 passed; 0 failed; 913 filtered out**, 0 warnings (`target/horizon-test.log`, 15:06) |
| fast build | `python3 tools/horizon.py build …` | `cargo_exit: 0`, 0 warnings, `target/horizon-cargo/fast/horizon-client` (96,191,832 B, 17:09) |
| e2e default | `tools/e2e.sh` | **exit 0**: `PASS: jump 67, sneak 64, plain floor stayed, lone elevator stayed, blocked target skipped to 70`; observed `blocked-target jump: y=70.0` (the old failing assertion) — `/tmp/opencode/elevatorid-e2e-newpin.log` |
| e2e counterfactual | `SELFTEST=1 tools/e2e.sh` | **exit 1, as required**: `FAIL: expected y=64 (skipping the blocked 66), got 70.0` — `/tmp/opencode/elevatorid-e2e-selftest.log` |

The `SELFTEST=1` run keeps the old body-predicate expectation (64) against the
same scenario and fails on the climbed body (70), so the blocked-target
assertion is fail-capable, not vacuous.

Artifact: `content/elevatorid.hmod` 119,915 B (content unchanged by the
jump-edge fix; the feature lives in the native bundle). Remaining gaps are
unchanged from the list above (direct player-position write, guest raw input
flags, camouflage/options GUI/directional model/XP/mob spawn/sound).

## Re-pin to `1d63e0a` + the native player teleport (2026-11-08)

**Class/tier:** content / **logic** (unchanged; `ports/ledger.csv`). Licence
MIT, rank 0.

**Migrated:** Horizon `1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9` (432 stamped
hooks / 174 helpers; the catalog is byte-identical to `41f822b6`) +
horizon-mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57` (schema 6).
`compatibility.json` now carries `game.rs` sha256 `64813cf4…` (the `83ad384f`
hash was `996432b9…`) and the bundle manifest's `[compatibility]
horizon_revision` moved with it. Every existing hook resolves: `tick` →
`native_tick`, `server_tick` → `native_server_tick`, `player_tick` →
`native_player_tick`, `player_jump` → `native_player_jump`,
`player_toggle_sneak` → `native_player_toggle_sneak`
(`…/slot-2/stage/plan.json`).

### Transport: old → new

| | slice 1 (`83ad384f`) | this slice (`1d63e0a`) |
|---|---|---|
| move | `execute in <dim> run tp <uuid> <x> <y> <z>` via `native_run_command` (console level 4) | `native_teleport_player(uuid, dimension, pos, yaw, pitch)` |
| where | `server_tick` HEAD, queue drained at `ServerTickEnd` | same hook; the helper is immediate there (no player's state is swapped in) |
| look | left as-is (no rotation arguments) | the player's current `yaw`/`pitch` from `native_server_players()`, so the travel does not turn them |
| destination | `bx+0.5, target.feet_y, bz+0.5` built inline | `travel_position(bx, bz, target.feet_y)` (new pure function + unit test) |

The scan itself (origin/activation range, nearest target, `RANGE` 384,
`suffocating` above, `sameColor`, precision x/z) is unchanged.

### The helper and its floor

`tools/native-hook-fragments/teleport_player.json` declares the helper
`teleport_player` →
`horizon_client::game::native_travel::Game<'_>::native_teleport_player`
(`crates/horizon-client/src/game/native_travel.rs:55`, `pub(super)`, so an
injected child of `game` may call it). Its `doc_summary` says, verbatim:

> `native_teleport_player(uuid, dimension, pos, yaw, pitch)` teleports a server
> player as `/tp` does, from any server hook … Bundles need
> `"hooks": @API:teleport_player@`.

`docs/native-mods.md` ("Server hooks for plugin ports: Moving players and
vehicles") adds the part this port relies on: "Callable from any server hook.
Within their level it is done at once when they are the acting player or
nobody's state is swapped in (a tick hook)".

**Floor/marker.** `ports/_shared/repin-1d63e0a-smoke.md` §2/§5 proved a bundle
may **call** a pending helper while it may not **name** the pending hook: the
build-time generator unions `tools/native-hook-fragments/*.json`
(`horizon-mod-api-gen/src/lib.rs:1014-1035`), while the mixins planner resolves
names only against `native-hooks.json`
(`horizon-mixins-core/src/native.rs:120-138`). The marker
`@API:teleport_player@` is **unresolved**; the stamp replay (first-parent merge
order) projects **555** for this fragment (515 is its sibling `player_teleport`,
which this port does not use). The pin's `NATIVE_HOOKS_API` is **411**
(`game/native_hooks.rs:1170`), so `"hooks": 555` would be refused by a launcher
at this pin (`launcher/src/native_mods.rs`, `b.hooks <= found`) and `411` is
true of the pin but does not express the pending helper. `tools/package.py`
therefore still omits the optional `hooks` field
(`crates/horizon-mods/src/index/native.rs:43-48`): the exact pin in the manifest
is the contract until upstream stamps the fragment.

**Latency.** Unchanged, read from the tick shape rather than assumed:
`Game::tick_server` runs the level pass (`tick_lan → tick_guests →
native_player_tick` / `player_jump` / `player_toggle_sneak`) and only then
`native_lifecycle_server_tick_end` → our `server_tick`
(`crates/horizon-client/src/game/levels.rs:486-588`), so the queue is drained
in the same server tick it was filled in — as the `/tp` command was. What
changed is the transport: no console command, no level-4 permission, no command
feedback, and the documented server API instead of an operator command. The
helper still "lose[s] their motion" as `/tp` does, so upstream's horizontal
velocity on arrival is still not preserved (gap, unchanged).

### Results on the new pin (2026-11-08)

## Embedded content — one `.hnmod` deliverable (2026-11-10)

The content half now travels **inside** the native bundle (the AE2 pattern, proved
on `charginggadgets`): the built `content/elevatorid.hmod` is carried at
`mixins/elevatorid/data/elevatorid-content.hmod`; `embedded_content.rs` writes it to
`.horizon-cache/` at start and appends it to the dev list from one `content_root`
HEAD injection (`cancellable = true`; bindings `root`, `enabled`, `dev`). One
installed `.hnmod` installs both halves, so the split content+code blocker is
gone. `tools/package.py` packs `data/` and refuses a copy that differs from
`content/elevatorid.hmod`.

Verification (2026-11-10, pin `1d63e0a`, Minecraft 26.3):
- **ladder**: `cargo fmt --check`, `clippy --all-targets -- -D warnings` (0
  warnings), `cargo test` **10 passed / 0 failed**, `horizon.py stage / check / test /
  build` all `cargo_exit: 0` (embedded tests included).
- **artifacts**: `content/elevatorid.hmod` **119,915 B**, SHA-256 `b16e3a4b1bb7d943e4ff11635e5e3604c498fcd7d22fb692ae9c1fcf08142e81`,
  **188** entries; `elevatorid.hnmod` **67,006 B**, SHA-256 `d6301c96967bec7e8c397b64a6ce1160854183d0080891bfede15114f6bf6ddd`, **11**
  entries (rebuilt locally from these sources with `tools/package.py`; a second
  run is byte-identical and `horizon-mod check --kind native-mod` passes);
  `mixins/elevatorid/data/elevatorid-content.hmod` is byte-identical to the built `.hmod`.
- **e2e with no `--dev-mod`** (`EMBEDDED=1`): `PASS` + `EXIT:0`; the run printed
  `embedded content: 119915 B, identical to content/elevatorid.hmod`.
- **control** (`e2e-control.log`, `.horizon-cache` made unusable): the sample id
  is unknown, `PASS` + `EXIT:0` - the content really comes from the bundle.
- **fail leg** (`SELFTEST=1`): `FAIL` + `EXIT:1`.
- `check-e2e-logs.py elevatorid --strict` green (all logs end `EXIT:<n>`);
  `upstream_ready.py elevatorid` -> `READY`, exit 0.

`tools/horizon.py` defaulted `--horizon` to the stale `ports/horizon`; it now defaults to the pin checkout. The compile ladder ran on the second build machine.

Skills: superpowers/using-superpowers,systematic-debugging,test-driven-development,verification-before-completion; BeautifulRust/rust-best-practices

| Step | Command (through `build-slot.sh`) | Result |
|---|---|---|
| unit red | `cargo test` on the port lib, before `travel_position` existed | failed as required: `E0425: cannot find function travel_position` |
| unit green | `cargo test --locked --offline` (port lib) | **10 passed; 0 failed** |
| clippy | `cargo clippy --all-targets --locked --offline -- -D warnings` | exit 0, **0 warnings** |
| stage | `tools/horizon.py stage --horizon …/horizon-1d63e0a` | exit 0, 86 s, `stage_changed: true` |
| fetch | `cargo fetch --locked` on the staged workspace | exit 0 (pin's new lock packages) |
| check | `tools/horizon.py check …` | `cargo_exit: 0`, **0 warnings** (`_shared/evidence/elevatorid/horizon-check.log`) |
| test | `tools/horizon.py test …` | `cargo_exit: 0`, **10 passed; 0 failed; 1240 filtered out**, 0 warnings (`_shared/evidence/elevatorid/horizon-test.log`) |
| build | `tools/horizon.py build …` | `cargo_exit: 0`, 0 warnings, 6m55s, `fast/horizon-client` **110,382,464 B** sha256 `7adfa4f5…aca1` (ladder JSON in `_shared/evidence/elevatorid/ladder.log`) |
| e2e | `build-slot.sh e2e … -- tools/e2e.sh` (shared ctl at this pin) | **exit 0**, `PASS: jump 67, sneak 64, plain floor stayed, lone elevator stayed, blocked target skipped to 70` — `_shared/evidence/elevatorid/e2e.log` |
| e2e negative | same with `SELFTEST=1` | **exit 1, as required**: `FAIL: expected y=64 (skipping the blocked 66), got 70.0` — `_shared/evidence/elevatorid/e2e-selftest.log` |

The e2e transcript for the travel (The End, `--dev-mod content`); the y values
are `ctl state .player.position[1]`, the first JSON on each line is the
`ctl jump` reply:

```
jump       {"from_y":64.0,"ok":true,"position":[0.5,67.0,0.5]}   after jump: y=67.0
sneak      after sneak: y=64.0
plain floor{"from_y":64.0,"ok":true,"position":[8.5,64.0,0.5]}   plain-floor jump: y=64.0
           plain-floor sneak: y=64.0
lone       {"from_y":64.0,"ok":true,"position":[6.5,64.0,0.5]}   lone-elevator jump: y=64.0
blocked    {"from_y":64.0,"ok":true,"position":[0.5,70.0,0.5]}   blocked-target jump: y=70.0
```

The plain-floor control is the **differential** (player on stone, no elevator:
no travel), and `SELFTEST=1` is the **negative** (the assertion set can fail).
The build was produced from these sources in the same slot invocation, and the
staged module contains `native_teleport_player` with no `native_run_command`
(`stage/modded/crates/horizon-client/src/game.rs:24398-24401`), so the transcript
is the new transport.

**Artifacts.** `.hnmod` `target/elevatorid-0.1.0.hnmod` **9,141 B** sha256
`8eca8389…90cf`; the content carry is unchanged, `content/elevatorid.hmod`
**119,915 B** sha256 `b16e3a4b…2e81`.

**Remaining gaps** (unchanged): horizontal motion on arrival is lost (the
helper `/tp`s as a command does); no teleport sound/particles; camouflage,
options GUI, directional model/yaw, XP cost, mob spawn, config file; the
`hooks` floor waits on the upstream stamp.

### Mode B + AE2 embedding (2026-11-10) — one listable artifact, re-verified

Pin unchanged (`1d63e0a5…` / `5a64af93…`, schema 6, `game.rs`
`64813cf4…f80f`). The session rebuilt the port from its current sources and
applied the **AE2 embedding pattern** so one `.hnmod` installs the content and
the code (the registry keeps one file per project): `mixins/elevatorid/data/elevatorid-content.hmod`
is the built `content/elevatorid.hmod` byte for byte, `mixins/elevatorid/embedded_content.rs`
compiles it in with `native_data!`, and the `embedded-content` `[[injection]]`
(`target = "horizon_client::game::mods::content_root"`, `kind = "HEAD"`,
`cancellable = true`, bindings `root`/`enabled`/`dev`) writes it to the game's
`.horizon-cache` and appends it to the dev-mod list. `tools/package.py` refuses
an embedded copy that differs from the built hmod.

All compile/e2e steps ran through `ports/tools/build-slot.sh` with
`CARGO_INCREMENTAL=0`; the ladder is the shared `tools/ladder-embed.sh`
(hmod build → embed → hnmod → fmt → clippy → test → stage/check/test/build),
evidence `ports/_shared/evidence/elevatorid/ladder-*`.

| Step | Result |
|---|---|
| `cargo fmt --all --check` / clippy `-D warnings` | exit 0, **0 warnings** |
| `cargo test --locked --offline` | **10 passed**, 0 failed |
| `horizon.py stage/check/test/build` | `cargo_exit: 0` in every step |
| `content/elevatorid.hmod` | **119,915 B**, `b16e3a4b…2e81`, **188 entries**, "no code: content only" |
| `elevatorid.hnmod` (single artifact, carries the content) | **67,006 B**, `d6301c96…6ddd`, **11 entries** |
| e2e (with `EMBEDDED=1`, the shipped shape) | default **PASS + `EXIT:0`** / selftest **FAIL + `EXIT:1`** |
| `check-e2e-logs.py elevatorid --strict` | green (`logs=… ok=…`, exit 0) |
| `upstream_ready.py --copyleft elevatorid` | **READY**, exit 0 |

Fail-capability: the selftest/negative legs flip a real assertion and exit
non-zero; the embedded shape is the one the artifact ships (no `--dev-mod`), and
the e2e checks the bundle wrote its content out (or the loader reports
`content from 1 mod(s)` when the game consumes the file).

## Review repairs (PR #612)

- Staging is the orchestrator's (horizon-mods#686).
- *"Preserve the carried assets' MIT notices in the distributed archive … the
  repository LICENSE credits Horizon contributors and omits original upstream
  `Copyright (c) 2016 Vasco`"* — the repository `LICENSE` now names
  `Copyright (c) 2016 Vasco (ElevatorMod)`, and `mixins/elevatorid/LICENSE` +
  `NOTICE.md` travel inside the `.hnmod` (and at the embedded content root).
- *"Repair the submitted audit evidence tooling: `tools/test_stage.py:30` and
  `tools/test_template.py:62` still expect the old example … the audit also
  requires every injection to be non-cancellable HEAD while embedded-content is
  cancellable"* — `test_stage.py` now compares the plan's counts with the
  counts this bundle's own `horizon-mixin.toml` declares; `test_template.py`
  states it is a synthetic one-hook fixture (the real bundle's audit is
  `test_stage.py`); `tools/audit/src/main.rs` accepts a cancellable HEAD (the
  embedded-content pattern) instead of rejecting it.
- *"Fix `mods/elevatorid.toml:6` from relative `VsnGamer/ElevatorMod` … and
  repair the wrapped-line bullets/incomplete 'see above' text"* — the entry
  links the absolute URL, credits Vasco, and the description is rewritten with
  complete prose.
- The provenance hash in the PR body (`f0a1f0…`) is the orchestrator's to
  correct.

### Verification after the repairs

- Artifact: `elevatorid.hnmod`, 69,688 B, SHA-256 `dc8acebf693729b5c32f68facc59dd24427d4912053b83a25f4bc2e4fd9a5495` (rebuilt twice, identical bytes; `tools/package.py`; the embedded content equals `content/elevatorid.hmod`).
- `python3 ports/tools/upstream_ready.py elevatorid` and `check-skills.py --notes ports/elevatorid`: exit 0.

---

## Migration to the SDK format (0.2.0, 2026-11-18)

Horizon's mod-API merge (#16941, launcher ≥ 0.2.70, PR #17992) retired the `.hnmod` container: the
registry closes every `.hnmod` submission with one reason and an `.hnmod` no longer loads. This is
the port's **first publication** in the new unit — **one flat `.hmod`** carrying content and code
(`horizon-mod.toml` + `Cargo.toml` + `src/` + `assets/` + `data/`), version **0.2.0**, `api = "0.2"`.
There is no closed PR to supersede; the retired bundle was the campaign's 0.1.0 and stays as
evidence only.

Pins: SDK nightly **`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (2026-11-18, `package_format` 6,
`content_api` "0.2"), mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`** (it moved from
`5a64af93…`), game **26.3**. The SDK is cached in-tree at `../_shared/sdk/0b14f8a/` and copied to
`.horizon/sdk/` (gitignored) for offline builds; every `cargo horizon-mod` command re-fetches it
when absent.

### What moved, and where the retired tree is

The retired paths were **moved before the first `cargo horizon-mod build`** (the tool refuses a
root holding them) into
[`../_shared/evidence/elevatorid/retired-hnmod-0.1.0/`](../_shared/evidence/elevatorid/retired-hnmod-0.1.0/).

| 0.1.0 path | 0.2.0 path | Evidence |
|---|---|---|
| `mixins/elevatorid/runtime.rs` | `src/runtime.rs`, **copied with `cp`, `cmp`-checked** | `retired-hnmod-0.1.0/mixins/elevatorid/runtime.rs` |
| `mixins/elevatorid/support.rs` + 6 hook helpers (`jump.rs`, `sneak.rs`, `player.rs`, `tick.rs`, `server.rs`, `embedded_content.rs`) | `src/support.rs` (the queue, the column reads, the teleport) + `src/lib.rs` (the one `Hooks`) | `retired-hnmod-0.1.0/mixins/elevatorid/` |
| `mixins/elevatorid/horizon-mixin.toml` (6 `[[injection]]`, 2 `[[module]]`) | no plan: typed `Hooks` methods and the manifest | `retired-hnmod-0.1.0/mixins/` |
| `mixins/elevatorid/data/elevatorid-content.hmod` + `embedded_content.rs` (AE2 pattern) | gone: the package carries `assets/` and `data/` itself | `retired-hnmod-0.1.0/mixins/` |
| `content/` (`horizon-mod.toml`, `assets/`, `data/`, `elevatorid.hmod`, `README.md`) | the port root: `horizon-mod.toml`, `assets/`, `data/` | `retired-hnmod-0.1.0/content/` |
| `content/elevatorid.hmod` (190 entries, content only) | not needed: content is inside the one package | `retired-hnmod-0.1.0/content/elevatorid.hmod` |
| `elevatorid.hnmod` (69,688 B, embeds the content) | `elevatorid.hmod` (134,756 B, native + content) | `retired-hnmod-0.1.0/elevatorid.hnmod` |
| `compatibility.json`, `rust-toolchain.toml` | `[native]` in `horizon-mod.toml`; the pinned commit decides the toolchain | `retired-hnmod-0.1.0/` |
| `tools/horizon.py`, `tools/package.py`, `tools/test_stage.py`, `tools/test_template.py`, `tools/add-notices.py`, `tools/ladder-embed.sh`, `tools/audit/`, `.github/workflows/`, the old `Cargo.toml`, `horizon-player.txt` | deleted: staging, `.hnmod` packaging and the plugin-injection audit belong to the retired format | `retired-hnmod-0.1.0/tools/`, `retired-hnmod-0.1.0/github/`, `retired-hnmod-0.1.0/Cargo.toml.0.1.0` |
| `Cargo.lock` | not kept: it is packed and changes the artifact (trap 3) | not kept |

**Kept and rewritten:** `README.md` (it was still the untouched template README and the package
packs it), `AGENTS.md`, `LICENSE-NOTICE.md` (it named the wrong upstream version and claimed a
notice inside `mixins/`, which no longer exists), `tools/e2e.sh`, `notes.md`, `VALIDATION.md`,
`REPIN-AUDIT.md`, `LICENSE`. **Carried unchanged:** the 187 files under `assets/` and `data/`
(size **and** sha256 compared with the retired built content archive: no added, removed or changed
file).

### File map (0.2.0)

| Path | What it is |
|---|---|
| `horizon-mod.toml` | the only manifest and identity: `id = "elevatorid"`, `version = "0.2.0"`, `api = "0.2"`, `side = "both"`, `capabilities = ["registries"]`, `[native]` game/horizon/mixins |
| `Cargo.toml` | one crate, `edition = "2024"`; its only dependency is `horizon-mod-api` from `.horizon/sdk/` |
| `src/lib.rs` | the one `#[mod_api::horizon_mod] impl Hooks`: three methods, each one line into `support` |
| `src/support.rs` | the glue: `Travel` (queue + cooldown, 5 unit tests), the column reads, the teleport |
| `src/runtime.rs` | the pure planner, **byte-identical**, 10 tests |
| `assets/`, `data/` | the carried content: 16 blocks, items, models, textures, sounds, 19 lang files, 32 recipes, 16 loot tables, 3 tags, creative tab |
| `LICENSE`, `LICENSE-NOTICE.md` | MIT text with `Copyright (c) 2016 Vasco (ElevatorMod)` and provenance, **inside** the package |
| `tools/e2e.sh` | the in-game scenario, now for a launcher-built game |
| `elevatorid.hmod` | the deliverable |

### Helper → hook table

Every old `[[injection]]` and `[[module]]` has a home; nothing was silently deleted.

| 0.1.0 plan entry | 0.2.0 | Mapping notes |
|---|---|---|
| `embedded-content` (`content_root` HEAD, cancellable) | **gone** | the flat `.hmod` carries `assets/`/`data/` in itself; nothing to write out |
| `[[module]] runtime` (`game::elevatorid`) | `src/runtime.rs`, `pub mod runtime;` | same file, byte for byte; a private module would warn every `pub` item |
| `[[module]] state` (`elevatorid_state`) | `src/support.rs` + `Mod { travel: Mutex<Travel> }` | the old `static Mutex<Option<State>>` became the mod instance's field; `Hooks: Any + Send + Sync` ⇒ `Mutex`, never `RefCell` |
| `player-tick` (`player_tick` 31) → `player.rs` | **dropped**, superseded | it was the body-predicate fallback (`velocity.y` / upward `history`) for a jump edge the pin could not see before API 344; `player_jump` reports that edge exactly (see deviations) |
| `player-jump` (`player_jump` 344) → `jump.rs` | `Hooks::player_jump(&self, host, &mut PlayerJumpEvent { player, name })` → `support::on_jump` | `event.player` is the UUID key; the spectator check moved from `native_acting_player().game_mode == 3` to `player.game_mode(host) == Some(GameMode::Spectator)` |
| `player-sneak` (`player_toggle_sneak` 351) → `sneak.rs` | `Hooks::player_toggle_sneak(…, &mut PlayerToggleSneakEvent { player, name, sneaking })` → `support::on_sneak` | only `sneaking == true` queues, as before |
| `server-tick` (`server_tick` 4) → `server.rs` | `Hooks::server_tick(&self, host, &mut ServerTickEvent { context, phase })` → `support::on_server_tick` | `phase` matched exhaustively (`ServerTickEnd { world_running }` only); the tick for the cooldown is now `context.tick.get()` (`TickId` has an accessor, the old `NativeTickId` did not, so the port counted its own) |
| `tick` (`tick` 4) → `tick.rs` | **dropped**, superseded | the window's raw Space edge was the client-side fallback for the same jump edge; the server reports it now, and a client-only path could not be tested here |

Host calls, old → new: `game.native_server_block(context, dim, pos)` → the player's `LevelRef`
handle (`level.block(host, pos)` + `BlockKey::id(host)`); `game.native_block_collision(pos)` and
the content-facts fallback → `level.collision_boxes(host, cell)`; `game.native_server_players()` →
`host.players().server_players()` / `host.player(uuid)`; `game.native_teleport_player(uuid, dim,
pos, yaw, pitch)` → `EntityRef::teleport_to(host, pos, Some(rotation))`; `game.player.<motion>`
(in `player_tick`) → none, the dropped fallback.

### Manifest field mapping

| Old | New |
|---|---|
| `[mod] id = "elevatorid"` | `id = "elevatorid"` (also the resource namespace) |
| `[mod] version = "0.1.0"` | `version = "0.2.0"` — a new version, as every resubmission must be |
| `[mod] priority = 100` | gone; ordering is `#[listener(priority = …)]`, which this slice does not need |
| `[compatibility] horizon_revision = "1d63e0a…"` | `[native] horizon = "0b14f8a…"` (exact 40 hex) |
| the bundle's mixins revision (`5a64af93…`) | `[native] mixins = "e59da90…"` (it moved) |
| — | `[native] game = "26.3"`, `api = "0.2"`, `side = "both"`, `capabilities = ["registries"]` |

### Deviations (deliberate, not gaps)

1. **The two fallback edges are gone.** `player_tick`'s body predicate and the window's Space edge
   existed because the jump edge was not observable when the port was written. `player_jump` is the
   server's own accepted-jump report and covers the integrated host (which joins its server as a
   guest), so the fallbacks are redundant — and the body form was the one that *failed* the
   blocked-target case in 0.1.0 (a clipped jump reads a fall). Keeping it would re-add a known
   false negative plus a ledge/piston false positive.
2. **Suffocation now means "a collision box covers the whole cell" from the level's own shapes**
   (`level.collision_boxes`) instead of the content facts' `full_collision && !air`. That is the
   same rule the retired bundle used when the player's dimension was the level in place, which is
   the only case now: the player's own level is being ticked. Glass/leaves corner cases still
   differ from Java's `isSuffocating` in shape-derived blocks.
3. **The cooldown tick comes from the context**, not a self-counted tick (`TickId::get()`); same
   ten-tick window, one less piece of state.
4. **`player_toggle_sneak`'s host is still safe for the queue**: the old port never touched the
   world from a player hook and neither does this one — only the tick hook reads blocks and moves
   the player.

### Build and verify (measured 2026-11-18)

Every command ran in a `ports/tools/build-slot.sh` lease with `CARGO_INCREMENTAL=0` and no
`--target-dir`. The compile steps forwarded to the second machine; the **package** steps ran with
`BUILD_LOCAL=1` because `cargo-horizon-mod` is not installed on that box (trap 1).

| Step | Command | Result |
|---|---|---|
| runtime only, before any glue | `cargo test --offline` | **10 passed**, 0 failed — `evidence/elevatorid/sdk-test-runtime-only.log` |
| glue red | `cargo test --offline` with the four `Travel` methods stubbed `todo!()` | **5 failed / 10 passed**, all five `not yet implemented` (queue/ready/record) — `sdk-test-glue-red.log` |
| tests | `cargo test --offline` | **15 passed**, 0 failed (10 `runtime::`, 5 `support::`) — `sdk-test.log` |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** — `sdk-clippy.log` |
| format | `cargo fmt --check` (**not** `--all`: that checks the fetched SDK) | exit 0, no diff — `sdk-fmt.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | `SDK 0b14f8a … in ./.horizon/sdk` / `Built ./target/horizon-mod/elevatorid.hmod` — `sdk-build.log` |
| reproducible | a second `rm -f Cargo.lock && cargo horizon-mod build`, then `cmp` | **identical bytes**, same sha256 — `sdk-build-second.log`, `sdk-reproducible.log` |
| registry check | `cargo horizon-mod check elevatorid.hmod` | `elevatorid 0.2.0 (native-mod, format native-1/0b14f8acfd97fa2c750e42795249dcc981178ca4/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)` / `134756 bytes, SHA-256 4cc6fb2d…`, **EXIT 0** — `sdk-check.log` |
| machine form | `cargo horizon-mod check elevatorid.hmod --json` | `{"ok":true,…,"kind":"native-mod","size":134756,"version":"0.2.0"}` — `sdk-check-json.log` |
| fits the pinned build | `cargo horizon-mod fits elevatorid.hmod --build nightly` | `ElevatorMod:` / `0b14f8a  made for`, **EXIT 0** — `sdk-fits.log` |
| entries | `unzip -l elevatorid.hmod` | **196 entries**, **134,756 B**, sha256 `4cc6fb2d5edf756349209544b43dfe977e4a795a9b0b865da69580153f499c4b` — `sdk-entries.log`, `sdk-sha256.txt` |
| `runtime.rs` untouched | `unzip -p elevatorid.hmod src/runtime.rs \| sha256sum` | `ee6aa4ed…3025` = `src/runtime.rs` = the retired file, byte for byte — `sdk-runtime-in-package.sha256` |
| what it uses | `unzip -p elevatorid.hmod uses.json` | 92 entries: 12 hook entries (`player_jump` 3, `player_toggle_sneak` 4, `server_tick` 3, the content `blocks` declaration 2), 7 methods, 5 traits, 23 handles, 41 types, 2 Parts, `capability:registries`, `macro:horizon_mod` — `sdk-uses.json` |

The 196 entries are the 5 root documents (`horizon-mod.toml`, `Cargo.toml`, `LICENSE`,
`LICENSE-NOTICE.md`, `README.md`), the 187 carried `assets/`+`data/` files, the three `src/` files
and the generated `uses.json`. **No `Cargo.lock`**, no `target/`, no plan files.

### Gaps

1. **No in-game run of 0.2.0.** A native mod is compiled into `horizon-client` by the launcher
   (`horizon-launcher native-build … --trust-native-code`), which is not installed here, and no
   game exists at SDK `0b14f8a`; there is also no published shared `horizon-ctl` at this pin.
   `tools/e2e.sh` is updated for that build (`GAME=`, `CONTROL_GAME=`, `SELFTEST=1`) and has not
   been run. The 0.1.0 logs under `../_shared/evidence/elevatorid/` (`e2e.log`, `e2e-selftest.log`,
   `e2e-control.log`) are **historical**: they prove the behaviour of the retired bundle, not of
   this package.
2. **Teleport motion** (unchanged): `EntityRef::teleport_to` moves a player as `/tp` does, so the
   horizontal velocity on arrival is lost and no upstream sound/particles are sent.
3. **Suffocation** is the level's full-cell collision shape, not Java's `isSuffocating` on every
   shape-derived block (glass/leaves corner cases).
4. **Not carried:** camouflage, the per-block options screen, `facing`/`directional`/`show_arrow`
   states and the arrow model, `resetPitch*`, the XP cost, mob spawning, the config file
   (`config/elevatorid-server.toml`) and the sounds — the range/activation/`sameColor`/precision
   defaults are constants in `src/runtime.rs`.
5. The 16 blocks use the carried opaque template (light dampening 15), so an un-camouflaged
   elevator looks right but does not reproduce `noOcclusion`.
6. **No logger**: the SDK has no logger and a native mod may not add a crate, so nothing is logged;
   diagnostics are the e2e readouts.
7. **No `hooks` field in the manifest** (it is optional and unused by this package); the exact
   `[native]` commits are the contract.
8. **The carried content's own gaps** (from `content/README.md` before it moved): no block entity,
   no state properties, no GUI, sounds or XP; `minecraft:mineable/*` intentionally absent.
9. **Upstream defects:** none found in this migration; the three carry hand-fixes recorded in
   `content/README.md` (block properties, creative tab, `c:` dye tags) are carried data fixes, not
   behaviour defects of the rewrite.

### Traps new to this port

1. **The second machine has no `cargo-horizon-mod`.** The compile steps (`test`, `clippy`, `fmt`)
   forward fine, but `cargo horizon-mod build/check/fits` answered `error: no such command:
   horizon-mod` there. Run the package steps with `BUILD_LOCAL=1` (or install the tool remotely).
   The failed forwarded run is kept as `sdk-build-remote-no-cargo-horizon-mod.log`.
2. **A forwarded command already starts in the port.** `remote-slot.sh` mirrors the caller's
   directory, so the brief's literal `cd ports/elevatorid && …` dies with
   `cd: ports/elevatorid: No such file or directory` on the far box. Guard it:
   `if [ -d ports/elevatorid ]; then cd ports/elevatorid; fi`.
3. **`cargo test` writes a `Cargo.lock`** (`Locking 40 packages…`) and the packer would include it.
   `rm -f Cargo.lock` immediately before **every** build (both runs), and `/Cargo.lock` in
   `.gitignore`; the entry list is where you prove it (no `Cargo.lock` in the 196).
4. **An assertion and its message can disagree.** `assert!(travel.ready(PLAYER, 109), "9 ticks
   later is inside the cooldown")` failed *because the implementation was right*: the test needed
   `assert!(!…)`. Read the failure before "fixing" the code.
5. **Rewriting the crate root drops the module declarations.** The first glue run failed with
   `E0432: unresolved import crate::support` because `pub mod support;` was lost when `src/lib.rs`
   was replaced — a compile-level red, not the intended `todo!()` red.
6. **`cargo fmt --check` wants long signatures wrapped.** `support::on_server_tick` was over the
   width; the format pass caught it. Run fmt **before** the package build, or the packaged sources
   and the tree disagree.
7. **`check --json` reports `"capabilities": []` and `"side": null`** although the manifest carries
   `capabilities = ["registries"]` and `side = "both"`: those JSON fields describe what a
   *sandboxed* build needs, not the manifest. `uses.json`'s `capability:registries` is the
   manifest's. Do not "fix" the manifest from that JSON.
8. **`hook:blocks` appears in `uses.json` with no `Hooks::blocks` in `src/lib.rs`**: the package's
   `data/elevatorid/horizon/block/*.json` declarations make the content registration path part of
   what the package uses. Expected.
9. **Diff the carry against the built archive, not the retired folder.** Moving `content/assets`
   and `content/data` to the package root leaves the evidence `content/` without them; the
   authoritative retired copy is `retired-hnmod-0.1.0/content/elevatorid.hmod` (187 files, all
   unchanged by size and sha256).
10. **`LICENSE-NOTICE.md` was wrong before this migration** (it named ElevatorMod "26.3" and
    claimed notices inside `mixins/`, both false for this port). A packaged notice is shipped
    text; read it against the artefact. It now names 1.21.1-1.11.4, the jar's MIT, and the
    package's own layout.

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port,rust-review

## Review fixes (2026-11-18, PR #1233)

The reviewer found the live travel path passed `false` for same-colour matching
(`src/support.rs:116`), although the registry entry and this port promise same-colour travel: a
player could enter a differently coloured elevator's landing where public and private landings share
a column.

- **Fix.** `SAME_COLOR = true` in `support.rs` and `plan_live` is the only call the tick makes;
  `runtime.rs` keeps the planner's upstream-configurable parameter untouched (the file is still
  byte-identical, sha `ee6aa4ed…`).
- **Test.** `the_live_ride_lands_on_the_same_colour_and_skips_another`: a red landing at 66 between
  white ones at 63 and 69 is skipped (pre-fix it returned 66; the test failed with
  `left: Some(66)`). The README now says same-colour.
- **Also fixed to make the existing in-game assertions hold at `0b14f8a`.** `Column::suffocating`
  read `LevelRef::collision_boxes`, which refuses a parked level; at `ServerTickEnd` a level can be
  parked, so every cell read clear and the "stone above the elevator" case chose the blocked target.
  It now reads the state's own collision shape (`block_state` + `block_shape`, block-local), which
  needs no resident level. The e2e also clears the End exit portal's dragon egg from its column
  (it made an unmodified game's jump land at y=65).
- **In-game.** The run passes the obstruction step, skips a red landing for a white one, and an
  all-red column does not travel; the differential on an unmodified game stays at 64. Evidence:
  `../_shared/evidence/elevatorid-e2e/e2e-launcher.log` (`PASS` + `EXIT:0`) and
  `e2e-launcher-control.log` (`FAIL: expected y=67 after jump, got 64.0`, `EXIT:1`); checker green.
- **Package.** `elevatorid.hmod`, **0.2.1**, 135,618 B, sha256
  `3d3ef0d422981c94de59939ce4ab824eba1d8c530ff80a7429f913cf814ea71a`; two builds byte-identical,
  `check` and `fits --build 0b14f8a` exit 0.

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices, rust-port, rust-review

---

## Slice 3 — the arrival view: directional elevators (0.2.2, 2026-11-21)

**Class/tier:** content / **logic** (`ports/ledger.csv`; approach per `horizon/docs/modpacks.md`
§Tiers — content plus behaviour, no renderer, no machine). The brief's parenthetical "(deep)" is a
template slip; the port's row and this slice stay at `logic`. Licence MIT, rank 0.

**Pin:** SDK nightly **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (2026-11-20) + horizon-mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`** (unchanged). Repinned from `0b14f8a`; the delta is
additive for this port (no hook this package names changed), and `.horizon/sdk` was deleted before
the first build so the tool re-fetched and printed `SDK 0a875da (build 0a875da91db…) in
./.horizon/sdk` (trap: a stale `.horizon/sdk` survives a manifest repin).

### The gap this slice closes

From the migration section's gap 4, verbatim:

> **Not carried:** camouflage, the per-block options screen, `facing`/`directional`/`show_arrow`
> states and the arrow model, `resetPitch*`, the XP cost, mob spawning, the config file
> (`config/elevatorid-server.toml`) and the sounds

This slice takes the **arrival view**: the two block states `facing` and `directional`, and the
server rule behind `TeleportPacket.java:71-77` of the upstream clone
(`ports/_shared/upstream/elevatorid/`, commit `129614f8…`):

```
final float yaw = toState.getValue(DIRECTIONAL)
        ? toState.getValue(FACING).toYRot() : player.getYRot();
final float pitch = (directional && resetPitchDirectional) || (!directional && resetPitchNormal)
        ? 0F : player.getXRot();
```

So a **directional** landing turns you to the target's facing and levels the pitch; a plain one
keeps your view. Defaults (`Config.java:53-58`): `resetPitchDirectional=true`,
`resetPitchNormal=false`. Before this slice the port always passed the player's own angles through,
so a directional elevator — which upstream lets a server put at a shop exit — did nothing.

The **arrow overlay** (`show_arrow`) is *not* carried: upstream draws it from a block-entity
renderer, which is a `client` half this `logic` slice must not grow, and a block state nothing reads
is dead data. It stays in the gap list, below.

### TDD: red first

`src/arrival.rs` (new pure module: `Facing` (parse + `Direction.toYRot` yaw),
`Look`, `arrival(facing: Option<Facing>, current: Look) -> Look`) was written with its four tests
and `todo!()` bodies, then `cargo test --offline`:

```
test result: FAILED. 16 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
---- arrival::tests::a_plain_elevator_keeps_the_view stdout ----
thread 'arrival::tests::a_plain_elevator_keeps_the_view' (117953) panicked at src/arrival.rs:51:5:
not yet implemented
```

Full log (sha of the red tree in its header): `_shared/evidence/elevatorid/sdk-test-arrival-red.log`,
`EXIT:101`. Then the two functions were implemented, and all 20 tests pass
(`sdk-test.log`: `test result: ok. 20 passed; 0 failed`, `EXIT:0`).

### What changed

| file | change |
|---|---|
| `src/arrival.rs` | new: `Facing` (`north/south/west/east`, yaw 180/0/90/270), `Look`, `arrival`, the two `resetPitch*` constants, 4 tests |
| `src/support.rs` | `landing_look(level, host, at, rotation)`: reads the **target's** `directional`/`facing` with `LevelRef::block_property` (block-local, so it works at `ServerTickEnd` where a level can be parked) and passes `Some([yaw, pitch])` to `EntityRef::teleport_to`; a directional block whose facing does not parse keeps the view rather than inventing a direction |
| `data/elevatorid/horizon/block/elevator_*.json` (16) | `state_properties`: `facing` enum `["north","south","west","east"]`, `directional` boolean; `default_state`: `north`, `false` (upstream's `ElevatorBlockBase.java:53-58`) |
| `src/lib.rs`, `src/support.rs` docs | the new module and the turn |
| `horizon-mod.toml`, `Cargo.toml` | 0.2.2, `[native] horizon = 0a875da…` |
| `tools/e2e.sh` | two arrival phases, a recipe phase, `--offscreen` (see below), `SELFTEST=look` |
| `tools/craft_check.py` | new: crafts the shaped white recipe and all 15 redyes in a table |

`src/runtime.rs` is untouched: sha256 `ee6aa4ed109a4080f474f9906abbe2ffd07750b8a0aa8ac631d8cc3c47983025`,
identical in the tree and inside the package (the campaign's byte-identical carry).

The blockstate JSONs stay the single `""` variant upstream ships: Horizon's binder treats an empty
variant key as the predicate that matches every state (`horizon-assets/src/blockstate/bind.rs`,
`variant_predicate`), exactly as vanilla's fallback does, so 8 states share the one cube model.

### In game (the leg)

`/home/adlf/.cache/horizon-e2e/0a875da/elevatorid/horizon-client`, built by
`launcher-game.sh build elevatorid` from this package (`Compiling elevatorid (1 of 1)`), game 26.3,
launcher 0.2.92, SDK `0a875da`.

The arrival phases use a second column at x=3 so the travel phases' column is untouched:

```
== arrival view: a plain elevator keeps the view it arrived with ==
plain target state: elevatorid:elevator_white[directional=false,facing=east]
plain jump: y=67.0 yaw=33.0 pitch=40.0
== arrival view: a directional elevator turns you to its facing and levels the pitch ==
directional target state: elevatorid:elevator_white[directional=true,facing=east]
directional jump: y=67.0 yaw=270.0 pitch=0.0 (expected yaw=270 pitch=0)
```

Same block, **same `facing=east`**, only `directional` flipped: the plain phase is the differential
for the directional one, and the read-back `state` string shows the declared states really exist on
the placed block (not just in the registry).

Fail-capability: `SELFTEST=look` keeps the pre-0.2.2 expectation (`yaw=33 pitch=40`) for the same
scenario and must fail —

```
FAIL: expected yaw=33 after a directional ride, got 270.0
```

`SELFTEST=1` (the 0.1.0 body-predicate counterfactual) still fails at the blocked target
(`FAIL: expected y=64 (skipping the blocked 66), got 70.0`), and the control leg — the same
assertions on the same Nightly without the package — fails at the first jump
(`FAIL: expected y=67 after jump, got 64.0`). `check-e2e-logs.py elevatorid-e2e --strict`:
`logs=4 ok=4 missing-exit=0 bad-exit=0`.

**Why `--offscreen`.** The recipe phase drives a crafting table with `ctl click-slot`, which answers
`{"error":"no renderer yet"}` under `--headless`; `--offscreen` gives the same headless play with a
renderer. It also keeps this leg local (`remote-build.md`: the far box has no renderer).

### Recipes, crafted for real (the parity rule)

The port had shipped 32 recipe files since the first carry and **never crafted one**. The phase now
opens a fresh crafting table per case (`craft_check.py`) and asserts the output id/count and that
the ingredients were consumed, from the hotbar after `clear @s`:

- `elevator_white`: shaped, 8 `minecraft:white_wool` in the ring, 1 `minecraft:ender_pearl` centre
  → 1 `elevatorid:elevator_white`, wool 0, pearl 0.
- 15 redyes: shapeless, 1 elevator of `#elevatorid:elevators` + 1 `#c:dyes/<colour>` → that
  colour, both inputs 0. All fifteen pass, which also proves the carried `data/c/tags/item/dyes/*`
  tags resolve at 26.3 (an unresolved tag would drop the recipe).
- control: eight wool and **no pearl** crafts nothing (the shaped pattern discriminates), and the
  eight wool come back after the close.

```
PASS: 16/16 recipes crafted, each consumed its ingredients; eight wool alone is not the pattern
```

### Coverage table (families in the jar)

| family | status |
|---|---|
| 16 coloured blocks + items, models, textures, lang | `tested` — travel/colour/arrival phases place and read them; `state` strings read back |
| block state `facing` (4 values) | `tested` — `plain`/`directional` phases read `[directional=…,facing=east]` back from the placed block; `setblock` accepts the property |
| block state `directional` | `tested` — the two arrival phases are each other's differential (only this flag differs) |
| block state `show_arrow` | `not ported` — the arrow is drawn by a block-entity renderer (client half); a state nothing reads would be dead data |
| jump-up / sneak-down travel, nearest target, range 384, activation 6 | `tested` — `jump`/`sneak` phases (67 / 64) |
| obstructed target skipped, scan continues | `tested` — stone above 66 → 70 |
| same-colour rule (`sameColor=true`, the registry promise) | `tested` — red landing at 66 skipped for white 69; all-red column does not travel |
| precision landing (x/z centred, feet on top) | `tested` — positions `[3.5, 67, 0.5]`, `[0.5, 67, 0.5]` |
| arrival view: plain keeps, directional turns + levels pitch | `tested` — the two arrival phases; `SELFTEST=look` fails on the old expectation |
| 16 recipes (1 shaped + 15 shapeless) | `tested` — `craft_check.py`, 16/16, ingredients consumed, 8-wool control empty |
| 16 loot tables (block drops itself) | `tested` — the `break` in the recipe/placement work returns the id (the drop is the block's own item; see the 0.1.0 ladder) — **see note** |
| 3 tags (`elevatorid:elevators` block+item, `c:dyes/*`) | `tested` via the redyes (the tag is the recipe's ingredient) |
| camouflage, options GUI, XP cost, mob spawning, sounds, config file | `not ported` — gap 4; the config has no SDK file API at `0a875da` (`grep config_dir` over the SDK is empty) |
| `noOcclusion` / light dampening | `not ported` — template fidelity, gap 5 (`BlockStates::{light_emission,is_solid_render}` exist at `0a875da` but `noOcclusion` has no content switch) |

**Note on loot:** the block drops were verified in the 0.1.0 bundle's ladder (`break` → the item),
not re-run in this slice; the 16 loot tables are unchanged. If the next slice touches drops, re-assert
them in the same leg.

### Deviations (deliberate)

1. **The two pitch switches are constants**, not a config file: the SDK at `0a875da` has no config
   directory API for a native mod (`Game::native_config_dir()` belongs to the retired hook API).
   Upstream's defaults (`true` / `false`) are `RESET_PITCH_DIRECTIONAL` / `RESET_PITCH_NORMAL` in
   `src/arrival.rs`, and `arrival` uses them, so wiring a config later changes one place.
2. **A directional elevator with an unreadable `facing` keeps the view** instead of turning to yaw 0:
   a failed state read must not spin the player.
3. **`show_arrow` is not declared** (above). Camouflage/XP/sound/mob-spawn stay out, as before.
4. The **horizontal motion on arrival** is still `/tp`'s (zeroed), gap 2 unchanged; `PlayerRef::set_velocity`
   is new at `0a875da` and is the next slice's tool (see the report's next slice).

### Ladder (all in `build-slot.sh` leases, `CARGO_INCREMENTAL=0`; package steps `BUILD_LOCAL=1`)

| Step | Command | Result | Evidence |
|---|---|---|---|
| tests (red) | `cargo test --offline` | 16 passed, **4 failed** `not yet implemented`, `EXIT:101` | `sdk-test-arrival-red.log` |
| tests | `cargo test --offline` | **20 passed**, 0 failed, `EXIT:0` | `sdk-test.log` |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings | `sdk-clippy.log` |
| format | `cargo fmt --check` (not `--all`) | exit 0 | `sdk-fmt.log` |
| package | `rm -f Cargo.lock && cargo horizon-mod build` ×2 | `SDK 0a875da … in ./.horizon/sdk`, `Built ./target/horizon-mod/elevatorid.hmod`, identical bytes | `sdk-build.log`, `sdk-build-second.log`, `sdk-reproducible.log` |
| registry check | `cargo horizon-mod check elevatorid.hmod` | `elevatorid 0.2.2 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, 139,378 B, sha256 `f17dcbc7…`, **EXIT 0** | `sdk-check.log` |
| fits the pin | `cargo horizon-mod fits elevatorid.hmod --build nightly` | `0a875da  made for`, EXIT 0 (a tautology, kept) | `sdk-fits.log` |
| fits another build | `… --build 0b14f8acfd97fa2c750e42795249dcc981178ca4` | `0b14f8a  fits`, EXIT 0 | `sdk-fits-other.log` |
| game build | `launcher-game.sh build elevatorid` | `Compiling elevatorid (1 of 1)`, `native-0a875da-a5231f3c9b96` | `sdk-build` of that leg / the e2e logs |
| e2e | `tools/e2e.sh` (`--offscreen`) | **PASS + `EXIT:0`** (travel, colour, arrival, 16 recipes) | `elevatorid-e2e/e2e-launcher.log` |
| e2e control | same assertions, unmodified Nightly | **`FAIL: expected y=67 after jump, got 64.0` + `EXIT:1`** | `elevatorid-e2e/e2e-launcher-control.log` |
| e2e selftest | `SELFTEST=1` (blocked target) | **`FAIL: expected y=64 …, got 70.0` + `EXIT:1`** | `elevatorid-e2e/e2e-selftest.log` |
| e2e selftest (this family) | `SELFTEST=look` | **`FAIL: expected yaw=33 …, got 270.0` + `EXIT:1`** | `elevatorid-e2e/e2e-selftest-look.log` |
| log gate | `check-e2e-logs.py elevatorid-e2e --strict` | 4/4 ok, exit 0 | — |
| package gate | `verify-sdk-package.py elevatorid` | 1/1 | — |

Artifact: `ports/elevatorid/elevatorid.hmod`, **139,378 B**, sha256
`f17dcbc7f3f4df8aec1392fef3cb40af93bff7d4ba1e80df8317ea3a7048de0e`, **197 entries** (5 root
documents, 187 carried `assets/`+`data/` files, `src/lib.rs`, `src/arrival.rs`, `src/runtime.rs`,
`src/support.rs`, `uses.json`), **no `Cargo.lock`**, version **0.2.2**.

### Gaps left after this slice

1. **Teleport motion** (gap 2, unchanged): `EntityRef::teleport_to` `/tp`s as a command does, so the
   horizontal velocity on arrival is lost and no teleport sound is played. `PlayerRef::set_velocity`
   is new at `0a875da` — the next slice.
2. **Arrow overlay / `show_arrow`** — client renderer.
3. **Camouflage + options GUI** — block entity + menu (a `machines`/`client` half).
4. **Config file** — no SDK config-dir API at this pin; the switches are constants.
5. **Suffocation** remains the level's full-cell collision shape (glass/leaves corner cases), and the
   blocks keep the opaque stone template (`noOcclusion` not expressed).
6. **No logger** (no logger in the SDK).

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices, rust-port, rust-review
