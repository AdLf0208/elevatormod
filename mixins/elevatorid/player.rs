// Runs at the head of `Game::native_player_tick` (API 31): detect a jump
// take-off or a sneak press and queue the travel.
fn observe_player_tick(game: &mut crate::game::Game<'_>) {
    crate::game::elevatorid_state::on_player_tick(game);
}
