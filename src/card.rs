use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

use crate::resource::{CardResource, PaymentCost, Resource};

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardRequirement {
    MaxOxygen(usize),
    MinOxygen(usize),
    MaxTemperature(isize),
    MinTemperature(isize),
    MaxOceans(usize),
    MinOceans(usize),
    MinTags(CardTag, usize),
    MinOwnedGreeneries(usize),
    MinProduction(Resource, usize),
}

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardKind {
    Active,
    Automatic,
    Event,
}

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardAction {
    CauseFreeImpact(ImmediateImpact),
    SpendResource(PaymentCost, Vec<ImmediateImpact>),
    SpendProduction(Resource, usize, Vec<ImmediateImpact>),

    // spend a card resource of the same card that has this action
    SpendSameCardResource(CardResource, usize, ImmediateImpact),

    // Take a card resource from any card having any of the matching resource.
    // This is "take" not "spend" because you can take resources from opponents' cards too.
    TakeAnyCardResource(CardResource, usize, ImmediateImpact),

    // pay resource in given quantity, then draw and discard a card from the main deck;
    // if the card contains the specified tag, cause the specified impact
    RandomizeBasedOnRevealedCardTag(Resource, usize, CardTag, ImmediateImpact),
}

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImmediateImpact {
    RaiseTemperature,
    RaiseOxygen,
    RaiseTerraformRating,

    // gain count of TR per number of tags
    GainTerraformRatingPerOwnTag(usize, CardTag, usize),

    PlaceOcean(Vec<LocationRestriction>),
    PlaceGreenery(Vec<LocationRestriction>),
    PlaceCity(CityKind, Vec<LocationRestriction>),

    // like PlaceOcean, but may also remove the specified resource amount from any player
    // adjacent to the placed ocean tile
    PlaceFloodingOcean(Resource, usize, Vec<LocationRestriction>),

    DrawCard(usize),
    DiscardCard(usize),

    // unbought / un-taken cards are always discarded
    LookAndBuyFromDeck(usize), // look at N cards, buy any of them at the standard purchase price
    LookAndTakeFromDeck(usize, usize), // (look_at, take): look at N cards, keep M of them

    AddResourceToSameCard(CardResource, usize), // card that caused the impact
    AddResourceToAnotherCard(CardResource, usize), // *not* the card that caused the impact
    AddResourceToAnyCard(CardResource, usize), // any card, including the one that caused the impact

    // used when impact is triggered from an effect,
    // the resource type is whatever card resource type the played card supports
    AddResourceToPlayedCard(usize),

    // (min_resource, add_amount): if a card has at least the given amount of its resource,
    // then add the specified amount of its resource to it; example:
    // "add 1 resource to a card with at least 1 resource on it"
    AddResourceToAnyCardWithExistingResource(usize, usize),

    // spend a card resource from the card that caused this impact in order to cause another impact;
    // example: "remove a science resource from this card to draw a card"
    SpendResourceFromSameCard(CardResource, usize, Box<ImmediateImpact>),

    GainResource(Resource, usize),
    SpendResource(Resource, usize),
    GainResourcePerCity(Resource, usize),
    GainResourcePerCityOnMars(Resource, usize),
    ChangeProduction(Resource, isize),
    GainProductionPerCity(Resource, usize),
    GainProductionPerCityOnMars(Resource, usize),

    // gain the specified resource amount if having at least the given number of tags in play
    GainProductionIfMinTags(Resource, usize, CardTag, usize),

    // used for cards like Mining Area and Mining Rights:
    // they must be placed on tiles with a steel or titanium placement bonus,
    // and the production matching the placement bonus goes up by the specified amount
    GainMiningProductionMatchingPlacementBonus(usize),

    // gain production: (per card tag, count, gain resource production, of magnitude)
    GainProductionPerOwnTag(CardTag, usize, Resource, usize),
    GainProductionPerOpponentTag(CardTag, usize, Resource, usize),
    GainProductionPerAnyTag(CardTag, usize, Resource, usize), // own or opponent-played tag

    // transform arbitrary amount of resources/production (from_resource, to_resource), examples:
    // - "spend any amount of energy to gain that amount of M$"
    // - "decrease your heat production any number of steps and increase your M$ production
    //    the same number of steps"
    TransformResource(Resource, Resource),
    TransformProduction(Resource, Resource),

    // all steal and destroy *up to* that amount
    DestroyAnyResource(Resource, usize),
    DestroyAnyCardResource(CardResource, usize),
    StealResource(Resource, usize), // steal from any player, give to yourself

    PlaceSpecialTile(SpecialTile, Vec<LocationRestriction>),

    CopyProductionOfCard(CardTag),

    // any one of the contained impacts, at the players' choice
    OneOf(Vec<ImmediateImpact>),

    // if the first impact can be resolved successfully, the second may happen too; example:
    // "discard a card from your hand to draw a card"
    Chained(Box<ImmediateImpact>, Box<ImmediateImpact>),
}

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LocationRestriction {
    LandTile,
    ReservedForOcean,

    // the former is *required*, the latter is "if able" but ignored if unable
    AdjacentToOwnedTile,
    AdjacentToOwnedTileIfAble, // some greenery placements don't have this! e.g. Mangrove

    NotNextToAnyOtherTile,
    NotNextToACity,
    NextToACity,
    NextToAtLeastTwoCities,
    NextToAGreenery,
    OnSteelOrTitaniumPlacementBonus,

    // tiles cannot be placed at SpecialLocation sites other than VolcanicArea unless they
    // specifically have the AtSpecialLocation restriction for that area
    AtSpecialLocation(SpecialLocation),
}

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpecialTile {
    NuclearZone,
    RestrictedArea,
    LavaFlows,
    CommercialDistrict, // scores 1VP per adjacent city tile
    NaturalPreserve,    // placed next to no other tile
    IndustrialCenter,   // placed adjacent to a city tile
    MoholeArea,         // placed on area reserved for ocean
    EcologicalZone,     // placed adjacent to any greenery tile
    MiningRights,       // placed on steel/titanium placement bonus
    MiningArea,         // placed on steel/titanium placement bonus, adjacent to owned tile

    // TODO: make land claim not be a tile -- it's a marker cube placed on the map,
    //       and therefore doesn't count for greenery adjacency, for the landlord award, etc.
    LandClaim,          // placed anywhere, may be replaced with any tile of the placing player,
                        // does not count for purposes of tile ownership
}

#[derive(Clone, Copy, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VictoryPointValue {
    Immediate(isize),

    PerNCities(usize), // 1 point per N cities, regardless of who owns them

    // how many points, per how many tags/resources, of which kind
    PerTag(usize, usize, CardTag),
    PerCardResource(usize, usize, CardResource),

    // fixed number of points, if the card has any of the given card resource
    FixedPointsIfAnyCardResourcePresent(usize, CardResource),
}

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardEffect {
    AnyCardDiscount(usize),

    // pay less up front for playing a card with that tag
    CardDiscountForTag(CardTag, usize),

    // steel and titanium gain additional value when converted to megacredits
    IncreasedMetalsValue(usize),

    // after playing a standard project (except selling patents), get megacredits back
    RebateForStandardProjects(usize),

    // oxygen / oceans / temperature requirements are +/- the given magnitude,
    // per player's choice for each
    GlobalRequirementsTolerance(usize),

    CannotRemoveThisCardResource(CardResource),
    CannotRemoveAnyCardResources(Vec<CardResource>),

    // whenever any player does the thing
    OnAnyPlacedOcean(ImmediateImpact),
    OnAnyPlacedCity(ImmediateImpact),
    OnAnyTagPlayed(CardTag, ImmediateImpact),

    // whenever the player with this effect does the thing
    OnOwnPlacedGreenery(ImmediateImpact),
    OnOwnTagPlayed(CardTag, ImmediateImpact),
    // all the tags are on the same card,
    // all the impacts happen individually (not conditioned on each other)
    OnOwnTagCombinationPlayed(Vec<CardTag>, Vec<ImmediateImpact>),
}

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub next_card_this_generation_effects: Vec<CardEffect>,
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
        next_card_this_generation_effects: Vec<CardEffect>,
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
            next_card_this_generation_effects,
        }
    }

    pub fn supports_card_resource(&self) -> Option<CardResource> {
        let mut result: Option<CardResource> = None;
        let mut possible_impacts: Vec<&ImmediateImpact> = vec![];

        for action in &self.actions {
            match action {
                CardAction::SpendSameCardResource(cr, _, impact) => {
                    assert_eq!(*cr, result.unwrap_or(*cr));
                    result = Some(*cr);

                    possible_impacts.push(impact);
                }
                CardAction::CauseFreeImpact(impact)
                | CardAction::TakeAnyCardResource(_, _, impact)
                | CardAction::RandomizeBasedOnRevealedCardTag(_, _, _, impact) => {
                    possible_impacts.push(impact)
                }
                CardAction::SpendResource(_, impacts)
                | CardAction::SpendProduction(_, _, impacts) => {
                    possible_impacts.extend(impacts.iter());
                }
            }
        }

        for impact in possible_impacts {
            match impact {
                ImmediateImpact::AddResourceToSameCard(cr, _) => {
                    assert_eq!(*cr, result.unwrap_or(*cr));
                    result = Some(*cr);
                }
                _ => {}
            }
        }

        result
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

fn get_cards_by_name(cards: &'static Vec<Card>) -> HashMap<&'static str, &'static Card> {
    cards
        .iter()
        .map(|card| (card.name.as_ref(), card))
        .collect()
}

lazy_static! {
    pub static ref BASE_GAME_DECK: Vec<Card> = get_base_game_deck();
    pub static ref CORPORATE_ERA_DECK: Vec<Card> = get_corporate_era_deck();
    pub static ref BASE_GAME_CARDS_BY_NAME: HashMap<&'static str, &'static Card> =
        get_cards_by_name(&BASE_GAME_DECK);
    pub static ref CORPORATE_GAME_CARDS_BY_NAME: HashMap<&'static str, &'static Card> =
        get_cards_by_name(&CORPORATE_ERA_DECK);
}

#[cfg(test)]
mod tests {
    use crate::{
        card::{
            get_base_game_deck, get_corporate_deck_only, Card, CardKind, CardTag, ImmediateImpact,
            BASE_GAME_CARDS_BY_NAME,
        },
        resource::{CardResource, PaymentCost},
    };

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

    #[test]
    fn card_resource_is_detected_properly() {
        let card = BASE_GAME_CARDS_BY_NAME["Predators"];
        assert_eq!(Some(CardResource::Animal), card.supports_card_resource());
    }

    #[test]
    fn cards_that_only_transfer_card_resources_do_not_support_their_own_card_resource() {
        let card = BASE_GAME_CARDS_BY_NAME["Ants"];
        assert_eq!(None, card.supports_card_resource());
    }
}
