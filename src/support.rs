//! Game-side glue for ElevatorMod: the travel queue, the column reads and the player teleport.
//!
//! The rules themselves live in [`crate::runtime`] (pure, no `mod_api` types). This module is the
//! adapter: a jump or a sneak press queues a travel for one player's UUID, and at the end of a
//! running server tick the queue is drained — the player's column is read through its level
//! handle, the planner picks the target, and the player is moved with the SDK's teleport.
//!
//! The move happens in `server_tick`, not in the input hook, because a player hook runs with that
//! guest's state swapped in; a tick hook is where the game lets a mod move a player.

use std::collections::BTreeMap;
use std::sync::Mutex;

use mod_api::{GameMode, Host, LevelRef, ServerHost, ServerTickEvent, ServerTickPhase};

use crate::runtime::{SAME_COLOR_DEFAULT, World, plan, travel_position};

/// One tick of cooldown between a player's travels. Upstream has none, but a held jump would
/// otherwise climb again on landing at the target (the retired 0.1.0 bundle's constant).
pub const COOLDOWN_TICKS: u64 = 10;

/// The travels queued this tick, and when each player last travelled.
#[derive(Default)]
pub struct Travel {
    /// A queued travel: true = jump (up), false = sneak (down).
    pending: BTreeMap<u128, bool>,
    /// The server tick of each player's last teleport.
    last_teleport: BTreeMap<u128, u64>,
}

impl Travel {
    /// Queue one travel. A second edge before the tick is drained replaces the first: the queue is
    /// keyed by player, so one press is one travel.
    pub fn queue(&mut self, player: u128, up: bool) {
        self.pending.insert(player, up);
    }

    /// Take everything queued, in UUID order, leaving the queue empty.
    pub fn take(&mut self) -> Vec<(u128, bool)> {
        std::mem::take(&mut self.pending).into_iter().collect()
    }

    /// Whether `player` may travel at `tick`, given its last travel.
    pub fn ready(&self, player: u128, tick: u64) -> bool {
        self.last_teleport
            .get(&player)
            .is_none_or(|last| tick.saturating_sub(*last) >= COOLDOWN_TICKS)
    }

    /// Remember that `player` travelled at `tick`.
    pub fn record(&mut self, player: u128, tick: u64) {
        self.last_teleport.insert(player, tick);
    }
}

/// A player jumped from the ground: queue the ride up. This is the server's exact jump edge, so a
/// jump clipped by a low ceiling is still one.
pub fn on_jump(travel: &Mutex<Travel>, host: &dyn Host, player: u128) {
    if is_spectator(host, player) {
        return;
    }
    lock(travel).queue(player, true);
}

/// A player's sneaking flag changed: only the press (down) queues a ride down, not the release.
pub fn on_sneak(travel: &Mutex<Travel>, host: &dyn Host, player: u128, sneaking: bool) {
    if !sneaking || is_spectator(host, player) {
        return;
    }
    lock(travel).queue(player, false);
}

/// At the end of a running server tick, ride every queued travel.
pub fn on_server_tick(
    travel: &Mutex<Travel>,
    host: &mut dyn ServerHost,
    event: &ServerTickEvent<'_>,
) {
    let ServerTickPhase::ServerTickEnd { world_running } = event.phase else {
        return;
    };
    if !world_running {
        return;
    }
    let tick = event.context.tick.get();
    let queued = lock(travel).take();
    for (uuid, up) in queued {
        if !lock(travel).ready(uuid, tick) {
            continue;
        }
        let Some(player) = host.player(uuid) else {
            continue;
        };
        if player.game_mode(host) == Some(GameMode::Spectator) {
            continue;
        }
        let entity = player.entity();
        let Some([x, feet, z]) = entity.position(host) else {
            continue;
        };
        let Some(level) = entity.level(host) else {
            continue;
        };
        let rotation = entity.rotation(host);
        let block_x = x.floor() as i32;
        let block_z = z.floor() as i32;
        let target = {
            // The column is the player's own level, which the server is ticking: its block reads
            // and collision boxes are live here (a parked level answers neither reliably).
            let mut column = Column {
                host,
                level,
                x: block_x,
                z: block_z,
            };
            plan(&mut column, feet, up, SAME_COLOR_DEFAULT)
        };
        let Some(target) = target else {
            continue;
        };
        // The SDK's teleport `/tp`s as a command does, keeping the look angles: pass them through
        // so the ride does not turn the player.
        let pos = travel_position(block_x, block_z, target.feet_y);
        if entity.teleport_to(host, pos, rotation) {
            lock(travel).record(uuid, tick);
        }
    }
}

/// Whether the player is a spectator, whom upstream does not move.
fn is_spectator(host: &dyn Host, player: u128) -> bool {
    host.player(player)
        .and_then(|player| player.game_mode(host))
        == Some(GameMode::Spectator)
}

/// The travel queue's lock. A hook body never panics while holding it, so poison is a bug.
fn lock(travel: &Mutex<Travel>) -> std::sync::MutexGuard<'_, Travel> {
    travel.lock().expect("the travel queue's lock is poisoned")
}

/// One block column around the player, read through its level handle.
struct Column<'a> {
    host: &'a dyn Host,
    level: LevelRef,
    x: i32,
    z: i32,
}

impl World for Column<'_> {
    fn block(&mut self, y: i32) -> Option<String> {
        self.level
            .block(self.host, [self.x, y, self.z])
            .map(|key| key.id(self.host))
    }

    fn suffocating(&mut self, y: i32) -> bool {
        // `isSuffocating` here is "a collision box covers the whole cell", as the retired bundle
        // asked the content facts; the level's own collision shapes answer it exactly. A level
        // that has no answer (not resident) has no suffocation to report.
        let (x, y, z) = (f64::from(self.x), f64::from(y), f64::from(self.z));
        let cell = [x, y, z, x + 1.0, y + 1.0, z + 1.0];
        self.level
            .collision_boxes(self.host, cell)
            .is_some_and(|boxes| {
                boxes.iter().any(|b| {
                    b[0] <= x + 0.001
                        && b[1] <= y + 0.001
                        && b[2] <= z + 0.001
                        && b[3] >= x + 0.999
                        && b[4] >= y + 0.999
                        && b[5] >= z + 0.999
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAYER: u128 = 7;

    #[test]
    fn a_queued_travel_is_taken_once() {
        let mut travel = Travel::default();
        travel.queue(PLAYER, true);
        assert_eq!(travel.take(), vec![(PLAYER, true)]);
        assert!(travel.take().is_empty());
    }

    #[test]
    fn a_repeat_edge_before_the_tick_is_one_travel() {
        let mut travel = Travel::default();
        travel.queue(PLAYER, true);
        travel.queue(PLAYER, false);
        assert_eq!(travel.take(), vec![(PLAYER, false)], "the last edge wins");
    }

    #[test]
    fn travels_are_taken_in_uuid_order() {
        let mut travel = Travel::default();
        travel.queue(9, true);
        travel.queue(3, false);
        assert_eq!(travel.take(), vec![(3, false), (9, true)]);
    }

    #[test]
    fn a_travel_waits_out_the_cooldown() {
        let mut travel = Travel::default();
        assert!(travel.ready(PLAYER, 0), "no travel yet");
        travel.record(PLAYER, 100);
        assert!(
            !travel.ready(PLAYER, 109),
            "9 ticks later is inside the cooldown"
        );
        assert!(
            travel.ready(PLAYER, 110),
            "10 ticks later is the first allowed tick"
        );
    }

    #[test]
    fn one_players_cooldown_does_not_block_another() {
        let mut travel = Travel::default();
        travel.record(PLAYER, 100);
        assert!(travel.ready(8, 101));
    }
}
