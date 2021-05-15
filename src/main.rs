#![allow(dead_code)]

use card::Card;
use game::PlayerState;

use crate::game::PlayerTurn;

mod card;
mod game;
mod resource;

pub(crate) fn get_possible_generation_plays(
    initial_state: &PlayerState,
    offered_cards: Vec<Card>,
) -> Vec<(Vec<Card>, Vec<PlayerTurn>, PlayerState)> {
    // return vec of (unbought cards, taken turns, final state)

    vec![(offered_cards, vec![PlayerTurn::Pass], initial_state.clone())]
}

pub fn main() {}
