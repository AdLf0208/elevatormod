# ElevatorMod — re-pin audit (read-only, no code changed, no builds)

Target pin: **Horizon `83ad384f`** (427 hooks / 159 helpers) at
`ports/_shared/pins/horizon-83ad384f`, mixins **`e8cd0aa`** at
`ports/_shared/pins/horizon-mixins-e8cd0aa`. Port is currently pinned at
`c9d474d` / `d9c83d7` (`compatibility.json`).

Method: read `notes.md` §Status/§Gaps/§"Pin facts settled" + ARCHITECTURE
items 3 and 10, then searched `native-hooks.json` and
`crates/horizon-client/src` at the new pin. No port code, bundle, content or
compatibility file was modified; nothing was built or run in-game.

**Evidence baseline.** `fbb17c980` (the campaign's read-only `horizon/`
provider) has 186 hooks and is newer than `c9d474d` (108). The new pin contains
every hook `fbb17c980` has (no removals in the diff), so a hook **absent from
`fbb17c980` did not exist at `c9d474d`**; a hook present there may predate
`c9d474d`. Verdicts below use that conservative reading.

Class: **usable** = writable/cancel/replace/collect or a helper the bundle can
call; **hook-only** = listen-only; **absent** = none.

## Deferred features (10)

| # | Deferred (notes/ARCHITECTURE) | Evidence at `83ad384f` | Class | Verdict | Next step |
|---|---|---|---|---|---|
| E1 | **Teleport = deferred `/tp`** (ARCHITECTURE §3 "player teleport / position write"): one-tick latency, velocity zeroed, level-4 permission | Still **no position write**: `native_run_command` keeps the "no player state swapped in" guard (`native_server.rs:472-479`); `native_teleport_mob` is mobs only (`native_mobs.rs:236`); `native_move_entity` is for entities Horizon does not simulate — "a display, a marker, an interaction" (`native_entities.rs:187-192`); `native_push_player` is client-local velocity (`native_motion.rs:31`). `native_player_teleported` (`native_hooks.rs:4261`) and `native_entity_teleport` (`:5425`) only hear a move **after** it | absent (write) | **Blocked as a direct API; the proven deferred-command path remains the only implementation.** Nothing at this pin shortens the one-tick latency or preserves velocity. | Keep the `/tp` queue at `server_tick`. Issue unchanged: "a direct hook to move the acting player, with chunk/packet/respawn semantics". |
| E2 | **Input edges / jump detection** (notes §Input detection, §Gaps): `shift_key_down` edge works; jump inferred from the body so a ceiling-clipped jump is missed; guest raw keys unreachable | `native_player_input(player, name, flags)` (`native_hooks.rs:6358`, **API 343**, after the `PlayerInput` packet is applied, guest swapped in); `native_player_jump(player, name)` (`:6366`, **API 344**, after accepted movement made the server player jump **from the ground**); `native_player_toggle_sneak` (`:4107`, **API 351**); `native_player_velocity` (`:5307`); flags are `forward 1/back 2/left 4/right 8/jump 16/shift 32/sprint 64` (`lan/guest.rs:131`, `lan_vehicles.rs:25-31`); client-only `native_movement_input` (`docs/native-mods.md:1601`, mutable) cannot serve guests | usable (new) | **Realizable now — this closes the recorded gap.** `player_jump` is an input-edge event, so a ceiling-clipped jump is still reported; `player_input` gives the exact flags; `player_toggle_sneak` replaces the `shift_key_down` inference. All three are **new since `fbb17c980`**, i.e. new versus `c9d474d`. | Replace the `velocity>0`/history inference in `github support.rs` with **`player_jump`** for up and **`player_toggle_sneak`** (or `player_input` bit 32) for down. Keep the 10-tick per-uuid cooldown. Prove the previously failing blocked-target case (elevators 63/66/69 + stone at 67 → 70) in the existing `tools/e2e.sh`. |
| E3 | **Suffocation approximation** (`facts.full_collision && !air`; glass/leaves corner cases differ) | `native_block_collision(pos)` (`native_hooks.rs:9050`) returns the block's actual collision boxes in block-local coordinates with an empty collision context; **new helper**, not in `fbb17c980` | usable (new) | **Realizable now.** `isValidPos = !above.isSuffocating(above)` can test the real boxes instead of a full-cube flag. | Read `native_block_collision` for the cell above the target; treat a box that covers the entity column as suffocating (match vanilla `isSuffocating`). Add a glass and a leaves case to the e2e. |
| E4 | **Camouflage** (block/particles show the copied block; `TileElevator` holds the camo state) | `native_block_entity` / `_ticks` / `_tick` / `_update_tag` (`native_hooks.rs:2985,2996,3122`; API 15/17); `native_block_entity_submit` (`:3190`, API 71) + draw helpers `native_draw_quad`/`draw_cube`/`draw_block`/`draw_item` (`native_block_entity_draw.rs:548,620,653,719`); `native_right_click_block` (`:6790`) / `native_use_item_on` (`:6482`) to set it; `native_block_entity_render_bounding_box` (`:3216`) | usable | **Realizable now.** A BE stores the camo state, a block item click sets it, `block_entity_submit` draws the camo block (or the port declares the camo state in the BE and renders it). Rendering was not a pin gap; it was scope. | Add the BE + `block_entity_submit` camo slice; keep the elevator block's own `stone` template for collision. Probe in-game with stone/dirt camo. |
| E5 | **Per-block options GUI** (directional / hide arrow) | `native_open_menu` (`native_menus.rs:124`, arbitrary title/slots), `native_set_menu_items` (`:143`), `native_menu_clicked`/`menu_clicked` (API 10), `native_screen`/`native_screen_pos`/`native_screen_extra_data` helpers, `native_menu_button`/`menu_button` (API 10/…), `native_mod_screen` (`native_hooks.rs` helper list) | usable | **Realizable now** (scope, not API). | Optional small menu after E2/E4; use `open_menu` + `menu_clicked` (the akashictome precedent). |
| E6 | **Directional model/arrow/yaw, `resetPitch*`** | Content: `state_properties`/`default_state` (`content.rs:272-278`) and `native_state_for_placement` (`:1727`); hook helpers `native_block_state_property`/`native_set_block_property`; the notes already record that the shipped blockstate is a single variant (facing only drives yaw). | usable | **Realizable now** (scope). `facing` placement and yaw-on-arrival are writable; a directional *model* is content work, not an API. | Fold into E4/E5; keep `facing` + `resetPitchNormal`/`resetPitchDirectional` as config (E9) and set yaw from the target's `facing`. |
| E7 | **XP cost** (`useXP`, `XPPointsAmount`) | `native_give_experience` / `native_give_levels` (`native_server.rs:856,870`, charge **and** reward); `player_exp_change` (`native_hooks.rs:4622`, listen, orb pickups only); `player_level_change` (`:6376`, listen, from/to); **no absolute XP read helper** | usable (charge) / hook-only (read) | **Realizable with bookkeeping.** Charge via `give_experience(uuid, -n)` through the u32 direction is awkward; use `give_levels(-n)` or in-crate points read. The absence of an absolute XP read is a small gap. | Prefer `native_give_levels(uuid, -n)` (integer levels) or read the player's points in-crate from the support module; add `useXP`/`XPPointsAmount` to the E9 config. |
| E8 | **Mob spawning on arrival** (`mobSpawn`) | `native_spawn_mob` (`native_mobs.rs:166`, API 37); `native_mob`/`native_mobs`/`native_remove_mob`; `native_mob_spawned` (`native_hooks.rs:7221`, listen) | usable | **Realizable now** (scope). | Config-gated `native_spawn_mob` at the destination on arrival; run it from `server_tick` after the `/tp`. |
| E9 | **Config file** (`serverconfig/elevatorid-server.toml`; defaults are constants) | `native_register_configs` (`native_hooks.rs:2424`, **API 397**, `NativeConfig::new(mod, kind, default)` → `config/<mod>-<kind>.toml`), `native_config_loading`/`reloading`/`unloading` (`:2432`, API 397), `native_config_dir` helper | usable (new) | **Realizable now — the gap closes.** Server configs are read from `serverconfig/` before `native_server_about_to_start`; the same hook covers `range`/`activationRange`/`sameColor`/`precisionTarget`/`mobSpawn`/`resetPitch*`/`useXP`. API 397 is **new since `fbb17c980`**. | Register a `Server` config with the upstream defaults inside the bundle and read it in `native_config_loading`; prove one non-default value in e2e. |
| E10 | **Teleport sound/particles** (notes: `native_play_sound` is on `GameLevel`, unreachable from `server_tick`) | `native_play_sound` **on `GameLevel`** (`native_hooks.rs:8193`, API 61) — reachable from a `GameLevel` hook such as `level_tick_post`; `native_play_local_sound` (client, `:8782`, API 83); `native_flame_particle` (`:8426`, flame/soul only) | usable (sound) / limited (particles) | **Sound realizable now** by queueing at `server_tick` and flushing in `level_tick_post` for that dimension. **Tinted camouflage particles remain limited to vanilla flame/soul.** | Flush a `(dimension, pos, sound)` queue in `level_tick_post` via `native_play_sound("elevatorid:teleport", …)`; leave tinted particles deferred. |

## Summary

- **Deferred: 10** (E1–E10; the held-jump auto-climb mitigation is already a
  10-tick cooldown, so it is not counted).
- **Realizable now: 8** — E2, E3, E4, E5, E6, E7 (bookkeeping/levels), E8, E9,
  plus E10's sound. Of these, **E2, E3 and E9 depend on APIs that are new
  since `fbb17c980`** (`player_jump`+`player_input`+`player_toggle_sneak`,
  `block_collision`, `register_configs`), i.e. they are genuinely unblocked by
  this re-pin.
- **Still blocked: 1** — E1 (direct player position write; the deferred `/tp`
  path stays), with E10's *tinted* particles as a limited cosmetic remainder.
- **Highest-value next step:** **E2**, `player_jump` + `player_input` /
  `player_toggle_sneak`. The port's only failing e2e assertion (a jump under a
  2-block ceiling is clipped and the hook sees a fall) disappears when the
  trigger is the server's own jump event instead of the post-collision body.
  This is a small bundle edit (`support.rs`/`player.rs` new injections) and the
  existing `tools/e2e.sh` already asserts the blocked-target case.

No build, test or e2e was run for this audit; all claims above are source
reads at `83ad384f`.

## Addendum — re-pin to `1d63e0a` (2026-11-08): E1 replaced by the native helper

The audit above was written against `83ad384f`, where **E1** (deferred `/tp`)
had no position write. At the campaign pin `1d63e0a`
(`ports/_shared/pins/horizon-1d63e0a`) the pending fragment
`tools/native-hook-fragments/teleport_player.json` adds
`native_teleport_player(uuid, dimension, pos, yaw, pitch)`
(`crates/horizon-client/src/game/native_travel.rs:55`). A bundle may **call**
a pending helper even though it may not **name** the pending hook
(`ports/_shared/repin-1d63e0a-smoke.md` §2/§5), so E1 is no longer a blocker
for this port.

| E1 item | `83ad384f` | `1d63e0a` (this slice) |
|---|---|---|
| move | `execute in <dim> run tp …` via `native_run_command` | `native_teleport_player`, immediate from the `server_tick` hook |
| console level 4 | required | none |
| latency | one queue step (same server tick) | unchanged (same tick; see `notes.md`) |
| motion on arrival | lost | still lost (the helper `/tp`s as a command does) |
| sound/particles | none | still none (E10, separate slice) |
| declaration | — | helper is **pending**: needs `"hooks": @API:teleport_player@` → projected **555**; the pin's `NATIVE_HOOKS_API` is 411, so no honest number is declared yet (`tools/package.py` omits `hooks`) |

Verification (2026-11-08): stage/check/test/build at `1d63e0a` all exit 0 with
0 warnings (10 embedded tests), and `tools/e2e.sh` passes exit 0 —
jump 64→67, sneak 67→64, plain floor stays, lone elevator stays, stone above the
middle elevator skips to 70 — with `SELFTEST=1` failing as required. Logs:
`/tmp/opencode/elevatorid-{ladder,e2e,e2e-selftest}.log`; details in
`notes.md` §"Re-pin to `1d63e0a` + the native player teleport".

"Still blocked: 1" in the summary above therefore describes `83ad384f` only;
at `1d63e0a` the transport is resolved and the remaining E1 deviation is the
lost horizontal motion (the helper offers no velocity parameter).

Skills: superpowers/using-superpowers,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices
