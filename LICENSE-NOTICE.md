# Licence notice — ElevatorMod for Horizon

- **Upstream:** ElevatorMod 1.21.1-1.11.4, <https://github.com/VsnGamer/ElevatorMod>
  (Modrinth project `hi2dSXTu`, version `neoforge-1.21.1-1.11.4`).
- **Licence as read from the artefact:** MIT — the jar's `META-INF/neoforge.mods.toml` declares
  `license = "MIT"` and the upstream licence text is the MIT licence,
  `Copyright (c) 2016 Vasco`. The jar kept as evidence is
  `../_shared/carry-work/elevatorid/elevatorid-neoforge-1.21.1-1.11.4.jar`
  (SHA-1 `87a5504efcd51de55cd4215a103cbfcf717fa022`).
- **Complete licence text:** [`LICENSE`](LICENSE), which keeps the original copyright notice
  beside the port's own.
- **What travels with the artifact:** `LICENSE` and this file are at the root of the built
  `elevatorid.hmod`, beside `assets/elevatorid/` and `data/elevatorid/` — the sixteen blocks,
  their items, models, textures, recipes, loot tables, tags, sounds and translations carried from
  the jar. An artifact-only download therefore carries the MIT notice (MIT's notice condition).
- **The rewrite:** `src/` is a clean-room Rust rewrite of the mod's behaviour from its public
  source; no Java source is translated or shipped.
- **Status:** local-only until the maintainer approves publishing (ARCHITECTURE.md §8).
