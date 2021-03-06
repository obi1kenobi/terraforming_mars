use std::collections::{BTreeMap, HashMap, HashSet};

use maplit::btreemap;
use rand::{prelude::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};

use crate::{
    board::{Coordinates, MarsBoard, TileLocation, TileStatus},
    card::{
        Card, CardAction, CardEffect, CardKind, CardRequirement, CardTag, CityKind,
        ImmediateImpact, SpecialTile, VictoryPointValue,
    },
    resource::{CardResource, PaymentCost, Resource},
};

const CARD_PURCHASE_COST: usize = 3;
const DEFAULT_STARTING_TERRAFORM_RATING: usize = 20;
const DEFAULT_SOLO_STARTING_TERRAFORM_RATING: usize = 14;
const DEFAULT_STEEL_VALUE: usize = 2;
const DEFAULT_TITANIUM_VALUE: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PlayerId(usize);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerState {
    // primary data
    pub player_id: PlayerId,
    pub resources: BTreeMap<Resource, usize>,
    pub production: BTreeMap<Resource, isize>,
    pub played_cards: Vec<Card>,
    pub card_resources: BTreeMap<(Card, CardResource), usize>,
    pub tapped_active_cards: HashSet<Card>,
    pub cards_in_hand: Vec<Card>,
    pub terraform_rating: usize,
    pub steel_value: usize,
    pub titanium_value: usize,
    pub next_card_this_generation_effects: Vec<CardEffect>,

    // indexes of primary data
    pub effects: Vec<CardEffect>,
}

pub struct PlayerStateBuilder {
    pub player_id: PlayerId,
    pub resources: Option<BTreeMap<Resource, usize>>,
    pub production: Option<BTreeMap<Resource, isize>>,
    pub played_cards: Option<Vec<Card>>,
    pub card_resources: BTreeMap<(Card, CardResource), usize>,
    pub tapped_active_cards: Option<HashSet<Card>>,
    pub cards_in_hand: Option<Vec<Card>>,
    pub terraform_rating: usize,
    pub next_card_this_generation_effects: Option<Vec<CardEffect>>,
}

impl PlayerStateBuilder {
    pub fn new(player_id: usize) -> PlayerStateBuilder {
        PlayerStateBuilder {
            player_id: PlayerId(player_id),
            resources: None,
            production: None,
            played_cards: None,
            card_resources: btreemap! {},
            tapped_active_cards: None,
            cards_in_hand: None,
            terraform_rating: DEFAULT_STARTING_TERRAFORM_RATING,
            next_card_this_generation_effects: None,
        }
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

        let effects: Vec<_> = self
            .played_cards
            .as_ref()
            .map(|cards| cards.iter().flat_map(|c| c.effects.clone()).collect())
            .unwrap_or_default();

        let mut steel_value = DEFAULT_STEEL_VALUE;
        let mut titanium_value = DEFAULT_TITANIUM_VALUE;
        for effect in &effects {
            match effect {
                &CardEffect::IncreasedMetalsValue(increase) => {
                    steel_value += increase;
                    titanium_value += increase;
                }
                _ => {}
            }
        }

        PlayerState {
            player_id: self.player_id,
            resources,
            production,
            played_cards: self.played_cards.unwrap_or_default(),
            card_resources: card_resources,
            tapped_active_cards: self.tapped_active_cards.unwrap_or_default(),
            cards_in_hand: self.cards_in_hand.unwrap_or_default(),
            terraform_rating: self.terraform_rating,
            steel_value: steel_value,
            titanium_value: titanium_value,
            next_card_this_generation_effects: self.next_card_this_generation_effects.unwrap_or_default(),
            effects: effects,
        }
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

    pub fn get_total_victory_points(&self, board: &MarsBoard) -> isize {
        let mut current_total_points = self.terraform_rating as isize;
        let card_points: isize = self
            .played_cards
            .iter()
            .map(|c| match c.points {
                Some(VictoryPointValue::Immediate(x)) => x,
                Some(VictoryPointValue::PerTag(vp, count, tag)) => {
                    assert!(tag != CardTag::Event);

                    let tag_count = self.played_cards.active_tag_count(tag);
                    ((tag_count / count) * vp) as isize
                }
                Some(VictoryPointValue::PerCardResource(vp, count, cr)) => {
                    let resources_present = self
                        .card_resources
                        .get(&(c.clone(), cr))
                        .copied()
                        .unwrap_or_default();

                    ((resources_present / count) * vp) as isize
                }
                Some(VictoryPointValue::FixedPointsIfAnyCardResourcePresent(count, cr)) => {
                    let resources_present = self
                        .card_resources
                        .get(&(c.clone(), cr))
                        .copied()
                        .unwrap_or_default();
                    if resources_present > 0 {
                        count as isize
                    } else {
                        0
                    }
                }
                Some(VictoryPointValue::PerNCities(n_cities)) => {
                    (board.cities.len() / n_cities) as isize
                }
                None => 0,
            })
            .sum();
        current_total_points += card_points;

        let greenery_points = board
            .greeneries
            .values()
            .filter(|player_id| **player_id == self.player_id)
            .count();
        current_total_points += greenery_points as isize;

        let city_points: usize = board
            .cities
            .iter()
            .filter(|(_, (_, player_id))| *player_id == self.player_id)
            .map(|(location, (city_kind, _))| {
                let capital_points = if matches!(city_kind, CityKind::Capital) {
                    board
                        .get_neighbor_tile_status(location)
                        .filter(|status| matches!(status, &TileStatus::Ocean(_)))
                        .count()
                } else {
                    0
                };

                let greenery_adjacency_points = board
                    .get_neighbor_tile_status(location)
                    .filter(|status| matches!(status, &TileStatus::Greenery(_, _)))
                    .count();

                capital_points + greenery_adjacency_points
            })
            .sum();
        current_total_points += city_points as isize;

        current_total_points
    }

    pub fn can_play_card(&self, board: &MarsBoard, card: &Card) -> Option<PaymentCost> {
        let megacredits_balance = self.resources[&Resource::Megacredits];

        let fails_requirements = card
            .requirements
            .iter()
            .any(|requirement| match requirement {
                // TODO: check for requirements-easing effect
                CardRequirement::MaxOxygen(max_oxygen) => board.oxygen <= *max_oxygen,
                CardRequirement::MinOxygen(min_oxygen) => board.oxygen >= *min_oxygen,
                CardRequirement::MaxTemperature(max_temp) => board.temperature <= *max_temp,
                CardRequirement::MinTemperature(min_temp) => board.temperature >= *min_temp,
                CardRequirement::MaxOceans(max_oceans) => board.oceans.len() <= *max_oceans,
                CardRequirement::MinOceans(min_oceans) => board.oceans.len() >= *min_oceans,
                CardRequirement::MinOwnedGreeneries(min_greeneries) => {
                    let owned_greeneries = board
                        .greeneries
                        .values()
                        .filter(|player_id| self.player_id == **player_id)
                        .count();

                    owned_greeneries >= *min_greeneries
                }
                CardRequirement::MinTags(tag, count) => self.active_tag_count(*tag) >= *count,
                CardRequirement::MinProduction(resource, amount) => {
                    self.production[resource] >= (*amount as isize)
                }
            });
        if fails_requirements {
            return None;
        }

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

        if !can_pay {
            None
        } else {
            let cloned_cost = card.cost.clone();
            Some(cloned_cost)
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
        self.next_card_this_generation_effects.clear();
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameOperation {
    ChangeResources(PlayerId, BTreeMap<Resource, isize>),
    ChangeProduction(PlayerId, BTreeMap<Resource, isize>),
    ChangeCardResource(PlayerId, Card, CardResource, isize),
    DrawCards(PlayerId, usize),
    DiscardCards(PlayerId, Vec<Card>),
    PutCardIntoPlay(PlayerId, Card),
    PlaceCityTile(PlayerId, CityKind, TileLocation),
    PlaceGreenery(PlayerId, Coordinates),
    PlaceSpecialTile(PlayerId, SpecialTile, Coordinates),
    PlaceOcean(Coordinates),
    RaiseTemperature,
    RaiseOxygen,
    RaiseTerraformRating(PlayerId, usize),
    AddEffect(PlayerId, CardEffect),
    MarkCardActionUsed(PlayerId, Card),
    ResetCardActions,
    ClaimMilestone, // TODO: add milestone info
    FundAward,      // TODO: add award info
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub board: MarsBoard,
    pub players: HashMap<PlayerId, PlayerState>,
    pub draw_deck: Vec<Card>,
    pub discard_pile: Vec<Card>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayAttempt {
    Unplayable,

    // operations so far + remaining impacts to resolve
    PartiallyPlayable(Vec<GameOperation>, Vec<ImmediateImpact>),

    Playable(Vec<GameOperation>),
}

impl Game {
    pub fn execute_operation(&mut self, operation: GameOperation) {
        match operation {
            GameOperation::ChangeResources(player_id, resources) => {
                let player = self.players.get_mut(&player_id).unwrap();
                for (resource, change) in resources.iter() {
                    if *change != 0 {
                        let prior_value = player.resources.get_mut(resource).unwrap();
                        let new_value = (*prior_value as isize) + change;
                        assert!(new_value >= 0);
                        *prior_value = new_value as usize;
                    }
                }
            }
            GameOperation::ChangeProduction(player_id, production) => {
                let player = self.players.get_mut(&player_id).unwrap();
                for (resource, change) in production.iter() {
                    if *change != 0 {
                        let prior_value = player.production.get_mut(resource).unwrap();
                        let new_value = *prior_value + change;
                        if *resource == Resource::Megacredits {
                            assert!(player.terraform_rating as isize + new_value >= 0);
                        } else {
                            assert!(new_value >= 0);
                        }
                        *prior_value = new_value;
                    }
                }
            }
            GameOperation::ChangeCardResource(player_id, card, card_resource, amount) => {
                let player = self.players.get_mut(&player_id).unwrap();

                assert!(card.supports_card_resource() == Some(card_resource));

                player
                    .card_resources
                    .entry((card, card_resource))
                    .and_modify(|quantity| {
                        let new_quantity = (*quantity as isize) + amount;
                        assert!(new_quantity >= 0);
                        *quantity = new_quantity as usize;
                    })
                    .or_insert_with(|| {
                        assert!(amount >= 0);
                        amount as usize
                    });
            }
            GameOperation::DrawCards(player_id, mut count) => {
                let player = self.players.get_mut(&player_id).unwrap();

                if count > self.draw_deck.len() {
                    player.cards_in_hand.extend(self.draw_deck.drain(..));
                    count -= self.draw_deck.len();

                    let mut rng = thread_rng();
                    self.discard_pile.shuffle(&mut rng);
                    self.draw_deck.extend(self.discard_pile.drain(..));
                }

                assert!(count <= self.draw_deck.len());
                player
                    .cards_in_hand
                    .extend(self.draw_deck.drain((self.draw_deck.len() - count)..));
            }
            GameOperation::DiscardCards(player_id, discard) => {
                let player = self.players.get_mut(&player_id).unwrap();

                let initial_hand_size = player.cards_in_hand.len();
                player.cards_in_hand.retain(|card| !discard.contains(card));
                assert_eq!(
                    initial_hand_size,
                    player.cards_in_hand.len() + discard.len()
                );

                self.discard_pile.extend_from_slice(&discard);
            }
            GameOperation::PutCardIntoPlay(player_id, played_card) => {
                let player = self.players.get_mut(&player_id).unwrap();

                let initial_hand_size = player.cards_in_hand.len();
                player.cards_in_hand.retain(|card| *card != played_card);
                assert_eq!(initial_hand_size, player.cards_in_hand.len() + 1);

                player.played_cards.push(played_card);
            }
            GameOperation::PlaceCityTile(player_id, city_kind, location) => {
                match location {
                    TileLocation::OnMars(coordinates) => {
                        assert!(self.board.greeneries.get(&coordinates).is_none());
                        assert!(self.board.special_tiles.get(&coordinates).is_none());
                        assert!(self.board.oceans.get(&coordinates).is_none());
                    }
                    _ => {}
                }

                let existing_city = self.board.cities.insert(location, (city_kind, player_id));
                assert!(existing_city.is_none());
            }
            GameOperation::PlaceGreenery(player_id, coordinates) => {
                assert!(self
                    .board
                    .cities
                    .get(&TileLocation::OnMars(coordinates))
                    .is_none());
                assert!(self.board.special_tiles.get(&coordinates).is_none());
                assert!(self.board.oceans.get(&coordinates).is_none());

                let existing_greenery = self.board.greeneries.insert(coordinates, player_id);
                assert!(existing_greenery.is_none());
            }
            GameOperation::PlaceSpecialTile(player_id, tile, coordinates) => {
                assert!(self
                    .board
                    .cities
                    .get(&TileLocation::OnMars(coordinates))
                    .is_none());
                assert!(self.board.greeneries.get(&coordinates).is_none());
                assert!(self.board.oceans.get(&coordinates).is_none());

                let existing_tile = self
                    .board
                    .special_tiles
                    .insert(coordinates, (tile, player_id));
                assert!(existing_tile.is_none());
            }
            GameOperation::PlaceOcean(coordinates) => {
                assert!(self
                    .board
                    .cities
                    .get(&TileLocation::OnMars(coordinates))
                    .is_none());
                assert!(self.board.greeneries.get(&coordinates).is_none());
                assert!(self.board.special_tiles.get(&coordinates).is_none());

                assert!(self.board.oceans.len() < MarsBoard::MAX_OCEANS);

                let has_existing_ocean = self.board.oceans.insert(coordinates);
                assert!(!has_existing_ocean);
            }
            GameOperation::RaiseTemperature => {
                assert!(self.board.temperature < MarsBoard::MAX_TEMPERATURE);

                self.board.temperature += MarsBoard::TEMPERATURE_INCREMENT;
            }
            GameOperation::RaiseOxygen => {
                assert!(self.board.oxygen < MarsBoard::MAX_OXYGEN);

                self.board.oxygen += MarsBoard::OXYGEN_INCREMENT;
            }
            GameOperation::RaiseTerraformRating(player_id, amount) => {
                let player = self.players.get_mut(&player_id).unwrap();
                player.terraform_rating += amount;
            }
            GameOperation::AddEffect(player_id, effect) => {
                let player = self.players.get_mut(&player_id).unwrap();
                player.effects.push(effect);
            }
            GameOperation::MarkCardActionUsed(player_id, card) => {
                let player = self.players.get_mut(&player_id).unwrap();
                player.tapped_active_cards.insert(card);
            }
            GameOperation::ResetCardActions => {
                for (_, player) in self.players.iter_mut() {
                    player.tapped_active_cards.clear();
                }
            }
            GameOperation::ClaimMilestone => todo!(),
            GameOperation::FundAward => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::board::make_base_game_board;
    use crate::board::Coordinates;
    use crate::board::TileLocation;
    use crate::card::CityKind;
    use crate::card::SpecialLocation;
    use crate::card::BASE_GAME_CARDS_BY_NAME;
    use crate::game::PlayerStateBuilder;
    use crate::game::DEFAULT_STARTING_TERRAFORM_RATING;

    #[test]
    fn test_victory_points_from_tags_count_own_card_tags() {
        let played_cards: Vec<_> = [
            BASE_GAME_CARDS_BY_NAME["Ganymede Colony"], // Jovian + 1VP/Jovian
            BASE_GAME_CARDS_BY_NAME["Water Import From Europa"], // Jovian + 1VP/Jovian
            BASE_GAME_CARDS_BY_NAME["Methane From Titan"], // Jovian + 2VP immediate
            BASE_GAME_CARDS_BY_NAME["Tundra Farming"],  // 2VP immediate, not Jovian
        ]
        .iter()
        .copied()
        .cloned()
        .collect();

        let player_state = PlayerStateBuilder::new(1)
            .with_played_cards(played_cards)
            .build();

        let board = make_base_game_board();

        // cards are worth 10 points:
        // 3 Jovian tags valued at 2VP per Jovian card + 4VP immediate
        let points_from_cards: isize = 10;
        let expected_points = (DEFAULT_STARTING_TERRAFORM_RATING as isize) + points_from_cards;
        assert_eq!(
            expected_points,
            player_state.get_total_victory_points(&board)
        );
    }

    #[test]
    fn test_per_city_victory_points_count_all_player_cities_both_on_and_off_mars() {
        let p1_played_cards: Vec<_> = [BASE_GAME_CARDS_BY_NAME["Immigration Shuttles"]]
            .iter()
            .copied()
            .cloned()
            .collect();

        let p2_played_cards: Vec<_> = [BASE_GAME_CARDS_BY_NAME["Ganymede Colony"]]
            .iter()
            .copied()
            .cloned()
            .collect();

        let p1_player_state = PlayerStateBuilder::new(1)
            .with_played_cards(p1_played_cards)
            .build();
        let p2_player_state = PlayerStateBuilder::new(2)
            .with_played_cards(p2_played_cards)
            .build();

        let mut board = make_base_game_board();
        board.cities.insert(
            TileLocation::OnMars(Coordinates::new(0, 0)),
            (CityKind::RegularCity, p1_player_state.player_id),
        );
        board.cities.insert(
            TileLocation::OffMars(SpecialLocation::GanymedeColony),
            (CityKind::GanymedeColony, p2_player_state.player_id),
        );
        board.cities.insert(
            TileLocation::OnMars(Coordinates::new(5, -3)),
            (CityKind::RegularCity, p2_player_state.player_id),
        );

        // 1VP from immigration shuttles because of 3 cities in existence
        assert_eq!(
            1 + DEFAULT_STARTING_TERRAFORM_RATING as isize,
            p1_player_state.get_total_victory_points(&board)
        );

        // 1VP from Ganymede Colony's Jovian tag
        assert_eq!(
            1 + DEFAULT_STARTING_TERRAFORM_RATING as isize,
            p2_player_state.get_total_victory_points(&board)
        );
    }

    #[test]
    fn test_city_and_greneery_scoring() {
        let p1_player_state = PlayerStateBuilder::new(1).build();
        let p2_player_state = PlayerStateBuilder::new(2).build();

        let mut board = make_base_game_board();
        board.cities.insert(
            TileLocation::OnMars(Coordinates::new(0, 0)),
            (CityKind::RegularCity, p1_player_state.player_id),
        );
        board
            .greeneries
            .insert(Coordinates::new(1, 0), p1_player_state.player_id);
        board
            .greeneries
            .insert(Coordinates::new(1, -1), p2_player_state.player_id);

        // 1VP from the greenery, 2VP from the city adjacent to 2 greeneries
        assert_eq!(
            3 + DEFAULT_STARTING_TERRAFORM_RATING as isize,
            p1_player_state.get_total_victory_points(&board)
        );

        // 1VP from the greenery
        assert_eq!(
            1 + DEFAULT_STARTING_TERRAFORM_RATING as isize,
            p2_player_state.get_total_victory_points(&board)
        );
    }

    #[test]
    fn test_capital_city_scoring() {
        let p1_played_cards: Vec<_> = [BASE_GAME_CARDS_BY_NAME["Capital"]]
            .iter()
            .copied()
            .cloned()
            .collect();

        let p1_player_state = PlayerStateBuilder::new(1)
            .with_played_cards(p1_played_cards)
            .build();
        let p2_player_state = PlayerStateBuilder::new(2).build();

        let mut board = make_base_game_board();
        board.cities.insert(
            TileLocation::OnMars(Coordinates::new(4, -5)),
            (CityKind::Capital, p1_player_state.player_id),
        );
        board
            .greeneries
            .insert(Coordinates::new(3, -5), p2_player_state.player_id);
        board
            .greeneries
            .insert(Coordinates::new(4, -6), p2_player_state.player_id);
        board.oceans.insert(Coordinates::new(5, -5));
        board.oceans.insert(Coordinates::new(5, -6));
        board.oceans.insert(Coordinates::new(4, -4));

        // 3VP from the oceans adjacent to the capital, 2VP from the adjacent greeneries
        assert_eq!(
            5 + DEFAULT_STARTING_TERRAFORM_RATING as isize,
            p1_player_state.get_total_victory_points(&board)
        );

        // 2VP from the greeneries
        assert_eq!(
            2 + DEFAULT_STARTING_TERRAFORM_RATING as isize,
            p2_player_state.get_total_victory_points(&board)
        );
    }
}
