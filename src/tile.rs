use std::collections::*;
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildingTile {
    pub building: Building,
    pub entryway_pos: Coord2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntranceTile {
    pub orientation: Orientation,
    pub road_pos: Coord2,
    pub building_pos: Coord2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PavingTile {
    pub entryways_pos: HashSet<Coord2>,
    pub pavings_pos: HashSet<Coord2>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RailsTile {
    pub is_station: bool,
    pub orientation: Orientation,
    pub rails_pos: HashSet<Coord2>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tile {
    Building(BuildingTile),
    Entrance(EntranceTile),
    Paving(PavingTile),
    Rails(RailsTile),
}

impl Tile {
    // .get(pos).and_then(Tile::as_building).expect("Must be a building.")
    pub fn as_building(&self) -> Option<&BuildingTile> {
        match *self {
            Tile::Building(ref building_tile) => Some(building_tile),
            _ => None,
        }
    }

    pub fn as_building_mut(&mut self) -> Option<&mut BuildingTile> {
        match *self {
            Tile::Building(ref mut building_tile) => Some(building_tile),
            _ => None,
        }
    }

    pub fn as_entrance(&self) -> Option<&EntranceTile> {
        match *self {
            Tile::Entrance(ref entrance_tile) => Some(entrance_tile),
            _ => None,
        }
    }

    pub fn as_entrance_mut(&mut self) -> Option<&mut EntranceTile> {
        match *self {
            Tile::Entrance(ref mut entrance_tile) => Some(entrance_tile),
            _ => None,
        }
    }

    pub fn as_paving(&self) -> Option<&PavingTile> {
        match *self {
            Tile::Paving(ref paving_tile) => Some(paving_tile),
            _ => None,
        }
    }

    pub fn as_paving_mut(&mut self) -> Option<&mut PavingTile> {
        match *self {
            Tile::Paving(ref mut paving_tile) => Some(paving_tile),
            _ => None,
        }
    }

    pub fn as_rails(&self) -> Option<&RailsTile> {
        match *self {
            Tile::Rails(ref rails_tile) => Some(rails_tile),
            _ => None,
        }
    }

    pub fn as_rails_mut(&mut self) -> Option<&mut RailsTile> {
        match *self {
            Tile::Rails(ref mut rails_tile) => Some(rails_tile),
            _ => None,
        }
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Tile::Building(BuildingTile { building, .. }) => write!(f, "{}", building),
            Tile::Entrance(EntranceTile { orientation, .. }) => {
                match orientation {
                    Orientation::Vertical => write!(f, "|"),
                    Orientation::Horizontal => write!(f, "-"),
                }
            }
            Tile::Paving(PavingTile { .. }) => write!(f, ":"),
            Tile::Rails(RailsTile { orientation, .. }) => {
                match orientation {
                    Orientation::Vertical => write!(f, "‖"),
                    Orientation::Horizontal => write!(f, "="),
                }
            }
        }
    }
}
