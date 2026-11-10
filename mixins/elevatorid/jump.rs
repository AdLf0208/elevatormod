// Runs at the head of `Game::native_player_jump` (API 344): an accepted
// movement made the server player jump from the ground. This is the exact
// jump edge, so a jump clipped by a low ceiling is still reported.
fn observe_player_jump(game: &mut crate::game::Game<'_>, player: u128) {
    crate::game::elevatorid_state::on_player_jump(game, player);
}
