use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    card::{CityKind, ImmediateImpact, LocationRestriction, SpecialLocation, SpecialTile},
    game::{PlayerId, PlayerState},
    resource::Resource,
};

/// Using implicit 3-axis "cube" coordinate system, with all points satisfying x + y + z = 0.
/// We always drop the z coordinate, since it's implicitly z = -(x + y).
/// Properties:
/// - (0, 0) on the left-most hex of the center row.
/// - Bottom left edge has x = 0. The x coordinate increases to the top-right.
/// - Top left edge has y = 0. The y coordinate increases to the top-left.
/// - Center row has z = 0. The z coordinate increases downward.
///
/// Diagram at:
/// https://www.redblobgames.com/grids/hexagons/
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Coordinates {
    x: isize,
    y: isize,
}

impl Coordinates {
    const BOUNDS_MIN_X: isize = 0;
    const BOUNDS_MAX_X: isize = 8;
    const BOUNDS_MIN_Y: isize = -8;
    const BOUNDS_MAX_Y: isize = 0;
    const BOUNDS_MIN_Z: isize = -4;
    const BOUNDS_MAX_Z: isize = 4;

    const NEIGHBORS_DX_DY: [(isize, isize); 6] = [
        // clockwise neighbors, starting from the right neighbor
        (1, -1),
        (0, -1),
        (-1, 0),
        (-1, 1),
        (0, 1),
        (1, 0),
    ];

    #[inline]
    pub fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }

    #[inline]
    pub const fn get_z(&self) -> isize {
        -(self.x + self.y)
    }

    #[inline]
    pub fn is_in_bounds(&self) -> bool {
        // Check that the point is between the bottom-left and the top-right edge.
        let within_x = self.x >= Coordinates::BOUNDS_MIN_X && self.x <= Coordinates::BOUNDS_MAX_X;

        // Check that the point is between the bottom-right and the top-left edge.
        let within_y = self.y >= Coordinates::BOUNDS_MIN_Y && self.y <= Coordinates::BOUNDS_MAX_Y;

        // Check that the point is between the top and bottom edges.
        let z = self.get_z();
        let within_z = z >= Coordinates::BOUNDS_MIN_Z && z <= Coordinates::BOUNDS_MAX_Z;

        within_x && within_y && within_z
    }

    pub fn neighbors_within_bounds(&self) -> impl Iterator<Item = Self> {
        let x = self.x;
        let y = self.y;

        Coordinates::NEIGHBORS_DX_DY
            .iter()
            .map(move |(dx, dy)| {
                let new_x = x + dx;
                let new_y = y + dy;

                (new_x, new_y).into()
            })
            .filter(Coordinates::is_in_bounds)
    }
}

impl From<(isize, isize)> for Coordinates {
    #[inline]
    fn from(coordinates: (isize, isize)) -> Self {
        Self::new(coordinates.0, coordinates.1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TileLocation {
    OnMars(Coordinates),
    OffMars(SpecialLocation),
}

impl TileLocation {
    pub fn neighbors_within_bounds(&self) -> impl Iterator<Item = Self> {
        let maybe_iter = match self {
            &Self::OnMars(coord) => Some(
                coord
                    .neighbors_within_bounds()
                    .map(|x| TileLocation::OnMars(x)),
            ),
            &Self::OffMars(_) => None,
        };
        maybe_iter.into_iter().flatten()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct EmptyLocation(TileLocation);

impl From<TileLocation> for EmptyLocation {
    #[inline]
    fn from(location: TileLocation) -> Self {
        Self(location)
    }
}

impl From<EmptyLocation> for TileLocation {
    #[inline]
    fn from(empty_location: EmptyLocation) -> Self {
        empty_location.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileStatus {
    Empty(EmptyLocation),
    Ocean(TileLocation),
    City(TileLocation, CityKind, PlayerId),
    Greenery(TileLocation, PlayerId),
    SpecialTile(TileLocation, SpecialTile, PlayerId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarsBoard {
    pub board_name: String,

    pub spaces: HashMap<TileLocation, BoardSpace>,

    pub cities: HashMap<TileLocation, (CityKind, PlayerId)>,
    pub oceans: HashSet<Coordinates>,
    pub greeneries: HashMap<Coordinates, PlayerId>,
    pub special_tiles: HashMap<Coordinates, (SpecialTile, PlayerId)>,

    pub oxygen: usize,
    pub temperature: isize,
}

impl MarsBoard {
    const DEFAULT_OCEAN_ADJACENCY_MEGACREDITS: usize = 2;

    pub fn new(
        board_name: String,
        spaces: HashMap<TileLocation, BoardSpace>,
        cities: HashMap<TileLocation, (CityKind, PlayerId)>,
        oceans: HashSet<Coordinates>,
        greeneries: HashMap<Coordinates, PlayerId>,
        special_tiles: HashMap<Coordinates, (SpecialTile, PlayerId)>,
        oxygen: usize,
        temperature: isize,
    ) -> Self {
        // Ensure no board location is marked as occupied by two different tile types.
        let mut occupied_locations: HashSet<TileLocation> = HashSet::new();
        occupied_locations.extend(cities.keys().cloned());
        occupied_locations.extend(oceans.iter().map(|x| TileLocation::OnMars(*x)));
        occupied_locations.extend(greeneries.keys().map(|x| TileLocation::OnMars(*x)));
        occupied_locations.extend(special_tiles.keys().map(|x| TileLocation::OnMars(*x)));
        assert_eq!(
            occupied_locations.len(),
            cities.len() + oceans.len() + greeneries.len() + special_tiles.len()
        );

        Self {
            board_name,
            spaces,
            cities,
            oceans,
            greeneries,
            special_tiles,
            oxygen,
            temperature,
        }
    }

    pub fn get_tile_status(&self, location: &TileLocation) -> TileStatus {
        let city_status = self.cities.get(&location).map(|(city_kind, player_id)| {
            TileStatus::City(location.clone(), *city_kind, *player_id)
        });
        city_status.unwrap_or_else(|| {
            match &location {
                TileLocation::OffMars(_) => {
                    // By this point, we know two things:
                    // - The location is off Mars.
                    // - There is no city at the given location.
                    // Since the only things that can be placed off Mars are cities,
                    // we know that the tile status for that location must be empty.
                    TileStatus::Empty(location.clone().into())
                }
                TileLocation::OnMars(coordinates) => {
                    let ocean_status = self
                        .oceans
                        .get(coordinates)
                        .map(|_| TileStatus::Ocean(location.clone()));

                    ocean_status.unwrap_or_else(|| {
                        let greenery_status = self
                            .greeneries
                            .get(coordinates)
                            .map(|player_id| TileStatus::Greenery(location.clone(), *player_id));

                        greenery_status.unwrap_or_else(|| {
                            let special_tile_status =
                                self.special_tiles
                                    .get(coordinates)
                                    .map(|(tile, player_id)| {
                                        TileStatus::SpecialTile(
                                            location.clone(),
                                            tile.clone(),
                                            *player_id,
                                        )
                                    });

                            special_tile_status
                                .unwrap_or_else(|| TileStatus::Empty(location.clone().into()))
                        })
                    })
                }
            }
        })
    }

    pub fn count_adjacent_oceans(&self, empty_location: &EmptyLocation) -> usize {
        let location = &empty_location.0;

        self.get_neighbor_tile_status(location)
            .filter(|status| matches!(status, TileStatus::Ocean(_)))
            .count()
    }

    pub fn get_neighbor_tile_status<'a>(
        &'a self,
        location: &TileLocation,
    ) -> impl Iterator<Item = TileStatus> + 'a {
        location
            .neighbors_within_bounds()
            .map(move |neighbor| self.get_tile_status(&neighbor))
    }

    pub fn get_placement_bonuses(&self, empty_location: &EmptyLocation) -> Vec<ImmediateImpact> {
        let adjacent_oceans = self.count_adjacent_oceans(empty_location);
        let ocean_adjacency_megacredits =
            adjacent_oceans * Self::DEFAULT_OCEAN_ADJACENCY_MEGACREDITS;

        let board_space = self
            .spaces
            .get(&empty_location.0)
            .expect("Tile location did not existon this board.");
        let mut placement_bonuses = board_space.placement_bonus.clone();

        let mut coalesced = false;
        for bonus in placement_bonuses.iter_mut() {
            let total_megacredits = match bonus {
                &mut ImmediateImpact::GainResource(Resource::Megacredits, megacredits) => {
                    Some(megacredits + ocean_adjacency_megacredits)
                }
                _ => None,
            };

            match total_megacredits {
                Some(megacredits) => {
                    coalesced = true;
                    bonus.clone_from(&ImmediateImpact::GainResource(
                        Resource::Megacredits,
                        megacredits,
                    ));
                    break;
                }
                _ => {}
            }
        }

        if !coalesced {
            placement_bonuses.push(ImmediateImpact::GainResource(
                Resource::Megacredits,
                ocean_adjacency_megacredits,
            ));
        }

        placement_bonuses
    }

    pub fn can_place_city(
        &mut self,
        player: &mut PlayerState,
        empty_location: EmptyLocation,
        city_kind: CityKind,
        location_restrictions: &[LocationRestriction],
    ) -> Option<()> {
        let location = &empty_location.0;

        let mut adjacent_tiles_of_any_kind: usize = 0;
        let mut adjacent_greeneries: usize = 0;
        let mut adjacent_cities: usize = 0;
        let mut adjacent_owned_tiles: usize = 0;

        for status in self.get_neighbor_tile_status(location) {
            if !matches!(status, TileStatus::Empty(_)) {
                adjacent_tiles_of_any_kind += 1;
            }

            match status {
                TileStatus::City(_, _, _) => {
                    adjacent_cities += 1;
                }
                TileStatus::Greenery(_, _) => {
                    adjacent_greeneries += 1;
                }
                TileStatus::Empty(_)
                | TileStatus::Ocean(_)
                | TileStatus::SpecialTile(_, _, _) => {}
            }

            adjacent_owned_tiles += match status {
                TileStatus::City(_, _, owner_id)
                | TileStatus::Greenery(_, owner_id)
                | TileStatus::SpecialTile(_, _, owner_id) => {
                    if owner_id == player.player_id {
                        1
                    } else {
                        0
                    }
                }
                _ => 0,
            }
        }

        let board_space = self.spaces.get(location).unwrap();
        for restriction in location_restrictions {
            match restriction {
                LocationRestriction::LandTile => {
                    let board_space = self.spaces.get(location).unwrap();
                    if !board_space.is_land() {
                        return None;
                    }
                },
                LocationRestriction::ReservedForOcean => {
                    if !board_space.is_reserved_for_ocean() {
                        return None;
                    }
                },
                LocationRestriction::OnSteelOrTitaniumPlacementBonus => {
                    let is_on_metal_placement_bonus = board_space.placement_bonus
                        .iter()
                        .any(|impact| {
                            matches!(
                                impact,
                                ImmediateImpact::GainResource(Resource::Steel, _)
                                | ImmediateImpact::GainResource(Resource::Titanium, _)
                            )
                        });
                    if !is_on_metal_placement_bonus {
                        return None;
                    }
                },
                LocationRestriction::AtSpecialLocation(special_location) => {
                    // TODO: Handle placing volcanic area city / Noctis City
                    //       on maps that don't have such tiles.
                    let has_matching_designation = board_space.designations
                        .iter()
                        .any(|d| matches!(d, Designation::Special(s) if s == special_location));
                    if !has_matching_designation {
                        return None;
                    }
                }
                LocationRestriction::AdjacentToOwnedTile => {
                    if adjacent_owned_tiles == 0 {
                        return None;
                    }
                }
                LocationRestriction::AdjacentToOwnedTileIfAble => unimplemented!(),
                LocationRestriction::NotNextToAnyOtherTile => {
                    if adjacent_tiles_of_any_kind > 0 {
                        return None;
                    }
                }
                LocationRestriction::NotNextToACity => {
                    if adjacent_cities > 0 {
                        return None;
                    }
                }
                LocationRestriction::NextToACity => {
                    if adjacent_cities < 1 {
                        return None;
                    }
                }
                LocationRestriction::NextToAtLeastTwoCities => {
                    if adjacent_cities < 2 {
                        return None;
                    }
                }
                LocationRestriction::NextToAGreenery => {
                    if adjacent_greeneries < 1 {
                        return None;
                    }
                }
            }
        }

        let existing_tile = self
            .cities
            .insert(empty_location.0, (city_kind, player.player_id));
        assert!(existing_tile.is_none());

        Some(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Designation {
    Land,
    ReservedForOcean,
    VolcanicArea,
    NonMarsTile,
    Special(SpecialLocation),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardSpace {
    pub name: Option<String>,
    pub location: TileLocation,
    pub designations: Vec<Designation>,
    pub placement_bonus: Vec<ImmediateImpact>,
}

impl BoardSpace {
    pub fn new(
        name: Option<String>,
        location: TileLocation,
        designations: Vec<Designation>,
        placement_bonus: Vec<ImmediateImpact>,
    ) -> Self {
        BoardSpace {
            name,
            location,
            designations,
            placement_bonus,
        }
    }

    #[inline]
    pub fn new_on_mars<CoordT: Into<Coordinates>>(
        mars_coordinates: CoordT,
        designations: Vec<Designation>,
        placement_bonus: Vec<ImmediateImpact>,
    ) -> BoardSpace {
        BoardSpace::new(
            None,
            TileLocation::OnMars(mars_coordinates.into()),
            designations,
            placement_bonus,
        )
    }

    #[inline]
    pub fn new_land_on_mars<CoordT: Into<Coordinates>>(
        mars_coordinates: CoordT,
        placement_bonus: Vec<ImmediateImpact>,
    ) -> BoardSpace {
        BoardSpace::new_on_mars(mars_coordinates, vec![Designation::Land], placement_bonus)
    }

    #[inline]
    pub fn new_non_bonus_land_on_mars<CoordT: Into<Coordinates>>(
        mars_coordinates: CoordT,
    ) -> BoardSpace {
        BoardSpace::new_land_on_mars(mars_coordinates, vec![])
    }

    #[inline]
    pub fn new_resource_bonus_land_on_mars<CoordT: Into<Coordinates>>(
        mars_coordinates: CoordT,
        resource: Resource,
        count: usize,
    ) -> BoardSpace {
        BoardSpace::new_land_on_mars(
            mars_coordinates,
            vec![ImmediateImpact::GainResource(resource, count)],
        )
    }

    #[inline]
    pub fn new_card_draw_land_on_mars<CoordT: Into<Coordinates>>(
        mars_coordinates: CoordT,
        card_count: usize,
    ) -> BoardSpace {
        BoardSpace::new_land_on_mars(
            mars_coordinates,
            vec![ImmediateImpact::DrawCard(card_count)],
        )
    }

    #[inline]
    pub fn new_ocean_on_mars<CoordT: Into<Coordinates>>(
        mars_coordinates: CoordT,
        placement_bonus: Vec<ImmediateImpact>,
    ) -> BoardSpace {
        BoardSpace::new(
            None,
            TileLocation::OnMars(mars_coordinates.into()),
            vec![Designation::ReservedForOcean],
            placement_bonus,
        )
    }

    #[inline]
    pub fn new_resource_bonus_ocean_on_mars<CoordT: Into<Coordinates>>(
        mars_coordinates: CoordT,
        resource: Resource,
        count: usize,
    ) -> BoardSpace {
        BoardSpace::new_ocean_on_mars(
            mars_coordinates,
            vec![ImmediateImpact::GainResource(resource, count)],
        )
    }

    #[inline]
    pub fn new_card_draw_ocean_on_mars<CoordT: Into<Coordinates>>(
        mars_coordinates: CoordT,
        card_count: usize,
    ) -> BoardSpace {
        BoardSpace::new_ocean_on_mars(
            mars_coordinates,
            vec![ImmediateImpact::DrawCard(card_count)],
        )
    }

    #[inline]
    pub fn is_land(&self) -> bool {
        self.designations
            .iter()
            .any(|d| matches!(d, Designation::Land))
    }

    #[inline]
    pub fn is_reserved_for_ocean(&self) -> bool {
        self.designations
            .iter()
            .any(|d| matches!(d, Designation::ReservedForOcean))
    }
}

pub fn make_standard_non_mars_board_spaces() -> Vec<BoardSpace> {
    vec![
        BoardSpace::new(
            Some("Phobos Space Haven".into()),
            TileLocation::OffMars(SpecialLocation::PhobosSpaceHaven),
            vec![
                Designation::NonMarsTile,
                Designation::Special(SpecialLocation::PhobosSpaceHaven),
            ],
            vec![],
        ),
        BoardSpace::new(
            Some("Ganymede Colony".into()),
            TileLocation::OffMars(SpecialLocation::GanymedeColony),
            vec![
                Designation::NonMarsTile,
                Designation::Special(SpecialLocation::GanymedeColony),
            ],
            vec![],
        ),
    ]
}

pub fn make_base_game_board() -> MarsBoard {
    let board_name = "Tharsis".into();
    let oxygen = 0usize;
    let temperature = -30isize;

    let mut spaces = make_standard_non_mars_board_spaces();
    spaces.extend_from_slice(&[
        // top-left edge = first top-rightward column
        BoardSpace::new(
            Some("Arsia Mons".into()),
            TileLocation::OnMars((0, 0).into()),
            vec![
                Designation::Land,
                Designation::Special(SpecialLocation::VolcanicArea),
            ],
            vec![ImmediateImpact::GainResource(Resource::Plants, 2)],
        ),
        BoardSpace::new(
            Some("Pavonis Mons".into()),
            TileLocation::OnMars((1, 0).into()),
            vec![
                Designation::Land,
                Designation::Special(SpecialLocation::VolcanicArea),
            ],
            vec![
                ImmediateImpact::GainResource(Resource::Plants, 1),
                ImmediateImpact::GainResource(Resource::Titanium, 1),
            ],
        ),
        BoardSpace::new(
            Some("Ascraeus Mons".into()),
            TileLocation::OnMars((2, 0).into()),
            vec![
                Designation::Land,
                Designation::Special(SpecialLocation::VolcanicArea),
            ],
            vec![ImmediateImpact::DrawCard(1)],
        ),
        BoardSpace::new_non_bonus_land_on_mars((3, 0)),
        BoardSpace::new_resource_bonus_land_on_mars((4, 0), Resource::Steel, 2),
        //
        // second top-rightward column
        BoardSpace::new_resource_bonus_land_on_mars((0, -1), Resource::Plants, 1),
        BoardSpace::new_resource_bonus_land_on_mars((1, -1), Resource::Plants, 2),
        BoardSpace::new_resource_bonus_land_on_mars((2, -1), Resource::Plants, 1),
        BoardSpace::new_non_bonus_land_on_mars((3, -1)),
        BoardSpace::new(
            Some("Tharsis Tholus".into()),
            TileLocation::OnMars((4, -1).into()),
            vec![
                Designation::Land,
                Designation::Special(SpecialLocation::VolcanicArea),
            ],
            vec![ImmediateImpact::GainResource(Resource::Steel, 1)],
        ),
        BoardSpace::new_resource_bonus_ocean_on_mars((5, -1), Resource::Steel, 2),
        //
        // third top-rightward column
        BoardSpace::new_non_bonus_land_on_mars((0, -2)),
        BoardSpace::new_resource_bonus_land_on_mars((1, -2), Resource::Plants, 2),
        BoardSpace::new(
            Some("Noctis City".into()),
            TileLocation::OnMars((2, -2).into()),
            vec![
                Designation::Land,
                Designation::Special(SpecialLocation::NoctisCity),
            ],
            vec![ImmediateImpact::GainResource(Resource::Plants, 2)],
        ),
        BoardSpace::new_resource_bonus_land_on_mars((3, -2), Resource::Plants, 1),
        BoardSpace::new_non_bonus_land_on_mars((4, -2)),
        BoardSpace::new_non_bonus_land_on_mars((5, -2)),
        BoardSpace::new_non_bonus_land_on_mars((6, -2)),
        //
        // fourth top-rightward column
        BoardSpace::new_resource_bonus_land_on_mars((0, -3), Resource::Steel, 2),
        BoardSpace::new_non_bonus_land_on_mars((1, -3)),
        BoardSpace::new_resource_bonus_land_on_mars((2, -3), Resource::Plants, 1),
        BoardSpace::new_resource_bonus_ocean_on_mars((3, -3), Resource::Plants, 2),
        BoardSpace::new_resource_bonus_land_on_mars((4, -3), Resource::Plants, 1),
        BoardSpace::new_non_bonus_land_on_mars((5, -3)),
        BoardSpace::new_non_bonus_land_on_mars((6, -3)),
        BoardSpace::new_card_draw_ocean_on_mars((7, -3), 1),
        //
        // fifth top-rightward column -- bottom-left map corner
        BoardSpace::new_resource_bonus_land_on_mars((0, -4), Resource::Steel, 1),
        BoardSpace::new_non_bonus_land_on_mars((1, -4)),
        BoardSpace::new_non_bonus_land_on_mars((2, -4)),
        BoardSpace::new_resource_bonus_land_on_mars((3, -4), Resource::Plants, 1),
        BoardSpace::new_resource_bonus_ocean_on_mars((4, -4), Resource::Plants, 2),
        BoardSpace::new_resource_bonus_land_on_mars((5, -4), Resource::Plants, 2),
        BoardSpace::new_non_bonus_land_on_mars((6, -4)),
        BoardSpace::new_non_bonus_land_on_mars((7, -4)),
        BoardSpace::new_ocean_on_mars((8, -4), vec![]),
        //
        // sixth top-rightward column -- starts with second tile on the bottom edge
        BoardSpace::new_resource_bonus_land_on_mars((1, -5), Resource::Steel, 2),
        BoardSpace::new_non_bonus_land_on_mars((2, -5)),
        BoardSpace::new_non_bonus_land_on_mars((3, -5)),
        BoardSpace::new_resource_bonus_land_on_mars((4, -5), Resource::Plants, 1),
        BoardSpace::new_resource_bonus_ocean_on_mars((5, -5), Resource::Plants, 2),
        BoardSpace::new_resource_bonus_land_on_mars((6, -5), Resource::Plants, 1),
        BoardSpace::new_non_bonus_land_on_mars((7, -5)),
        BoardSpace::new_card_draw_ocean_on_mars((8, -5), 2),
        //
        // seventh top-rightward column -- starts with third tile on the bottom edge
        BoardSpace::new_non_bonus_land_on_mars((2, -6)),
        BoardSpace::new_card_draw_land_on_mars((3, -6), 1),
        BoardSpace::new_non_bonus_land_on_mars((4, -6)),
        BoardSpace::new_resource_bonus_ocean_on_mars((5, -6), Resource::Plants, 1),
        BoardSpace::new_resource_bonus_land_on_mars((6, -6), Resource::Plants, 2),
        BoardSpace::new_resource_bonus_land_on_mars((7, -6), Resource::Plants, 1),
        BoardSpace::new_resource_bonus_land_on_mars((8, -6), Resource::Steel, 1),
        //
        // eighth top-rightward column -- starts with fourth tile on the bottom edge
        BoardSpace::new_non_bonus_land_on_mars((3, -7)),
        BoardSpace::new_non_bonus_land_on_mars((4, -7)),
        BoardSpace::new_resource_bonus_land_on_mars((5, -7), Resource::Plants, 1),
        BoardSpace::new_resource_bonus_ocean_on_mars((6, -7), Resource::Plants, 1),
        BoardSpace::new_resource_bonus_land_on_mars((7, -7), Resource::Plants, 2),
        BoardSpace::new_resource_bonus_ocean_on_mars((8, -7), Resource::Plants, 2),
        //
        // last top-rightward column -- lower-right edge
        BoardSpace::new_resource_bonus_ocean_on_mars((4, -8), Resource::Titanium, 2),
        BoardSpace::new_resource_bonus_land_on_mars((5, -8), Resource::Titanium, 1),
        BoardSpace::new_non_bonus_land_on_mars((6, -8)),
        BoardSpace::new_resource_bonus_ocean_on_mars((7, -8), Resource::Plants, 1),
        BoardSpace::new_resource_bonus_land_on_mars((8, -8), Resource::Plants, 2),
    ]);

    MarsBoard::new(
        board_name,
        spaces
            .drain(..)
            .map(|tile| (tile.location.clone(), tile))
            .collect(),
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
        oxygen,
        temperature,
    )
}
