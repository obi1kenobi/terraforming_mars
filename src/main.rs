#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    num::NonZeroUsize,
};
use std::hash::Hash;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum CardTag {
    Building,
    Space,
    Power,
    Science,
    Jovian,
    Earth,
    Plant,
    Microbe,
    Animal,
    City,
    Wild,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum Resource {
    Megacredits,
    Steel,
    Titanium,
    Plants,
    Energy,
    Heat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum CardResource {
    Microbe,
    Plant,
    Animal,
    Science,
    Fighter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum CardRequirement {
    MaxOxygen(usize),
    MinOxygen(usize),
    MaxTemperature(isize),
    MinTemperature(isize),
    MinTags(CardTag, NonZeroUsize),
    MinGreeneries(NonZeroUsize),
    MinProduction(Resource, NonZeroUsize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ResourceCost {
    Megacredits(usize),
    Space(usize),
    Building(usize),
    SpaceOrBuilding(usize),
    Microbe(usize),
    Plant(usize),
    Animal(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum CardKind {
    Active,
    Automatic,
    Event,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum VictoryPointValue {
    Immediate(isize),
    PerCity(usize),

    // how many points, per how many tags/resources, of which kind
    PerTag(usize, usize, CardTag),
    PerCardResource(usize, usize, CardResource),

    // fixed number of points, if the card has any of the given card resource
    FixedPointsIfAnyCardResourcePresent(usize, CardResource),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum CardAction {
    ResourceConversion(ResourceConversionAction),
    DrawCard(DrawCardsAction),
    AddResourceToSameCard(AddCardResourceAction),
    GainResourcesPerCityOnMars(GainResourcePerCityOnMarsAction),
    RevealAndDiscardCardToGainCardResource(RevealAndDiscardCardToGainCardResourceAction),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ResourceConversionAction {
    cost: HashMap<Resource, usize>,
    production: HashMap<Resource, usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DrawCardsAction {
    cost: HashMap<Resource, usize>,
    draw_size: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AddCardResourceAction {
    card_resource: CardResource,
    count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct GainResourcePerCityOnMarsAction {
    cost: HashMap<Resource, usize>,
    production_per_city: HashMap<Resource, usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RevealAndDiscardCardToGainCardResourceAction {
    cost: HashMap<Resource, usize>,
    sought_tags: HashSet<CardTag>,
    gained_resources: HashMap<CardResource, usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum ImmediateImpact {
    RaiseTemperature,
    RaiseOxygen,
    RaiseTerraformRating,
    
    // None in the option means "placed anywhere"
    PlaceOcean(Option<LocationKind>),
    PlaceGreenery(Option<LocationKind>),
    PlaceCity(Option<CityKind>),
    
    DrawCard(usize),
    AddResourceToSameCard(CardResource, usize),
    AddResourceToAnyCard(CardResource, usize),
    GainResource(Resource, usize),
    DestroyOwnPlants(usize),
    DestroyAnyPlants(usize),
    PlaceSpecialTile(SpecialTile),
    CopyProductionOfCard(CardTag),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum LocationKind {
    RegularLand,
    ReservedForOcean,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum CityKind {
    RegularCity,
    NoctisCity,
    LavaTunnelCity,
    PhobosSpaceHaven,
    GanymedeColony,
    UrbanizedArea,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum SpecialTile {
    // TODO: fill me in
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum CardEffect {
    AnyCardDiscount(usize),
    CardDiscountForTag(CardTag, usize),
    CannotRemoveCardResources(HashSet<CardResource>),

    // gain (card) resource / production, of given quantity, when own/anyone's move has one of the given immediate impacts
    GainCardResourceForOwnImpact(CardResource, isize, HashSet<ImmediateImpact>),
    GainResourceForOwnImpact(Resource, isize, HashSet<ImmediateImpact>),
    GainCardResourceForAnyImpact(CardResource, isize, HashSet<ImmediateImpact>),
    GainResourceForAnyImpact(Resource, isize, HashSet<ImmediateImpact>),
    GainProductionForOwnImpact(Resource, isize, HashSet<ImmediateImpact>),
    GainProductionForAnyImpact(Resource, isize, HashSet<ImmediateImpact>),

    // gain production when anyone plays a card with this tag
    GainProductionForCardTagPlayed(Resource, isize, CardTag),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Card {
    name: String,
    kind: CardKind,
    tags: Vec<CardTag>,
    cost: ResourceCost,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    requirements: Vec<CardRequirement>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    points: Option<VictoryPointValue>,

    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    own_production: HashMap<Resource, isize>, // the card player's production change

    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    any_production: HashMap<Resource, isize>, // any player's change (for stealing production)

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    immediate_impacts: Vec<ImmediateImpact>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    actions: Vec<CardAction>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    effects: Vec<CardEffect>,
}

impl Card {
    pub(crate) fn new(
        name: String,
        kind: CardKind,
        tags: Vec<CardTag>,
        cost: ResourceCost,
        requirements: Vec<CardRequirement>,
        points: Option<VictoryPointValue>,
        own_production: HashMap<Resource, isize>,
        any_production: HashMap<Resource, isize>,
        immediate_impacts: Vec<ImmediateImpact>,
        actions: Vec<CardAction>,
        effects: Vec<CardEffect>,
    ) -> Self {
        // Either there are no actions, or the card is considered active.
        assert!(actions.is_empty() ^ (kind == CardKind::Active));

        if kind == CardKind::Event {
            // Events have no actions.
            assert!(actions.is_empty());
        }

        Self {
            name,
            kind,
            tags,
            cost,
            requirements,
            points,
            own_production,
            any_production,
            immediate_impacts,
            actions,
            effects,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct PlayerState {
    resources: HashMap<Resource, usize>,
    production: HashMap<Resource, isize>,
    played_cards: Vec<Card>,
}

impl PlayerState {
    pub(crate) fn played_event_count(&self) -> usize {
        self.played_cards
            .iter()
            .filter(|card| card.kind == CardKind::Event)
            .count()
    }

    pub(crate) fn active_tag_count(&self, tag_kind: CardTag) -> usize {
        self.get_played_non_event_tags()
            .filter(|&tag| tag == tag_kind)
            .count()
    }

    pub(crate) fn active_tag_count_for_action(&self, tag_kind: CardTag) -> usize {
        // Wild tags only count for the purposes of performing actions.
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

    pub(crate) fn advance_generation() {
        todo!()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MarsBoard {}

pub fn main() {}

#[cfg(test)]
mod tests {
    use crate::Card;

    #[test]
    fn base_deck_is_valid() {
        let base_deck_text = include_str!("./cards/base/deck.json");

        let cards: Vec<Card> = serde_json::from_str(base_deck_text).unwrap();
        assert!(!cards.is_empty());
    }

    #[test]
    fn corporate_deck_is_valid() {
        let base_deck_text = include_str!("./cards/corporate/deck.json");

        let cards: Vec<Card> = serde_json::from_str(base_deck_text).unwrap();
        assert!(!cards.is_empty());
    }
}
