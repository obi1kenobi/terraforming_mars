use crate::{
    card::Card,
    game::{PlayerState, PlayerTurn, TurnAction},
};

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
    initial_state: &PlayerState,
    opponent_states: &Vec<&PlayerState>,
) -> Vec<(Vec<PlayerTurn>, PlayerState)> {
    let mut next_card_index_to_consider: usize = 0;

    _make_all_possible_plays_recursively(&mut next_card_index_to_consider, initial_state, opponent_states)
}

fn _make_all_possible_plays_recursively(
    next_card_index_to_consider: &mut usize,
    initial_state: &PlayerState,
    opponent_states: &Vec<&PlayerState>,
) -> Vec<(Vec<PlayerTurn>, PlayerState)> {
    match initial_state.cards_in_hand.get(*next_card_index_to_consider) {
        None => {
            vec![(vec![], initial_state.clone())]
        }
        Some(card) => {
            let mut state = initial_state.clone();

            let play_vector = match state.play_card(*next_card_index_to_consider) {
                None => {
                    vec![]
                }
                Some(_) => {
                    vec![PlayerTurn::Play(TurnAction::PlayCard(card.clone()), None)]
                }
            };

            let mut result = vec![];
            *next_card_index_to_consider += 1;
            for (mut moves, final_state) in
                _make_all_possible_plays_recursively(next_card_index_to_consider, &state, opponent_states)
            {
                let mut final_plays = play_vector.clone();
                final_plays.extend(moves.drain(..));
                result.push((final_plays, final_state));
            }

            *next_card_index_to_consider -= 1;
            result
        }
    }
}
