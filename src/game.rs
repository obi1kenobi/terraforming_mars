use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    card::{Card, CardAction, CardKind, CardTag},
    resource::Resource,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerState {
    resources: BTreeMap<Resource, usize>,
    production: BTreeMap<Resource, isize>,
    played_cards: Vec<Card>,
    tapped_active_cards: HashSet<Card>,
    cards_in_hand: Vec<Card>,
    terraform_rating: usize,
}

impl PlayerState {
    pub fn played_event_count(&self) -> usize {
        self.played_cards
            .iter()
            .filter(|card| card.kind == CardKind::Event)
            .count()
    }

    pub fn active_tag_count(&self, tag_kind: CardTag) -> usize {
        assert_ne!(tag_kind, CardTag::Event);
        self.get_played_non_event_tags()
            .filter(|&tag| tag == tag_kind)
            .count()
    }

    pub fn active_tag_count_for_action(&self, tag_kind: CardTag) -> usize {
        // Wild tags only count for the purposes of performing actions.
        assert_ne!(tag_kind, CardTag::Event);
        self.get_played_non_event_tags()
            .filter(|&tag| tag == tag_kind || tag == CardTag::Wild)
            .count()
    }

    fn get_played_non_event_tags(&self) -> impl Iterator<Item = CardTag> + '_ {
        self.played_cards.iter().flat_map(|card| match card.kind {
            CardKind::Event => [].iter().copied(),
            _ => card.tags.iter().copied(),
        })
    }

    pub fn advance_generation(&mut self) {
        let mut new_resources = self.resources.clone();

        // All energy becomes heat.
        let current_energy = new_resources[&Resource::Energy];
        new_resources
            .entry(Resource::Heat)
            .and_modify(|val| *val += current_energy);
        new_resources.insert(Resource::Energy, 0);

        // Gain credits equal to the terraform rating.
        new_resources
            .entry(Resource::Megacredits)
            .and_modify(|val| *val += self.terraform_rating);

        // Gain resources according to production.
        for (key, production) in self.production.iter() {
            new_resources.entry(*key).and_modify(|val| {
                let new_val = *val as isize + production;
                assert!(new_val >= 0);
                *val = new_val as usize;
            });
        }

        self.resources = new_resources;
        self.tapped_active_cards.clear();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarsBoard {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TurnAction {
    PlayStandardProject,
    PlayCard(Card),
    PerformAction(CardAction),
    ClaimMilestone,
    FundAward,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlayerTurn {
    Play(TurnAction, Option<TurnAction>),
    Pass,
}