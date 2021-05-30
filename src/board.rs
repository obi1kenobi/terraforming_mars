use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    card::{ImmediateImpact, SpecialLocation},
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarsBoard {
    pub tiles: BTreeMap<TileLocation, BoardTile>,
    pub placed_oceans: usize,
    pub oxygen: usize,
    pub temperature: isize,
}

impl MarsBoard {
    pub fn new(
        tiles: BTreeMap<TileLocation, BoardTile>,
        placed_oceans: usize,
        oxygen: usize,
        temperature: isize,
    ) -> MarsBoard {
        MarsBoard {
            tiles,
            placed_oceans,
            oxygen,
            temperature,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileDesignation {
    Land,
    ReservedForOcean,
    VolcanicArea,
    NonMarsTile,
    Special(SpecialLocation),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardTile {
    pub name: Option<String>,
    pub location: TileLocation,
    pub designations: Vec<TileDesignation>,
    pub placement_bonus: Vec<ImmediateImpact>,
}

impl BoardTile {
    pub fn new(
        name: Option<String>,
        location: TileLocation,
        designations: Vec<TileDesignation>,
        placement_bonus: Vec<ImmediateImpact>,
    ) -> Self {
        BoardTile {
            name,
            location,
            designations,
            placement_bonus,
        }
    }

    #[inline]
    pub fn new_on_mars<CoordT: Into<Coordinates>>(
        mars_coordinates: CoordT,
        designations: Vec<TileDesignation>,
        placement_bonus: Vec<ImmediateImpact>,
    ) -> BoardTile {
        BoardTile::new(
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
    ) -> BoardTile {
        BoardTile::new_on_mars(
            mars_coordinates,
            vec![TileDesignation::Land],
            placement_bonus,
        )
    }

    #[inline]
    pub fn new_non_bonus_land_on_mars<CoordT: Into<Coordinates>>(
        mars_coordinates: CoordT,
    ) -> BoardTile {
        BoardTile::new_land_on_mars(mars_coordinates, vec![])
    }

    #[inline]
    pub fn new_resource_bonus_land_on_mars<CoordT: Into<Coordinates>>(
        mars_coordinates: CoordT,
        resource: Resource,
        count: usize,
    ) -> BoardTile {
        BoardTile::new_land_on_mars(
            mars_coordinates,
            vec![ImmediateImpact::GainResource(resource, count)],
        )
    }

    #[inline]
    pub fn new_card_draw_land_on_mars<CoordT: Into<Coordinates>>(
        mars_coordinates: CoordT,
        card_count: usize,
    ) -> BoardTile {
        BoardTile::new_land_on_mars(
            mars_coordinates,
            vec![ImmediateImpact::DrawCard(card_count)],
        )
    }

    #[inline]
    pub fn new_ocean_on_mars<CoordT: Into<Coordinates>>(
        mars_coordinates: CoordT,
        placement_bonus: Vec<ImmediateImpact>,
    ) -> BoardTile {
        BoardTile::new(
            None,
            TileLocation::OnMars(mars_coordinates.into()),
            vec![TileDesignation::ReservedForOcean],
            placement_bonus,
        )
    }

    #[inline]
    pub fn new_resource_bonus_ocean_on_mars<CoordT: Into<Coordinates>>(
        mars_coordinates: CoordT,
        resource: Resource,
        count: usize,
    ) -> BoardTile {
        BoardTile::new_ocean_on_mars(
            mars_coordinates,
            vec![ImmediateImpact::GainResource(resource, count)],
        )
    }

    #[inline]
    pub fn new_card_draw_ocean_on_mars<CoordT: Into<Coordinates>>(
        mars_coordinates: CoordT,
        card_count: usize,
    ) -> BoardTile {
        BoardTile::new_ocean_on_mars(
            mars_coordinates,
            vec![ImmediateImpact::DrawCard(card_count)],
        )
    }
}

pub fn make_standard_non_mars_tiles() -> Vec<BoardTile> {
    vec![
        BoardTile::new(
            Some("Phobos Space Haven".into()),
            TileLocation::OffMars(SpecialLocation::PhobosSpaceHaven),
            vec![
                TileDesignation::NonMarsTile,
                TileDesignation::Special(SpecialLocation::PhobosSpaceHaven),
            ],
            vec![],
        ),
        BoardTile::new(
            Some("Ganymede Colony".into()),
            TileLocation::OffMars(SpecialLocation::GanymedeColony),
            vec![
                TileDesignation::NonMarsTile,
                TileDesignation::Special(SpecialLocation::GanymedeColony),
            ],
            vec![],
        ),
    ]
}

pub fn make_standard_game_board() -> MarsBoard {
    let placed_oceans = 0usize;
    let oxygen = 0usize;
    let temperature = -30isize;

    let mut tiles = make_standard_non_mars_tiles();
    tiles.extend_from_slice(&[
        // top-left edge = first top-rightward column
        BoardTile::new(
            Some("Arsia Mons".into()),
            TileLocation::OnMars((0, 0).into()),
            vec![
                TileDesignation::Land,
                TileDesignation::Special(SpecialLocation::VolcanicArea),
            ],
            vec![ImmediateImpact::GainResource(Resource::Plants, 2)],
        ),
        BoardTile::new(
            Some("Pavonis Mons".into()),
            TileLocation::OnMars((1, 0).into()),
            vec![
                TileDesignation::Land,
                TileDesignation::Special(SpecialLocation::VolcanicArea),
            ],
            vec![
                ImmediateImpact::GainResource(Resource::Plants, 1),
                ImmediateImpact::GainResource(Resource::Titanium, 1),
            ],
        ),
        BoardTile::new(
            Some("Ascraeus Mons".into()),
            TileLocation::OnMars((2, 0).into()),
            vec![
                TileDesignation::Land,
                TileDesignation::Special(SpecialLocation::VolcanicArea),
            ],
            vec![ImmediateImpact::DrawCard(1)],
        ),
        BoardTile::new_non_bonus_land_on_mars((3, 0)),
        BoardTile::new_resource_bonus_land_on_mars((4, 0), Resource::Steel, 2),
        //
        // second top-rightward column
        BoardTile::new_resource_bonus_land_on_mars((0, -1), Resource::Plants, 1),
        BoardTile::new_resource_bonus_land_on_mars((1, -1), Resource::Plants, 2),
        BoardTile::new_resource_bonus_land_on_mars((2, -1), Resource::Plants, 1),
        BoardTile::new_non_bonus_land_on_mars((3, -1)),
        BoardTile::new(
            Some("Tharsis Tholus".into()),
            TileLocation::OnMars((4, -1).into()),
            vec![
                TileDesignation::Land,
                TileDesignation::Special(SpecialLocation::VolcanicArea),
            ],
            vec![ImmediateImpact::GainResource(Resource::Steel, 1)],
        ),
        BoardTile::new_resource_bonus_ocean_on_mars((5, -1), Resource::Steel, 2),
        //
        // third top-rightward column
        BoardTile::new_non_bonus_land_on_mars((0, -2)),
        BoardTile::new_resource_bonus_land_on_mars((1, -2), Resource::Plants, 2),
        BoardTile::new(
            Some("Noctis City".into()),
            TileLocation::OnMars((2, -2).into()),
            vec![
                TileDesignation::Land,
                TileDesignation::Special(SpecialLocation::NoctisCity),
            ],
            vec![ImmediateImpact::GainResource(Resource::Plants, 2)],
        ),
        BoardTile::new_resource_bonus_land_on_mars((3, -2), Resource::Plants, 1),
        BoardTile::new_non_bonus_land_on_mars((4, -2)),
        BoardTile::new_non_bonus_land_on_mars((5, -2)),
        BoardTile::new_non_bonus_land_on_mars((6, -2)),
        //
        // fourth top-rightward column
        BoardTile::new_resource_bonus_land_on_mars((0, -3), Resource::Steel, 2),
        BoardTile::new_non_bonus_land_on_mars((1, -3)),
        BoardTile::new_resource_bonus_land_on_mars((2, -3), Resource::Plants, 1),
        BoardTile::new_resource_bonus_ocean_on_mars((3, -3), Resource::Plants, 2),
        BoardTile::new_resource_bonus_land_on_mars((4, -3), Resource::Plants, 1),
        BoardTile::new_non_bonus_land_on_mars((5, -3)),
        BoardTile::new_non_bonus_land_on_mars((6, -3)),
        BoardTile::new_card_draw_ocean_on_mars((7, -3), 1),
        //
        // fifth top-rightward column -- bottom-left map corner
        BoardTile::new_resource_bonus_land_on_mars((0, -4), Resource::Steel, 1),
        BoardTile::new_non_bonus_land_on_mars((1, -4)),
        BoardTile::new_non_bonus_land_on_mars((2, -4)),
        BoardTile::new_resource_bonus_land_on_mars((3, -4), Resource::Plants, 1),
        BoardTile::new_resource_bonus_ocean_on_mars((4, -4), Resource::Plants, 2),
        BoardTile::new_resource_bonus_land_on_mars((5, -4), Resource::Plants, 2),
        BoardTile::new_non_bonus_land_on_mars((6, -4)),
        BoardTile::new_non_bonus_land_on_mars((7, -4)),
        BoardTile::new_ocean_on_mars((8, -4), vec![]),
        //
        // sixth top-rightward column -- starts with second tile on the bottom edge
        BoardTile::new_resource_bonus_land_on_mars((1, -5), Resource::Steel, 2),
        BoardTile::new_non_bonus_land_on_mars((2, -5)),
        BoardTile::new_non_bonus_land_on_mars((3, -5)),
        BoardTile::new_resource_bonus_land_on_mars((4, -5), Resource::Plants, 1),
        BoardTile::new_resource_bonus_ocean_on_mars((5, -5), Resource::Plants, 2),
        BoardTile::new_resource_bonus_land_on_mars((6, -5), Resource::Plants, 1),
        BoardTile::new_non_bonus_land_on_mars((7, -5)),
        BoardTile::new_card_draw_ocean_on_mars((8, -5), 2),
        //
        // seventh top-rightward column -- starts with third tile on the bottom edge
        BoardTile::new_non_bonus_land_on_mars((2, -6)),
        BoardTile::new_card_draw_land_on_mars((3, -6), 1),
        BoardTile::new_non_bonus_land_on_mars((4, -6)),
        BoardTile::new_resource_bonus_ocean_on_mars((5, -6), Resource::Plants, 1),
        BoardTile::new_resource_bonus_land_on_mars((6, -6), Resource::Plants, 2),
        BoardTile::new_resource_bonus_land_on_mars((7, -6), Resource::Plants, 1),
        BoardTile::new_resource_bonus_land_on_mars((8, -6), Resource::Steel, 1),
        //
        // eighth top-rightward column -- starts with fourth tile on the bottom edge
        BoardTile::new_non_bonus_land_on_mars((3, -7)),
        BoardTile::new_non_bonus_land_on_mars((4, -7)),
        BoardTile::new_resource_bonus_land_on_mars((5, -7), Resource::Plants, 1),
        BoardTile::new_resource_bonus_ocean_on_mars((6, -7), Resource::Plants, 1),
        BoardTile::new_resource_bonus_land_on_mars((7, -7), Resource::Plants, 2),
        BoardTile::new_resource_bonus_ocean_on_mars((8, -7), Resource::Plants, 2),
        //
        // last top-rightward column -- lower-right edge
        BoardTile::new_resource_bonus_ocean_on_mars((4, -8), Resource::Titanium, 2),
        BoardTile::new_resource_bonus_land_on_mars((5, -8), Resource::Titanium, 1),
        BoardTile::new_non_bonus_land_on_mars((6, -8)),
        BoardTile::new_resource_bonus_ocean_on_mars((7, -8), Resource::Plants, 1),
        BoardTile::new_resource_bonus_land_on_mars((8, -8), Resource::Plants, 2),
    ]);

    MarsBoard::new(
        tiles
            .drain(..)
            .map(|tile| (tile.location.clone(), tile))
            .collect(),
        placed_oceans,
        oxygen,
        temperature,
    )
}
