use std::collections::{BTreeMap, HashSet};

use maplit::btreemap;
use serde::{Deserialize, Serialize};

use crate::{
    card::{Card, CardAction, CardEffect, CardKind, CardTag, VictoryPointValue},
    resource::{CardResource, PaymentCost, Resource},
};

const CARD_PURCHASE_COST: usize = 3;
const DEFAULT_STARTING_TERRAFORM_RATING: usize = 20;
const DEFAULT_SOLO_STARTING_TERRAFORM_RATING: usize = 14;
const DEFAULT_STEEL_VALUE: usize = 2;
const DEFAULT_TITANIUM_VALUE: usize = 3;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerState {
    // primary data
    pub resources: BTreeMap<Resource, usize>,
    pub production: BTreeMap<Resource, isize>,
    pub played_cards: Vec<Card>,
    pub card_resources: BTreeMap<(Card, CardResource), usize>,
    pub tapped_active_cards: HashSet<Card>,
    pub cards_in_hand: Vec<Card>,
    pub terraform_rating: usize,
    pub steel_value: usize,
    pub titanium_value: usize,
    pub city_count: usize,

    // indexes of primary data
    pub effects: Vec<CardEffect>,
    pub current_total_points: isize, // total victory points from all sources: cards, TR, etc.
}

pub struct PlayerStateBuilder {
    pub resources: Option<BTreeMap<Resource, usize>>,
    pub production: Option<BTreeMap<Resource, isize>>,
    pub played_cards: Option<Vec<Card>>,
    pub card_resources: BTreeMap<(Card, CardResource), usize>,
    pub tapped_active_cards: Option<HashSet<Card>>,
    pub cards_in_hand: Option<Vec<Card>>,
    pub terraform_rating: usize,
    pub city_count: usize,
}

impl PlayerStateBuilder {
    pub fn new() -> PlayerStateBuilder {
        PlayerStateBuilder {
            resources: None,
            production: None,
            played_cards: None,
            card_resources: btreemap! {},
            tapped_active_cards: None,
            cards_in_hand: None,
            terraform_rating: DEFAULT_STARTING_TERRAFORM_RATING,
            city_count: 0,
        }
    }

    pub fn with_cities(mut self, city_count: usize) -> PlayerStateBuilder {
        self.city_count = city_count;
        self
    }

    pub fn with_played_cards(mut self, played_cards: Vec<Card>) -> PlayerStateBuilder {
        self.played_cards = Some(played_cards);
        self
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
        let city_count = self.city_count;
        let card_resources = self.card_resources;

        let resources = self.resources.unwrap_or_else(|| {
            btreemap! {
                Resource::Megacredits => 0,
                Resource::Steel => 0,
                Resource::Titanium => 0,
                Resource::Plants => 0,
                Resource::Energy => 0,
                Resource::Heat => 0,
            }
        });

        let production = self.production.unwrap_or_else(|| {
            btreemap! {
                Resource::Megacredits => 0,
                Resource::Steel => 0,
                Resource::Titanium => 0,
                Resource::Plants => 0,
                Resource::Energy => 0,
                Resource::Heat => 0,
            }
        });

        let mut current_total_points = self.terraform_rating as isize;
        let card_points: isize = self
            .played_cards
            .as_ref()
            .map(|cards| {
                cards
                    .iter()
                    .map(|c| match c.points {
                        Some(VictoryPointValue::Immediate(x)) => x,
                        Some(VictoryPointValue::PerTag(vp, count, tag)) => {
                            assert!(tag != CardTag::Event);

                            let tag_count = cards.active_tag_count(tag);
                            ((tag_count / count) * vp) as isize
                        }
                        Some(VictoryPointValue::PerCardResource(vp, count, cr)) => {
                            let resources_present = card_resources.get(&(c.clone(), cr)).copied().unwrap_or_default();

                            ((resources_present / count) * vp) as isize
                        }
                        Some(VictoryPointValue::FixedPointsIfAnyCardResourcePresent(
                            count,
                            cr,
                        )) => {
                            let resources_present = card_resources.get(&(c.clone(), cr)).copied().unwrap_or_default();
                            if resources_present > 0 {
                                count as isize
                            } else {
                                0
                            }
                        }
                        Some(VictoryPointValue::PerCity(vp)) => {
                            (city_count * vp) as isize
                        }
                        Some(VictoryPointValue::PerNCities(n_cities)) => {
                            (city_count / n_cities) as isize
                        }
                        None => 0,
                    })
                    .sum()
            })
            .unwrap_or_default();
        current_total_points += card_points;

        let effects: Vec<_> = self
            .played_cards
            .as_ref()
            .map(|cards| cards.iter().flat_map(|c| c.effects.clone()).collect())
            .unwrap_or_default();

        PlayerState {
            resources,
            production,
            played_cards: self.played_cards.unwrap_or_default(),
            card_resources: card_resources,
            tapped_active_cards: self.tapped_active_cards.unwrap_or_default(),
            cards_in_hand: self.cards_in_hand.unwrap_or_default(),
            terraform_rating: self.terraform_rating,
            steel_value: DEFAULT_STEEL_VALUE, // TODO: adjust for the advanced alloys effect
            titanium_value: DEFAULT_TITANIUM_VALUE, // TODO: adjust for the advanced alloys effect
            city_count: self.city_count,
            effects: effects,
            current_total_points: current_total_points,
        }
    }
}

impl Default for PlayerStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

trait ActiveTags {
    fn active_tag_count(&self, tag_kind: CardTag) -> usize;
    fn active_tag_count_for_action(&self, tag_kind: CardTag) -> usize;
    fn event_count(&self) -> usize;
    fn get_non_event_tags(&self) -> Box<dyn Iterator<Item = CardTag> + '_>;
}

impl ActiveTags for Vec<Card> {
    fn event_count(&self) -> usize {
        self.iter()
            .filter(|card| card.kind == CardKind::Event)
            .count()
    }

    fn active_tag_count(&self, tag_kind: CardTag) -> usize {
        assert_ne!(tag_kind, CardTag::Event);
        self.get_non_event_tags()
            .filter(|&tag| tag == tag_kind)
            .count()
    }

    fn active_tag_count_for_action(&self, tag_kind: CardTag) -> usize {
        // Wild tags only count for the purposes of performing actions.
        assert_ne!(tag_kind, CardTag::Event);
        self.get_non_event_tags()
            .filter(|&tag| tag == tag_kind || tag == CardTag::Wild)
            .count()
    }

    fn get_non_event_tags(&self) -> Box<dyn Iterator<Item = CardTag> + '_> {
        Box::new(self.iter().flat_map(|card| match card.kind {
            CardKind::Event => [].iter().copied(),
            _ => card.tags.iter().copied(),
        }))
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

    pub fn can_play_card(&self, index_in_hand: usize) -> Option<PaymentCost> {
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
            Some(cloned_cost)
        } else {
            None
        }
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

impl ActiveTags for PlayerState {
    fn event_count(&self) -> usize {
        self.played_cards.event_count()
    }

    fn active_tag_count(&self, tag_kind: CardTag) -> usize {
        self.played_cards.active_tag_count(tag_kind)
    }

    fn active_tag_count_for_action(&self, tag_kind: CardTag) -> usize {
        self.played_cards.active_tag_count_for_action(tag_kind)
    }

    fn get_non_event_tags(&self) -> Box<dyn Iterator<Item = CardTag> + '_> {
        self.played_cards.get_non_event_tags()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarsBoard {
    pub placed_oceans: usize,
    pub oxygen: usize,
    pub temperature: isize,
}

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

#[cfg(test)]
mod tests {
    use crate::game::DEFAULT_STARTING_TERRAFORM_RATING;
    use crate::game::PlayerStateBuilder;
    use crate::card::BASE_GAME_CARDS_BY_NAME;

    #[test]
    fn test_victory_points_from_tags_count_own_card_tags() {
        let played_cards: Vec<_> = [
            BASE_GAME_CARDS_BY_NAME["Ganymede Colony"],           // Jovian + 1VP/Jovian
            BASE_GAME_CARDS_BY_NAME["Water Import From Europa"],  // Jovian + 1VP/Jovian
            BASE_GAME_CARDS_BY_NAME["Methane From Titan"],        // Jovian + 2VP immediate
            BASE_GAME_CARDS_BY_NAME["Tundra Farming"],            // 2VP immediate, not Jovian
        ].iter().copied().cloned().collect();

        let player_state = PlayerStateBuilder::new()
            .with_played_cards(played_cards)
            .build();

        // cards are worth 10 points:
        // 3 Jovian tags valued at 2VP per Jovian card + 4VP immediate
        let points_from_cards: isize = 10;
        let expected_points = (DEFAULT_STARTING_TERRAFORM_RATING as isize) + points_from_cards;
        assert_eq!(expected_points, player_state.current_total_points);
    }
}
