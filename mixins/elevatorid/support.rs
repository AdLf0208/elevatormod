// Game-side glue for ElevatorMod: jump/sneak detection in `player_jump` /
// `player_toggle_sneak` / `player_tick`, and the native teleport executed in
// `server_tick`.
//
// The move is queued because the column scan reads blocks through the server
// tick's read scope (`native_server_block` takes the lifecycle context). It is
// then moved with the pin's `native_teleport_player(uuid, dimension, pos,
// yaw, pitch)` helper (pending fragment `teleport_player`), which `/tp`s a
// server player from any server hook: at once for the acting player or a tick
// hook, as this one. The previous `execute in <dimension> run tp …` console
// command is gone.
//
// Injected into `crate::game` as `elevatorid_state`; it needs the game, so it
// is not part of the standalone library (only `runtime.rs` is).

use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::game::elevatorid::{SAME_COLOR_DEFAULT, World, plan, travel_position};

/// One tick of cooldown between a player's teleports (upstream has none, but a
/// held jump would otherwise climb on every landing).
const COOLDOWN_TICKS: i64 = 10;

#[derive(Default)]
struct State {
    /// (shift down, on ground) at the previous player tick, by UUID.
    inputs: BTreeMap<u128, (bool, bool)>,
    /// A queued travel: true = jump (up), false = sneak (down).
    pending: BTreeMap<u128, bool>,
    /// The server tick of the player's last teleport.
    last_teleport: BTreeMap<u128, i64>,
    /// Space held at the previous client tick (jump edge).
    last_space: bool,
    /// Jump requests from the window's own input, by player name.
    pending_name: BTreeMap<String, bool>,
    /// Our own running server-tick count (`NativeTickId` has no accessor at
    /// the pin).
    now: i64,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

fn state() -> std::sync::MutexGuard<'static, Option<State>> {
    STATE.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// The window's tick: a Space press is the jump edge the server cannot see in
/// `player_tick` (raw input is not swapped in). The integrated server shares
/// this process, so the queue reaches `server_tick`.
#[allow(private_interfaces)]
pub fn on_client_tick(game: &mut crate::game::Game<'_>) {
    let space = game.keys.contains(&winit::keyboard::KeyCode::Space);
    let name = game.options.player_name();
    let mut guard = state();
    let state = guard.get_or_insert_with(State::default);
    let was = state.last_space;
    state.last_space = space;
    if space && !was {
        state.pending_name.insert(name, true);
    }
}

/// One server tick of the acting player: detect a jump take-off or a sneak
/// press (rising edges, as upstream) and queue the travel.
#[allow(private_interfaces)]
pub fn on_player_tick(game: &mut crate::game::Game<'_>) {
    let Some(player) = game.native_acting_player() else {
        return;
    };
    if player.game_mode == 3 {
        return;
    }
    let Some(motion) = game.player.as_ref() else {
        return;
    };
    let shift = motion.shift_key_down;
    let ground = motion.ground.on_ground;
    // The server's own take-off predicate (lan_actions.rs): it left the ground
    // on an upward move this tick. Velocity is post-collision here (a jump
    // clipped by a low ceiling reports a fall), the move is not.
    let rose = motion
        .history
        .records()
        .iter()
        .any(|record| record.to.y - record.from.y > 0.0);
    let jump = !ground && (motion.velocity.y > 0.05 || rose);
    let mut guard = state();
    let state = guard.get_or_insert_with(State::default);
    let (last_shift, last_ground) = state
        .inputs
        .get(&player.uuid)
        .copied()
        .unwrap_or((false, true));
    let jump_edge = jump && last_ground;
    let sneak_edge = shift && !last_shift;
    state.inputs.insert(player.uuid, (shift, ground));
    if jump_edge {
        state.pending.insert(player.uuid, true);
    } else if sneak_edge {
        state.pending.insert(player.uuid, false);
    }
}

/// A guest jumped from the ground (`player_jump`, API 344): queue the up
/// travel on the exact edge, so a jump clipped by a low ceiling is still seen
/// (the body-based predicate in `on_player_tick` misses it).
#[allow(private_interfaces)]
pub fn on_player_jump(game: &mut crate::game::Game<'_>, player: u128) {
    if game.native_acting_player().is_some_and(|p| p.game_mode == 3) {
        return;
    }
    queue(player, true);
}

/// A guest's sneak flag changed (`player_toggle_sneak`, API 351): only the
/// press (down) queues a travel; the release does not.
#[allow(private_interfaces)]
pub fn on_player_sneak(game: &mut crate::game::Game<'_>, player: u128, sneaking: bool) {
    if !sneaking || game.native_acting_player().is_some_and(|p| p.game_mode == 3) {
        return;
    }
    queue(player, false);
}

fn queue(uuid: u128, up: bool) {
    let mut guard = state();
    let state = guard.get_or_insert_with(State::default);
    state.pending.insert(uuid, up);
}

/// At the end of a running server tick: execute queued travels. The column is
/// read through the tick's read scope (`native_server_block`); the move is the
/// native player teleport, called here where no player's state is swapped in.
#[allow(private_interfaces)]
pub fn on_server_tick(
    game: &mut crate::game::Game<'_>,
    context: &crate::game::native_lifecycle::NativeLifecycleContext,
    phase: crate::game::native_lifecycle::ServerTickPhase,
) {
    let crate::game::native_lifecycle::ServerTickPhase::ServerTickEnd { world_running } = phase
    else {
        return;
    };
    if !world_running {
        return;
    }
    let (queued, named, tick) = {
        let mut guard = state();
        let state = guard.get_or_insert_with(State::default);
        state.now += 1;
        (
            std::mem::take(&mut state.pending),
            std::mem::take(&mut state.pending_name),
            state.now,
        )
    };
    if queued.is_empty() && named.is_empty() {
        return;
    }
    let mut work: Vec<(u128, bool)> = queued.into_iter().collect();
    for (name, up) in named {
        if let Some(player) = game
            .native_server_players()
            .into_iter()
            .find(|player| player.name == name)
        {
            work.push((player.uuid, up));
        }
    }
    for (uuid, up) in work {
        {
            let guard = state();
            if let Some(last) = guard.as_ref().and_then(|s| s.last_teleport.get(&uuid)) {
                if tick - last < COOLDOWN_TICKS {
                    continue;
                }
            }
        }
        let Some(player) = game
            .native_server_players()
            .into_iter()
            .find(|player| player.uuid == uuid)
        else {
            continue;
        };
        if player.game_mode == 3 {
            continue;
        }
        let [x, feet, z] = player.position;
        let bx = x.floor() as i32;
        let bz = z.floor() as i32;
        let dimension = player.dimension.clone();
        let target = {
            let mut world = ServerColumn {
                game,
                context,
                dimension: dimension.clone(),
                x: bx,
                z: bz,
            };
            plan(&mut world, feet, up, SAME_COLOR_DEFAULT)
        };
        let Some(target) = target else {
            continue;
        };
        // The helper `/tp`s as a command does, keeping the look angles: pass
        // them through so the travel does not turn the player.
        let pos = travel_position(bx, bz, target.feet_y);
        if game.native_teleport_player(uuid, &dimension, pos, player.yaw, player.pitch) {
            if let Some(state) = state().as_mut() {
                state.last_teleport.insert(uuid, tick);
            }
        }
    }
}

/// The player's column, read through the tick's server read scope.
struct ServerColumn<'a, 'g, 'c> {
    game: &'a mut crate::game::Game<'g>,
    context: &'c crate::game::native_lifecycle::NativeLifecycleContext,
    dimension: String,
    x: i32,
    z: i32,
}

impl World for ServerColumn<'_, '_, '_> {
    fn block(&mut self, y: i32) -> Option<String> {
        self.game
            .native_server_block(self.context, &self.dimension, [self.x, y, self.z])
            .ok()
            .map(|state| state.name)
    }

    fn suffocating(&mut self, y: i32) -> bool {
        // The unsuffixed collision helper reads the level in place, so only
        // use it when that level is the player's own dimension; otherwise the
        // dimension-explicit block read plus the content facts is the answer.
        if self.game.dim.id() == self.dimension {
            return self
                .game
                .native_block_collision([self.x, y, self.z])
                .iter()
                .any(|box_| {
                    box_[0] <= 0.001
                        && box_[1] <= 0.001
                        && box_[2] <= 0.001
                        && box_[3] >= 0.999
                        && box_[4] >= 0.999
                        && box_[5] >= 0.999
                });
        }
        let Some(name) = self.block(y) else {
            return false;
        };
        let Some(id) = self.game.pack.assets.registry.default_state(&name) else {
            return false;
        };
        let facts = self.game.res.facts();
        facts.get(id).full_collision && !facts.is_air(id)
    }
}
