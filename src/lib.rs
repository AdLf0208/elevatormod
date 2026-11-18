//! ElevatorMod for Horizon — jump to ride an elevator column up, sneak to ride it down.
//!
//! Ported from ElevatorMod 1.21.1-1.11.4 (VsnGamer/ElevatorMod, MIT) for Minecraft 26.3, from the
//! upstream source read for behaviour only: this is a clean-room Rust rewrite.
//!
//! [`runtime`] is the pure planner with its tests. [`support`] is the glue: the travel queue, the
//! column reads through the level handle and the player teleport. This file is the one
//! `impl mod_api::Hooks`, which the macro requires at the crate root.
//!
//! # Where the retired `.hnmod` container's hooks went
//!
//! Horizon's mod-API merge (#16941, launcher ≥ 0.2.70) retired `.hnmod`: the plan, its injection
//! points and the per-hook helper files are gone. Each old `[[injection]]` is now a typed method:
//!
//! | `.hnmod` hook (API) | here |
//! |---|---|
//! | `player_jump` (344) → `jump.rs` | [`Mod::player_jump`] → [`support::on_jump`] |
//! | `player_toggle_sneak` (351) → `sneak.rs` | [`Mod::player_toggle_sneak`] → [`support::on_sneak`] |
//! | `server_tick` (4) → `server.rs` | [`Mod::server_tick`] → [`support::on_server_tick`] |
//! | `content_root` HEAD → `embedded_content.rs` | gone: the flat `.hmod` carries `assets/` and `data/` itself |
//! | `player_tick` (31) → `player.rs` | dropped: it was the body-predicate fallback for the jump edge, which `player_jump` now reports exactly |
//! | `tick` (4) → `tick.rs` | dropped: it was the window's raw Space edge, the same fallback on the client side |
pub mod runtime;
pub mod support;

use std::sync::Mutex;

use mod_api::{self, Hooks, PlayerJumpEvent, PlayerToggleSneakEvent, ServerHost, ServerTickEvent};

use crate::support::Travel;

/// The session's state: one travel queue for every player, alive as long as the mod is loaded.
#[derive(Default)]
struct Mod {
    travel: Mutex<Travel>,
}

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// A guest's accepted movement made the server jump them from the ground: the exact up edge.
    fn player_jump(&self, host: &mut dyn ServerHost, event: &mut PlayerJumpEvent<'_>) {
        support::on_jump(&self.travel, host, event.player);
    }

    /// A guest's sneaking flag changed after the hub applied it: only the press rides down.
    fn player_toggle_sneak(
        &self,
        host: &mut dyn ServerHost,
        event: &mut PlayerToggleSneakEvent<'_>,
    ) {
        support::on_sneak(&self.travel, host, event.player, event.sneaking);
    }

    /// The end of a running server tick: read each queued player's column and ride it.
    fn server_tick(&self, host: &mut dyn ServerHost, event: &mut ServerTickEvent<'_>) {
        support::on_server_tick(&self.travel, host, event);
    }
}
