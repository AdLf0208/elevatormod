// Runs at the head of `Game::native_tick` (API 4), once per tick in every Game:
// on the window the raw Space key is visible, so the jump edge is queued here.
fn observe_tick(game: &mut crate::game::Game<'_>) {
    crate::game::elevatorid_state::on_client_tick(game);
}
