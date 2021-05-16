use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;
use std::{collections::BTreeMap, hash::Hash};

use crate::resource::{CardResource, PaymentCost, Resource};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardTag {
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
pub enum CardRequirement {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardKind {
    Active,
    Automatic,
    Event,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardAction {
    CauseFreeImpact(ImmediateImpact),
    SpendResource(PaymentCost, ImmediateImpact),
    SpendSameCardResource(CardResource, usize, ImmediateImpact),
    SpendProduction(Resource, usize, ImmediateImpact),

    // pay resource in given quantity, then draw and discard a card from the main deck;
    // if the card contains the specified tag, cause the specified impact
    RandomizeBasedOnRevealedCardTag(Resource, usize, CardTag, ImmediateImpact),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImmediateImpact {
    RaiseTemperature,
    RaiseOxygen,
    RaiseTerraformRating,

    // gain count of TR per number of tags
    GainTerraformRatingPerOwnTag(usize, CardTag, usize),

    PlaceOcean(Vec<LocationRestriction>),
    PlaceGreenery(Vec<LocationRestriction>),
    PlaceCity(CityKind, Vec<LocationRestriction>),

    DrawCard(usize),
    AddResourceToSameCard(CardResource, usize), // card that caused the impact
    AddResourceToAnotherCard(CardResource, usize), // *not* the card that caused the impact
    AddResourceToAnyCard(CardResource, usize), // any card, including the one that caused the impact
    GainResource(Resource, usize),
    GainResourcePerCity(Resource, usize),
    GainResourcePerCityOnMars(Resource, usize),
    GainProduction(Resource, usize),
    GainProductionPerCity(Resource, usize),
    GainProductionPerCityOnMars(Resource, usize),

    // gain production: (per card tag, count, gain resource production, of magnitude)
    GainProductionPerOwnTag(CardTag, usize, Resource, usize),
    GainProductionPerOpponentTag(CardTag, usize, Resource, usize),
    GainProductionPerAnyTag(CardTag, usize, Resource, usize), // own or opponent-played tag

    DestroyOwnPlants(usize),
    DestroyAnyPlants(usize),
    PlaceSpecialTile(SpecialTile, Vec<LocationRestriction>),
    CopyProductionOfCard(CardTag),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LocationRestriction {
    LandTile,
    ReservedForOcean,
    AdjacentToOwnedTileIfAble,  // some greenery placements don't have this! e.g. Mangrove
    NotNextToAnyOtherTile,
    NotNextToACity,
    NextToACity,
    NextToAtLeastTwoCities,
    AtSpecialLocation(SpecialLocation),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpecialLocation {
    // N.B.: Not all of these locations exist on all game maps.
    //       The base game ships with only the Tharsis map.

    PhobosSpaceHaven,
    GanymedeColony,

    // On the Tharsis map, Noctis City has a reserved named tile.
    // On Elysium and Hellas, Noctis City is played as a regular city.
    // https://boardgamegeek.com/thread/2134024/article/31112841#31112841
    NoctisCity,

    // Volcanic areas are different on different maps.
    // On the Tharsis map, those are: Tharsis Tholus, Ascraeus Mons, Pavonis Mons, and Arsia Mons.
    // On the Elysium map, those are: Hecates Tholus, Elysium Mons, Olympus Mons, and Arsia Mons.
    // On the Hellas map, there are no volcanoes, so the placement restriction becomes
    // "anywhere except otherwise restricted areas, as if a standard tile of its kind."
    // https://boardgamegeek.com/thread/2096004/article/30497241#30497241
    VolcanicArea,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CityKind {
    RegularCity,
    Capital,          // regular city + scores 1VP per adjacent ocean
    NoctisCity,       // reserved place on map
    PhobosSpaceHaven, // reserved place on map
    GanymedeColony,   // reserved place on map
    LavaTunnelCity,   // one of several possible places on map
    UrbanizedArea,    // placed between two cities
    ResearchOutpost,  // placed next to no other tile
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpecialTile {
    Capital,
    NuclearZone,
    RestrictedArea,
    LavaFlows,
    CommercialDistrict, // scores 1VP per adjacent city tile
    NaturalPreserve,    // placed next to no other tile
    IndustrialCenter,   // placed adjacent to a city tile
    MoholeArea,         // placed on area reserved for ocean
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VictoryPointValue {
    Immediate(isize),

    PerCity(usize),    // N points per 1 city
    PerNCities(usize), // 1 point per N cities

    // how many points, per how many tags/resources, of which kind
    PerTag(usize, usize, CardTag),
    PerCardResource(usize, usize, CardResource),

    // fixed number of points, if the card has any of the given card resource
    FixedPointsIfAnyCardResourcePresent(usize, CardResource),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardEffect {
    AnyCardDiscount(usize),

    // pay less up front for playing a card with that tag
    CardDiscountForTag(CardTag, usize),

    // get Megacredits back after putting a tag into play (must be able to afford the inital price)
    RebateAfterPlayingCardTag(CardTag, usize),

    CannotRemoveThisCardResource(CardResource),
    CannotRemoveAnyCardResources(Vec<CardResource>),

    // whenever any player does the thing
    OnAnyPlacedOcean(ImmediateImpact),
    OnAnyPlacedCity(ImmediateImpact),
    OnAnyTagPlayed(CardTag, ImmediateImpact),

    // whenever the player with this effect does the thing
    OnOwnPlacedGreenery(ImmediateImpact),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card {
    pub name: String,
    pub kind: CardKind,
    pub tags: Vec<CardTag>,
    pub cost: PaymentCost,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub requirements: Vec<CardRequirement>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points: Option<VictoryPointValue>,

    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub own_production: BTreeMap<Resource, isize>, // the card player's production change

    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub any_production: BTreeMap<Resource, isize>, // any player's change (for stealing production)

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub immediate_impacts: Vec<ImmediateImpact>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<CardAction>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub effects: Vec<CardEffect>,
}

impl Card {
    pub(crate) fn new(
        name: String,
        kind: CardKind,
        tags: Vec<CardTag>,
        cost: PaymentCost,
        requirements: Vec<CardRequirement>,
        points: Option<VictoryPointValue>,
        own_production: BTreeMap<Resource, isize>,
        any_production: BTreeMap<Resource, isize>,
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

pub fn get_base_game_deck() -> Vec<Card> {
    let base_deck_text = include_str!("./cards/base/deck.json");
    let cards: Vec<Card> = serde_json::from_str(base_deck_text).unwrap();
    cards
}

fn get_corporate_deck_only() -> Vec<Card> {
    let base_deck_text = include_str!("./cards/corporate/deck.json");
    let cards: Vec<Card> = serde_json::from_str(base_deck_text).unwrap();
    cards
}

pub fn get_corporate_era_deck() -> Vec<Card> {
    let mut deck = get_base_game_deck();
    deck.extend(get_corporate_deck_only());
    deck
}

#[cfg(test)]
mod tests {
    use crate::{card::{Card, CardKind, CardTag, ImmediateImpact, get_base_game_deck, get_corporate_deck_only}, resource::PaymentCost};

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

        if card.tags.contains(&CardTag::City) {
            is_valid &= card
                .immediate_impacts
                .iter()
                .filter(|x| matches!(x, ImmediateImpact::PlaceCity(_, _)))
                .count()
                > 0
        }

        match card.cost {
            PaymentCost::Building(_) => {
                is_valid &= card.tags.contains(&CardTag::Building);
                is_valid &= !card.tags.contains(&CardTag::Space);
            }
            PaymentCost::Space(_) => {
                is_valid &= !card.tags.contains(&CardTag::Building);
                is_valid &= card.tags.contains(&CardTag::Space);
            }
            PaymentCost::SpaceOrBuilding(_) => {
                is_valid &= card.tags.contains(&CardTag::Building);
                is_valid &= card.tags.contains(&CardTag::Space);
            }
            PaymentCost::Megacredits(_) | _ => {
                is_valid &= !card.tags.contains(&CardTag::Building);
                is_valid &= !card.tags.contains(&CardTag::Space);
            }
        }

        is_valid
    }

    #[test]
    fn base_deck_is_valid() {
        let cards = get_base_game_deck();
        assert!(!cards.is_empty());

        let invalid_cards: Vec<_> = cards.iter().filter(|x| !is_card_valid(x)).collect();
        assert!(invalid_cards.is_empty(), "{:?}", invalid_cards);
    }

    #[test]
    fn corporate_deck_is_valid() {
        let cards = get_corporate_deck_only();
        assert!(!cards.is_empty());

        let invalid_cards: Vec<_> = cards.iter().filter(|x| !is_card_valid(x)).collect();
        assert!(invalid_cards.is_empty(), "{:?}", invalid_cards);
    }
}
