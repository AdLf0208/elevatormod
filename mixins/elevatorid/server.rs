// Runs at the head of `Game::native_server_tick` (API 4): execute queued
// travels at the end of a running tick through the native player teleport
// (`native_teleport_player`), where no player's state is swapped in.
fn observe_server_tick(
    game: &mut crate::game::Game<'_>,
    context: &NativeLifecycleContext,
    phase: ServerTickPhase,
) {
    crate::game::elevatorid_state::on_server_tick(game, context, phase);
}
