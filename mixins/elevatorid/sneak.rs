// Runs at the head of `Game::native_player_toggle_sneak` (API 351): the
// guest's sneak flag changed, after the hub applied it. Only the press (down)
// starts a travel; the release does not.
fn observe_sneak_change(game: &mut crate::game::Game<'_>, player: u128, sneaking: bool) {
    crate::game::elevatorid_state::on_player_sneak(game, player, sneaking);
}
