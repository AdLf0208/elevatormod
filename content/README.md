# ElevatorMod — carried content mod

ElevatorMod 1.21.1-1.11.4 (MC 1.21.1, NeoForge) carried from its jar into a Horizon
content mod (`.hmod`): 16 elevator blocks (one per dye colour), their block items,
blockstates/models/item definitions, textures, sounds, 19 lang files, 32 recipes,
16 loot tables and the two `elevatorid:elevators` tags. This folder **is** the mod
root (`horizon-mod.toml` + `assets/` + `data/`); `elevatorid.hmod` is the built
bundle. Content only, no code: jumping up / sneaking down, camouflage and the
options GUI belong to the native port in `ports/elevatorid/`.

## Provenance

| | |
|---|---|
| Modrinth project | `hi2dSXTu` (slug `elevatormod`) |
| Version | `neoforge-1.21.1-1.11.4` (version id `NKEMiRIu`), game 1.21.1, loader neoforge |
| Jar | `elevatorid-neoforge-1.21.1-1.11.4.jar`, 191 190 bytes |
| SHA-1 | `87a5504efcd51de55cd4215a103cbfcf717fa022` (matches Modrinth's `files[].hashes.sha1`) |
| Download | <https://cdn.modrinth.com/data/hi2dSXTu/versions/NKEMiRIu/elevatorid-neoforge-1.21.1-1.11.4.jar> |
| Licence | MIT (`META-INF/neoforge.mods.toml`: `license = "MIT"`); local-only until the maintainer approves publishing (ARCHITECTURE.md §8) |
| Upstream read | `https://github.com/VsnGamer/ElevatorMod`, branch `1.21-architectury`, commit `6dd5255bb41e4a9caeff12e3a0846212ba5cd11d` (`gradle.properties`: `mod_version = 1.21.1-1.11.4` — the branch head is exactly the jar's build) |

Files read upstream: `common/src/main/java/com/vsngarcia/ElevatorBlockBase.java`,
`neoforge/src/main/java/com/vsngarcia/neoforge/ElevatorBlock.java`,
`neoforge/src/main/java/com/vsngarcia/neoforge/init/Registry.java`,
`common/src/main/resources/assets/elevatorid/blockstates/elevator_white.json`.
Behaviours extracted and used here: `HorizontalDirectionalBlock` with
`FACING`/`DIRECTIONAL`/`SHOW_ARROW`; properties
`mapColor(color).sound(SoundType.WOOL).strength(0.8F).dynamicShape().noOcclusion()`;
no `requiresCorrectToolForDrops()` and no mineable tag; the NeoForge block has no
light level (the Fabric one adds a `LIGHT` property); the tab is `elevators_tab`,
title `itemGroup.elevators_tab`, icon the white item, items in `DyeColor` enum
order. The block's `facing`/`directional`/`show_arrow` states are read by the
client's `ElevatorBakedModel`, which is why the jar's blockstates have a single
`""` variant.

## Carry

Scratch work folder: `ports/_shared/carry-work/elevatorid/` (jar, `apply_fixes.py`,
`content-check.sh`, the carry output in `check-out/`, the recipe probe in
`recipe-probe.json`; all gitignored, never shipped).

```bash
cd ports/_shared/carry-work/elevatorid
/tmp/opencode/atm10-carry-target/debug/horizon-port-pack \
    check elevatorid-neoforge-1.21.1-1.11.4.jar /tmp/opencode/elevatorid-check
```

Verbatim output (exit 0):

```
elevatorid: Partly. 16 blocks, 32 recipes port; not 2 blocks with behaviour of their own (use, ticks, collisions), 1 item with behaviour of their own (use, on hit).
  ports: 16 blocks
  ports: 32 recipes
  ports: 16 loot tables
  ports: 18 textures
  ports: blocks take the closest vanilla block's shape, hardness and sounds
  skipped: 2 blocks with behaviour of their own (use, ticks, collisions)
  skipped: 1 item with behaviour of their own (use, on hit)
  skipped: 2 block entities (machines, containers, animated blocks)
  skipped: 1 menu (a container screen)
  skipped: 2 screens of its own
  skipped: 5 network messages
```

`carry-report.json`: 138 copied, namespaces `[]`, outside `[]`, unreadable `{}`,
32 recipes upgraded, 16 loot tables upgraded, 16 item definitions added, 0
convention tags, oracle `false`, all 16 blocks guessed `minecraft:stone`. Then:

```bash
cd ports/elevatorid/content
cp -r /tmp/opencode/elevatorid-check/carry/assets .
cp -r /tmp/opencode/elevatorid-check/carry/data .
cp /tmp/opencode/elevatorid-check/carry/horizon-mod.toml .
python3 ../../_shared/carry-work/elevatorid/apply_fixes.py   # the 3 hand fixes
CARGO_TARGET_DIR=/tmp/opencode/atm10-carry-target cargo run --locked --offline --quiet \
  --manifest-path ../../../horizon/crates/cargo-horizon-mod/Cargo.toml -- horizon-mod build
# Built ./target/horizon-mod/elevatorid.hmod
cp target/horizon-mod/elevatorid.hmod . && rm -rf target
CARGO_TARGET_DIR=/tmp/opencode/atm10-carry-target cargo run --locked --offline --quiet \
  --manifest-path ../../../horizon/crates/cargo-horizon-mod/Cargo.toml -- horizon-mod check elevatorid.hmod
```

```
elevatorid 1.21.1-1.11.4 (mod, API 0.2, both)
  119915 bytes, SHA-256 b16e3a4b1bb7d943e4ff11635e5e3604c498fcd7d22fb692ae9c1fcf08142e81, unsigned
  may add blocks, items, mob effects and entity types to your worlds
  no code: content only, nothing to run
```

The `.hmod` (188 entries) = `horizon-mod.toml` + 187 assets/data files.

### Block ids (16)

`elevatorid:elevator_white`, `_orange`, `_magenta`, `_light_blue`, `_yellow`, `_lime`,
`_pink`, `_gray`, `_light_gray`, `_cyan`, `_purple`, `_blue`, `_brown`, `_green`,
`_red`, `_black` — the jar's ids, untouched (`elevator_white`, not `elevatorWhite`).

| Path | Files | |
|---|---|---|
| `assets/elevatorid/blockstates` | 16 | carried, one `""` variant each (no state properties) |
| `assets/elevatorid/items` | 16 | 26.3 item model definitions (carry added all 16) |
| `assets/elevatorid/models/block` | 16 | carried `cube_all` models, one texture each |
| `assets/elevatorid/models/item` | 16 | carried, parent = the block model |
| `assets/elevatorid/models/arrow.json` | 1 | carried, the baked model's arrow (unused here) |
| `assets/elevatorid/lang` | 19 | carried (`en_us` + 18 translations) |
| `assets/elevatorid/textures/block` | 17 | 16 block textures + `arrow.png` |
| `assets/elevatorid/textures/gui` | 1 | `elevator_gui.png`, the native options screen's art (unused here) |
| `assets/elevatorid/sounds.json` + `sounds/teleport.ogg` | 2 | carried |
| `data/elevatorid/horizon/block` | 16 | declarations (this port) |
| `data/elevatorid/horizon/creative_tab` | 1 | added here |
| `data/elevatorid/recipe` | 32 | upgraded by carry: 16 shaped + 16 redye |
| `data/elevatorid/loot_table/blocks` | 16 | upgraded by carry (drops itself) |
| `data/elevatorid/tags/{block,item}/elevators.json` | 2 | carried (16 blocks / 16 items) |
| `data/c/tags/item/dyes` | 16 | added here (`<colour>_dye`) |

## The block declarations, verbatim

All 16 are identical but for `map_color`
(`data/elevatorid/horizon/block/elevator_white.json`):

```json
{
  "template": "minecraft:stone",
  "properties": {
    "hardness": 0.8,
    "explosion_resistance": 0.8,
    "requires_tool": false,
    "sounds": "minecraft:white_wool",
    "map_color": 8
  }
}
```

**No `state_properties`.** The jar's blockstates are a single `""` variant: the
Java block does have `facing` (horizontal), `directional` and `show_arrow`
properties, but on NeoForge they are consumed by the client `ElevatorBakedModel`,
not by blockstate JSON, so the carried model is the plain cube and declaring
states here would only add variants with nothing to render. When the native port
adds the arrow/camouflage it must declare the states it needs itself (a
block-state property is a content or native declaration; the block entity in
Java holds the held state, not the state properties).

Why `stone`: it is the only vanilla template that gives a full, opaque,
non-ticking cube with a full collision and outline and nothing else. `glass`
would be `solid_render: false` / `can_occlude: false` (wrong for an opaque
wool-textured cube). The carry's guess was already `stone` because the
blockstates have no properties; only the numbers and the sounds were wrong:
Java's `strength(0.8F)` sets **both** destroy time 0.8 and explosion resistance
0.8 (stone's 1.5 / 6.0 are overridden), `SoundType.WOOL` → borrow
`minecraft:white_wool`'s sounds, and `requiresCorrectToolForDrops()` is never
called, so `requires_tool: false` (stone's `true` would otherwise be inherited
— the block dropped by hand in the check below). `map_color` is
`DyeColor.mapColor`: `WHITE` is `MapColor.SNOW` (8), the other colours are the
same-named ids 15–29; horizon-items maps them by index in `MAP_COLORS`, i.e. the
vanilla 1.21.1 order. White is 8 (`0xFFFFFF`), black 29 (`0x191919`), and so on;
the built facts show each block's own colour.

## Hand fixes (what the carry got wrong or left out)

1. **Block properties** — the carry wrote `{"template": "minecraft:stone"}` with
   no `properties`, so every block got stone's hardness 1.5, resistance 6.0,
   `requires_tool: true`, stone sounds and stone map colour. Rewritten to Java's
   0.8 / 0.8 / wool / no tool / dye map colour (above).
2. **Creative tab** — the carry wrote none. Added
   `data/elevatorid/horizon/creative_tab/elevators.json`: title
   `itemGroup.elevators_tab` (the key the jar's lang files have), icon the white
   item, the 16 items in `DyeColor` enum order (`white, orange, magenta,
   light_blue, yellow, lime, pink, gray, light_gray, cyan, purple, blue, brown,
   green, red, black`) — the order `EnumMap` gives the Java tab.
3. **Convention tags** — the carry leaves `c:` tags alone
   (`convention_tags: 0`) and the 16 redye recipes use `#c:dyes/<colour>`; an
   unknown tag expands to an empty ingredient, so those recipes would load and
   never match. Wrote the 16 tags by hand under `data/c/tags/item/dyes/`, each
   `{"values": ["minecraft:<colour>_dye"]}`.
4. **Lang** — nothing to rewrite: the jar already uses vanilla keys
   (`block.elevatorid.elevator_<colour>`, `itemGroup.elevators_tab`), unlike the
   NeoForge `simplemagnets.item.*` case. Confirmed in-game: `give` prints
   "White Elevator", "Light Blue Elevator", "Black Elevator".
5. **Block tags** — none needed. The jar ships only `elevatorid:elevators`
   (carried) and never calls `requiresCorrectToolForDrops()`, so no
   `minecraft:mineable/*` or `needs_*_tool` entry; adding one would be a
   behaviour change.
6. **Blockstates / item models** — carried, none rewritten. The carry added the
   26.3 `assets/elevatorid/items/<name>.json` definitions (the 1.21.1 jar has
   none) and left the `cube_all` block models and their `parent` item models as
   the jar has them.

## Evidence

Content builds and loads with no warnings: the launch log prints
`Horizon: content from 1 mod(s) in …/.horizon-cache/mod-content` and that
cache's `horizon-mod-content-warnings.txt` is 0 bytes. Headless run (data only,
the `content-check.sh` in the work folder; fast build at the pin, scratch
`HORIZON_HOME`/`XDG_*`, content folder as `--dev-mod`):

- **Blocks exist, one state each.** Built `model-states.json` has all 16 with
  `"schema": {}`, `"states": [{}]`; `render-definitions.json` gives each one
  state, `can_occlude: true`, `rule: "never"`. In-game, `setblock x 200 z
  elevatorid:elevator_<colour>` followed by `ctl block` returned
  `"block":"elevatorid:elevator_<colour>"` for all 16.
- **Facts.** Built `gameplay-facts.json` rows for all 16: hardness 0.80,
  `requires_tool: false`, explosion resistance 0.8, `can_occlude: true`,
  dampening 15, `render_shape: "MODEL"`, full-collision outline, wool
  break/step sounds, `emission: 0`, map colours `ffffff`, `d87f33`, `b24cd8`,
  `6699d8`, `e5e533`, `7fcc19`, `f27fa5`, `4c4c4c`, `999999`, `4c7f99`,
  `7f3fb2`, `334cb2`, `664c33`, `667f33`, `993333`, `191919`.
- **Recipes resolve their tags.** `cargo run -p horizon-client-lib --example
  mod_port_probe -- <prepared 26.3> <work> out.json elevatorid` loaded the
  content root and wrote 32 recipe rows, **none with an empty ingredient**:
  `elevatorid:elevator_white` = 8× `minecraft:white_wool` + `minecraft:ender_pearl`
  → 1 white elevator; `elevatorid:redye_black` = the 16-item `elevatorid:elevators`
  tag + `minecraft:black_dye` → 1 black elevator (same for the other 15
  colours). `elevatorid:elevators` resolves to all 16 items.
- **Drops.** Survival `ctl break 4 200 4` on a placed lime elevator:
  `"drops": {"elevatorid:elevator_lime": 1}`, no item left on the ground, picked
  up into the inventory — the loot table and `requires_tool: false` both hold.
- **Item → block.** `ctl place 7 200 8 --item elevatorid:elevator_purple` placed
  `elevatorid:elevator_purple` (the carried 26.3 item definition + block item work).
- **Creative tab.** Built `creative-inventory.json` has tab
  `elevatorid:elevators`, type `CATEGORY`, title `itemGroup.elevators_tab`, icon
  `elevatorid:elevator_white`, and all 16 items in DyeColor order.

## Gaps (remaining, and why)

1. **No block entity, so no camouflage and no arrow.** The content format has no
   block-entity declaration (see `ports/restrictions/content/README.md` §Gaps for
   the loader's key list): the block entity holds the held state (camo), the
   facing, `directional` and `show_arrow`, and the dynamic light/shape/sound/
   friction/redstone delegates. All of that is the native port's job
   (`native_block_entity`, `native_block_entity_tick`, a menu).
2. **`noOcclusion()` / `dynamicShape()` not expressed.** `stone` occludes and
   culls neighbour faces and blocks light (`dampening: 15`); Java's elevator does
   not occlude and takes its shape, light and sounds from the camouflage. With no
   camo the visible result is the same opaque cube, but a camouflaged elevator
   will not reproduce Java's lighting/face-culling until the native port renders
   it.
3. **No state properties.** `facing`, `directional` and `show_arrow` are absent
   from the carried block (and from its blockstates). The native port must
   declare them (and the arrow model) itself; the carried `models/arrow.json`,
   `textures/block/arrow.png` and `textures/gui/elevator_gui.png` are there for it.
4. **No GUI, sounds or XP.** The carried `sounds.json` (teleport, camouflage),
   the 29 `en_us` keys (`screen.elevatorid.*`, `elevatorid.message.missing_xp`,
   `elevatorid.subtitle.*`, `tag.item.elevatorid.elevators`) and the GUI texture
   are inert until the native screen, XP cost and sound calls exist.
5. **Not visually verified.** This environment is headless (`ctl screenshot` →
   "no renderer yet"), so nobody has seen the 16 cubes or item icons drawn. The
   blockstates, models, item definitions and 16×16 textures are in the built
   overlay and the model tables have one state per block, but the pixels are
   unchecked.
6. **No `minecraft:mineable/*` tag** (deliberate, see fixes 5): the block breaks
   by hand like Java's. If the pack's balance wants a tool, that is a behaviour
   change, not a carry fix.
7. **Native behaviour absent by design** (this folder is data only): jump
   up / sneak down, the same-colour/range/precision/activation config, the
   obstruction rule, fall-damage/pitch handling. See `ports/elevatorid/notes.md`
   and the ledger row.

## Re-running

`horizon-port-pack check <jar> ports/elevatorid/content` deletes and rewrites
`content/carry/`, `content/outside/` and the two report JSONs; the folder-root
`assets/`, `data/` and `horizon-mod.toml` are hand-owned and **not** touched.
A fresh carry means re-applying the three fixes (`apply_fixes.py` writes the 16
block declarations, the creative tab and the 16 `c:` tags), then
`horizon-mod build` / `horizon-mod check` again.
