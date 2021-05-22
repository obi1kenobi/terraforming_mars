use crate::{
    card::Card,
    game::{PlayerState, TurnAction},
};

pub fn get_possible_generation_plays(
    initial_state: &PlayerState,
    opponent_states: &Vec<&PlayerState>,
    offered_cards: Vec<Card>,
) -> Vec<(Vec<Card>, Vec<TurnAction>, PlayerState)> {
    // return vec of (bought cards, actions taken, final state)
    let mut result: Vec<(Vec<Card>, Vec<TurnAction>, PlayerState)> = Vec::with_capacity(1024);

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
) -> Vec<(Vec<TurnAction>, PlayerState)> {
    let mut next_card_index_to_consider: usize = 0;

    make_all_possible_plays_recursively(
        &mut next_card_index_to_consider,
        initial_state,
        opponent_states,
    )
}

fn make_all_possible_plays_recursively(
    next_card_index_to_consider: &mut usize,
    initial_state: &PlayerState,
    opponent_states: &Vec<&PlayerState>,
) -> Vec<(Vec<TurnAction>, PlayerState)> {
    match initial_state
        .cards_in_hand
        .get(*next_card_index_to_consider)
    {
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
                    vec![TurnAction::PlayCard(card.clone())]
                }
            };

            let mut result = vec![];
            *next_card_index_to_consider += 1;
            for (mut moves, final_state) in make_all_possible_plays_recursively(
                next_card_index_to_consider,
                &state,
                opponent_states,
            ) {
                let mut final_plays = play_vector.clone();
                final_plays.extend(moves.drain(..));
                result.push((final_plays, final_state));
            }

            *next_card_index_to_consider -= 1;
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{card::{BASE_GAME_CARDS_BY_NAME, Card}, game::{PlayerState, TurnAction}, sim::get_possible_generation_plays};
    use crate::game::PlayerStateBuilder;

    #[test]
    fn get_possible_plays_when_no_card_buys_or_plays_exist() {
        let player_state = PlayerStateBuilder::new()
            .with_resources(2, 0, 0, 0, 0, 0)
            .build();

        let offered_cards = vec![
            BASE_GAME_CARDS_BY_NAME["Fueled Generators"],
            BASE_GAME_CARDS_BY_NAME["Nuclear Power"],
            BASE_GAME_CARDS_BY_NAME["Solar Power"],
            BASE_GAME_CARDS_BY_NAME["GHG Factories"],
        ];

        let opponent_state = PlayerStateBuilder::new().build();
        let opponent_states = vec![&opponent_state];

        let expected_plays: Vec<(Vec<Card>, Vec<TurnAction>, PlayerState)> = vec![
            (vec![], vec![], player_state.clone()),
        ];

        let actual_plays = get_possible_generation_plays(
            &player_state, &opponent_states, offered_cards.iter().copied().cloned().collect());

        assert_eq!(expected_plays, actual_plays);
    }
}
