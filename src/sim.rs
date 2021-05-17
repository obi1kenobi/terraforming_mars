use crate::{
    card::Card,
    game::{PlayerState, PlayerTurn},
};

#[derive(Clone, Debug)]
pub struct SimState {}

pub fn get_possible_generation_plays(
    initial_state: &PlayerState,
    opponent_states: &Vec<&PlayerState>,
    offered_cards: Vec<Card>,
) -> Vec<(Vec<Card>, Vec<PlayerTurn>, PlayerState)> {
    // return vec of (bought cards, taken turns, final state)

    let mut result: Vec<(Vec<Card>, Vec<PlayerTurn>, PlayerState)> = Vec::with_capacity(1024);

    assert!(offered_cards.len() <= 10);
    let all_variations = 1usize << offered_cards.len();

    for variation in 0..all_variations {
        let mut current_state = initial_state.clone();
        let mut purchased_cards: Vec<&Card> = Vec::new();
        for (i, card) in offered_cards.iter().enumerate() {
            if variation & (1usize << i) != 0 {
                purchased_cards.push(card);
            }
        }

        match current_state.purchase_cards(&purchased_cards) {
            None => continue,
            Some(_) => {
                let mut possible_plays = make_all_possible_plays(&current_state, opponent_states);

                result.extend(possible_plays.drain(..).map(|(turns, final_state)| {
                    (
                        purchased_cards.drain(..).cloned().collect(),
                        turns,
                        final_state,
                    )
                }));
            }
        }
    }

    result
}

fn make_all_possible_plays(
    _initial_state: &PlayerState,
    _opponent_states: &Vec<&PlayerState>,
) -> Vec<(Vec<PlayerTurn>, PlayerState)> {
    todo!()
}
