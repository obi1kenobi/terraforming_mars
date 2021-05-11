#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::{
    collections::{HashMap, HashSet},
    num::NonZeroUsize,
};

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
    Event,
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
    MaxOceans(usize),
    MinOceans(usize),
    MinTags(CardTag, NonZeroUsize),
    MinCities(NonZeroUsize),
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
    CauseFreeImpact(ImmediateImpact),
    SpendResource(Resource, usize, ImmediateImpact),
    SpendSameCardResource(CardResource, usize, ImmediateImpact),
    SpendProduction(Resource, usize, ImmediateImpact),

    // pay resource in given quantity, then draw and discard a card from the main deck;
    // if the card contains the specified tag, cause the specified impact
    RandomizeBasedOnRevealedCardTag(Resource, usize, CardTag, ImmediateImpact),
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
    AddResourceToSameCard(CardResource, usize), // card that caused the impact
    AddResourceToAnotherCard(CardResource, usize), // *not* the card that caused the impact
    AddResourceToAnyCard(CardResource, usize), // any card, including the one that caused the impact
    GainResource(Resource, usize),
    GainResourcePerCityOnMars(Resource, usize),
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
    GainProductionForAnyTagPlayed(Resource, isize, CardTag),
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
            assert!(tags.contains(&CardTag::Event));
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
    use crate::{Card, CardKind, CardTag, ResourceCost};

    fn is_card_valid(card: &Card) -> bool {
        let mut is_valid = true;

        if card.kind == CardKind::Active {
            is_valid &= !card.actions.is_empty();
        } else {
            is_valid &= card.actions.is_empty();
        }

        if card.kind == CardKind::Event {
            is_valid &= card.effects.is_empty();

            is_valid &= card.tags.contains(&CardTag::Event);
        }

        if card.tags.contains(&CardTag::Event) {
            is_valid &= card.kind == CardKind::Event;
        }

        match card.cost {
            ResourceCost::Building(_) => {
                is_valid &= card.tags.contains(&CardTag::Building);
                is_valid &= !card.tags.contains(&CardTag::Space);
            }
            ResourceCost::Space(_) => {
                is_valid &= !card.tags.contains(&CardTag::Building);
                is_valid &= card.tags.contains(&CardTag::Space);
            }
            ResourceCost::SpaceOrBuilding(_) => {
                is_valid &= card.tags.contains(&CardTag::Building);
                is_valid &= card.tags.contains(&CardTag::Space);
            }
            ResourceCost::Megacredits(_) | _ => {
                is_valid &= !card.tags.contains(&CardTag::Building);
                is_valid &= !card.tags.contains(&CardTag::Space);
            }
        }

        is_valid
    }

    #[test]
    fn base_deck_is_valid() {
        let base_deck_text = include_str!("./cards/base/deck.json");

        let cards: Vec<Card> = serde_json::from_str(base_deck_text).unwrap();
        assert!(!cards.is_empty());

        let invalid_cards: Vec<_> = cards.iter().filter(|x| !is_card_valid(x)).collect();
        assert!(invalid_cards.is_empty(), "{:?}", invalid_cards);
    }

    #[test]
    fn corporate_deck_is_valid() {
        let base_deck_text = include_str!("./cards/corporate/deck.json");

        let cards: Vec<Card> = serde_json::from_str(base_deck_text).unwrap();
        assert!(!cards.is_empty());

        let invalid_cards: Vec<_> = cards.iter().filter(|x| !is_card_valid(x)).collect();
        assert!(invalid_cards.is_empty(), "{:?}", invalid_cards);
    }
}
