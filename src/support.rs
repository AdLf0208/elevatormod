//! Game-side glue for ElevatorMod: the travel queue, the column reads and the player teleport.
//!
//! The rules themselves live in [`crate::runtime`] and [`crate::arrival`] (pure, no `mod_api`
//! types). This module is the adapter: a jump or a sneak press queues a travel for one player's
//! UUID, and at the end of a running server tick the queue is drained — the player's column is
//! read through its level handle, the planner picks the target, and the player is moved with the
//! SDK's teleport, turned to the target's facing when the target is directional.
//!
//! The move happens in `server_tick`, not in the input hook, because a player hook runs with that
//! guest's state swapped in; a tick hook is where the game lets a mod move a player.

use std::collections::BTreeMap;
use std::sync::Mutex;

use mod_api::{
    BlockShapeKind, GameMode, Host, LevelRef, ServerHost, ServerTickEvent, ServerTickPhase,
};

use crate::arrival::{Facing, Look, arrival};
use crate::runtime::{Target, World, plan, travel_position};
/// One tick of cooldown between a player's travels. Upstream has none, but a held jump would
/// otherwise climb again on landing at the target (the retired 0.1.0 bundle's constant).
pub const COOLDOWN_TICKS: u64 = 10;

/// Whether the live ride only lands on an elevator of the origin's own colour.
///
/// The registry entry and this port's README promise same-colour travel, so the live path
/// enforces it even though `runtime::SAME_COLOR_DEFAULT` keeps upstream's configurable off
/// default for the planner's own tests. Public and private landings (a shop floor and a private
/// room's floor) can share a column, and a colour-blind ride would let a player into the wrong
/// one.
pub const SAME_COLOR: bool = true;

/// Plan the ride the queue will actually take, with this port's own colour rule.
fn plan_live(world: &mut impl World, feet: f64, up: bool) -> Option<Target> {
    plan(world, feet, up, SAME_COLOR)
}

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
            plan_live(&mut column, feet, up)
        };
        let Some(target) = target else {
            continue;
        };
        // The SDK's teleport `/tp`s as a command does and turns when given an angle: a plain
        // landing keeps the view (pass the player's own angles through), a directional one turns
        // to the target's facing and levels the pitch, as upstream's server half does.
        let pos = travel_position(block_x, block_z, target.feet_y);
        let landing = landing_look(level, host, [block_x, target.elevator_y, block_z], rotation);
        if entity.teleport_to(host, pos, landing) {
            lock(travel).record(uuid, tick);
        }
    }
}

/// The view the teleport should leave the player with on the elevator at `at`.
///
/// The two states are read from the block itself, block-local (`LevelRef::block_property`), so
/// this works at `ServerTickEnd` where the level can be parked. A directional elevator with a
/// facing that does not parse keeps the view rather than inventing a direction.
fn landing_look(
    level: LevelRef,
    host: &dyn Host,
    at: [i32; 3],
    rotation: Option<[f32; 2]>,
) -> Option<[f32; 2]> {
    let [yaw, pitch] = rotation?;
    let directional = level.block_property(host, at, "directional").as_deref() == Some("true");
    let facing = if directional {
        level
            .block_property(host, at, "facing")
            .as_deref()
            .and_then(Facing::parse)
    } else {
        None
    };
    let look = arrival(facing, Look { yaw, pitch });
    Some([look.yaw, look.pitch])
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
        // asked the content facts. The state's own collision shape is block-local and needs no
        // resident level (`level.collision_boxes` refuses a parked level, and this runs at
        // ServerTickEnd where a level can be parked, which made every cell read clear).
        let at = [self.x, y, self.z];
        let Some(state) = self.level.block_state(self.host, at) else {
            return false;
        };
        self.level
            .block_shape(self.host, state, at, BlockShapeKind::Collision)
            .is_some_and(|boxes| {
                boxes.iter().any(|b| {
                    b[0] <= 0.001
                        && b[1] <= 0.001
                        && b[2] <= 0.001
                        && b[3] >= 0.999
                        && b[4] >= 0.999
                        && b[5] >= 0.999
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

    /// A column with the given `(y, block)` entries and nothing suffocating.
    struct Column(Vec<(i32, &'static str)>);

    impl World for Column {
        fn block(&mut self, y: i32) -> Option<String> {
            Some(
                self.0
                    .iter()
                    .find(|(at, _)| *at == y)
                    .map_or("minecraft:air", |(_, name)| *name)
                    .to_owned(),
            )
        }
        fn suffocating(&mut self, _: i32) -> bool {
            false
        }
    }

    /// The live ride must land on the origin's colour, not on the first elevator in the column:
    /// a red landing between two white ones is someone else's floor.
    #[test]
    fn the_live_ride_lands_on_the_same_colour_and_skips_another() {
        let mut column = Column(vec![
            (63, "elevatorid:elevator_white"),
            (66, "elevatorid:elevator_red"),
            (69, "elevatorid:elevator_white"),
        ]);
        assert_eq!(
            plan_live(&mut column, 64.0, true).map(|target| target.elevator_y),
            Some(69),
            "the red elevator at 66 must be skipped"
        );
        assert_eq!(
            plan_live(&mut column, 64.0, false).map(|target| target.elevator_y),
            None,
            "nothing of the same colour is below"
        );
    }
}
