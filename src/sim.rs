use crate::{card::Card, game::{PlayerState, PlayerTurn}};

#[derive(Clone, Debug)]
pub struct SimState {

}

pub fn get_possible_generation_plays(
    initial_state: &PlayerState,
    offered_cards: Vec<Card>,
) -> Vec<(Vec<Card>, Vec<PlayerTurn>, PlayerState)> {
    // return vec of (unbought cards, taken turns, final state)

    vec![(offered_cards, vec![PlayerTurn::Pass], initial_state.clone())]
}
