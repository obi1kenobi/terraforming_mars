use std::collections::{BTreeMap, HashSet};

use maplit::btreemap;
use serde::{Deserialize, Serialize};

use crate::{
    card::{Card, CardAction, CardEffect, CardKind, CardTag},
    resource::{PaymentCost, Resource},
};

const CARD_PURCHASE_COST: usize = 3;
const DEFAULT_STARTING_TERRAFORM_RATING: usize = 20;
const DEFAULT_SOLO_STARTING_TERRAFORM_RATING: usize = 14;
const DEFAULT_STEEL_VALUE: usize = 2;
const DEFAULT_TITANIUM_VALUE: usize = 3;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerState {
    pub resources: BTreeMap<Resource, usize>,
    pub production: BTreeMap<Resource, isize>,
    pub played_cards: Vec<Card>,
    pub tapped_active_cards: HashSet<Card>,
    pub cards_in_hand: Vec<Card>,
    pub terraform_rating: usize,
    pub steel_value: usize,
    pub titanium_value: usize,
    pub effects: Vec<CardEffect>,
}

pub struct PlayerStateBuilder {
    pub resources: Option<BTreeMap<Resource, usize>>,
    pub production: Option<BTreeMap<Resource, isize>>,
    pub played_cards: Option<Vec<Card>>,
    pub tapped_active_cards: Option<HashSet<Card>>,
    pub cards_in_hand: Option<Vec<Card>>,
    pub terraform_rating: usize,
    pub effects: Option<Vec<CardEffect>>,
}

impl PlayerStateBuilder {
    pub fn new() -> PlayerStateBuilder {
        PlayerStateBuilder {
            resources: None,
            production: None,
            played_cards: None,
            tapped_active_cards: None,
            cards_in_hand: None,
            terraform_rating: DEFAULT_STARTING_TERRAFORM_RATING,
            effects: None,
        }
    }

    pub fn with_resources(
        mut self,
        megacredits: usize,
        steel: usize,
        titanium: usize,
        plants: usize,
        energy: usize,
        heat: usize,
    ) -> PlayerStateBuilder {
        assert!(self.resources.is_none());

        let resources = btreemap! {
            Resource::Megacredits => megacredits,
            Resource::Steel => steel,
            Resource::Titanium => titanium,
            Resource::Plants => plants,
            Resource::Energy => energy,
            Resource::Heat => heat,
        };

        self.resources = Some(resources);
        self
    }

    pub fn with_production(
        mut self,
        megacredits: isize,
        steel: isize,
        titanium: isize,
        plants: isize,
        energy: isize,
        heat: isize,
    ) -> PlayerStateBuilder {
        assert!(self.production.is_none());

        assert!(steel >= 0);
        assert!(titanium >= 0);
        assert!(plants >= 0);
        assert!(energy >= 0);
        assert!(heat >= 0);

        let production = btreemap! {
            Resource::Megacredits => megacredits,
            Resource::Steel => steel,
            Resource::Titanium => titanium,
            Resource::Plants => plants,
            Resource::Energy => energy,
            Resource::Heat => heat,
        };

        self.production = Some(production);
        self
    }

    pub fn build(self) -> PlayerState {
        let resources = self.resources.unwrap_or_else(
            || btreemap! {
                Resource::Megacredits => 0,
                Resource::Steel => 0,
                Resource::Titanium => 0,
                Resource::Plants => 0,
                Resource::Energy => 0,
                Resource::Heat => 0,
            }
        );

        let production = self.production.unwrap_or_else(
            || btreemap! {
                Resource::Megacredits => 0,
                Resource::Steel => 0,
                Resource::Titanium => 0,
                Resource::Plants => 0,
                Resource::Energy => 0,
                Resource::Heat => 0,
            }
        );

        PlayerState {
            resources,
            production,
            played_cards: self.played_cards.unwrap_or_default(),
            tapped_active_cards: self.tapped_active_cards.unwrap_or_default(),
            cards_in_hand: self.cards_in_hand.unwrap_or_default(),
            terraform_rating: self.terraform_rating,
            steel_value: DEFAULT_STEEL_VALUE,  // TODO: adjust for the advanced alloys effect
            titanium_value: DEFAULT_TITANIUM_VALUE,  // TODO: adjust for the advanced alloys effect
            effects: self.effects.unwrap_or_default(),
        }
    }
}

impl Default for PlayerStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerState {
    pub fn purchase_cards(&mut self, cards: &Vec<&Card>) -> Option<()> {
        let megacredits_balance = self.resources[&Resource::Megacredits];
        let megacredits_cost = cards.len() * CARD_PURCHASE_COST;

        if megacredits_balance < megacredits_cost {
            None
        } else {
            self.cards_in_hand.extend(cards.iter().copied().cloned());
            self.resources.insert(
                Resource::Megacredits,
                megacredits_balance - megacredits_cost,
            );
            Some(())
        }
    }

    pub fn play_card(&mut self, index_in_hand: usize) -> Option<PaymentCost> {
        let card = &self.cards_in_hand[index_in_hand];
        let megacredits_balance = self.resources[&Resource::Megacredits];

        let can_pay = match &card.cost {
            PaymentCost::Megacredits(x) => *x <= megacredits_balance,
            PaymentCost::Building(x) => {
                let steel_balance = self.resources[&Resource::Steel];

                *x <= (megacredits_balance + (steel_balance * self.steel_value))
            }
            PaymentCost::Space(x) => {
                let titanium_balance = self.resources[&Resource::Titanium];

                *x <= (megacredits_balance + (titanium_balance * self.titanium_value))
            }
            PaymentCost::SpaceOrBuilding(x) => {
                let steel_balance = self.resources[&Resource::Steel];
                let titanium_balance = self.resources[&Resource::Titanium];

                *x <= (megacredits_balance
                    + (steel_balance * self.steel_value)
                    + (titanium_balance * self.titanium_value))
            }
            _ => unreachable!(),
        };

        let satisfies_requirements = true; // TODO: implement me

        if satisfies_requirements && can_pay {
            let cloned_cost = card.cost.clone();

            self.effects.extend_from_slice(&card.effects);
            // TODO: resolve immediate impacts

            let owned_card = self.cards_in_hand.swap_remove(index_in_hand);
            self.played_cards.push(owned_card);

            Some(cloned_cost)
        } else {
            None
        }
    }

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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarsBoard {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurnAction {
    PlayStandardProject,
    PlayCard(Card),
    PerformAction(CardAction),
    ClaimMilestone,
    FundAward,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerTurn {
    Play(TurnAction, Option<TurnAction>),
    Pass,
}
