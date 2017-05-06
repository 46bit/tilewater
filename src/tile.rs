use std::collections::*;
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tile {
    Building {
        building: Building,
        entryway_pos: Coord2,
    },
    Entrance {
        orientation: Orientation,
        road_pos: Coord2,
        building_pos: Coord2,
    },
    Paving {
        entryways_pos: HashSet<Coord2>,
        pavings_pos: HashSet<Coord2>,
    },
    Rails {
        is_station: bool,
        orientation: Orientation,
        rails_pos: HashSet<Coord2>,
    },
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Tile::Building { building, .. } => write!(f, "{}", building),
            Tile::Entrance { orientation, .. } => {
                match orientation {
                    Orientation::Vertical => write!(f, "|"),
                    Orientation::Horizontal => write!(f, "-"),
                }
            }
            Tile::Paving { .. } => write!(f, ":"),
            Tile::Rails { orientation, .. } => {
                match orientation {
                    Orientation::Vertical => write!(f, "‖"),
                    Orientation::Horizontal => write!(f, "="),
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct TileMap {
    pub cursor: Coord2,
    dimensions: Coord2,
    tiles: HashMap<Coord2, Tile>,
}

impl TileMap {
    pub fn new(dimensions: Coord2) -> TileMap {
        let cursor = Coord2 { x: 0, y: 0 };
        let mut tiles = HashMap::new();
        // @TODO: Instead spawn a station on the railway track, and an entranceway
        // - and maybe a road tile attached the entranceway, if the roadless entryway
        // causes coding problems.
        tiles.insert(Coord2 {
                         x: dimensions.x / 2,
                         y: 0,
                     },
                     Tile::Paving {
                         entryways_pos: HashSet::new(),
                         pavings_pos: HashSet::new(),
                     });
        TileMap {
            cursor,
            dimensions,
            tiles,
        }
    }

    pub fn height(&self) -> u64 {
        self.dimensions.y
    }

    pub fn width(&self) -> u64 {
        self.dimensions.x
    }

    pub fn build(&mut self, location: Coord2, building: Building) -> bool {
        // Find possible entryways.
        let possible_entryways = self.entryways_to_build_from(location);
        // Take the first road + entryway coordinates and its orientation.
        if possible_entryways.is_empty() {
            return false;
        }
        let (road_pos, entryway_pos, orientation) = possible_entryways[0];

        // Insert the new building.
        let building_tile = Tile::Building {
            building: building,
            entryway_pos: entryway_pos,
        };
        self.tiles.insert(location, building_tile);

        // Insert the new entrance.
        let entrance_tile = Tile::Entrance {
            road_pos: road_pos,
            building_pos: location,
            orientation: orientation,
        };
        self.tiles.insert(entryway_pos, entrance_tile);

        // Update road the entrance attaches, to record this new entryway.
        match self.tiles.get_mut(&road_pos) {
            Some(&mut Tile::Paving { ref mut entryways_pos, .. }) => {
                entryways_pos.insert(entryway_pos);
            }
            _ => unreachable!("Known paving tile was not present."),
        };

        true
    }

    pub fn pave(&mut self, location: Coord2) {
        let paving_tile = Tile::Paving {
            entryways_pos: HashSet::new(),
            pavings_pos: self.neighbouring_pavings(location),
        };
        self.tiles.insert(location, paving_tile);
    }

    pub fn rail(&mut self, location: Coord2) {
        let rail_tile = Tile::Rails {
            rails_pos: HashSet::new(),
            orientation: Orientation::Horizontal,
            is_station: false,
        };
        self.tiles.insert(location, rail_tile);
    }

    pub fn delete(&mut self, location: Coord2) {
        let tile_cloned = {
            let tile = match self.get(location) {
                Some(tile) => tile.clone(),
                None => return,
            };
            tile.clone()
        };
        match tile_cloned {
            Tile::Building { entryway_pos, .. } => {
                // Delete building and entrance.
                self.tiles.remove(&location);
                self.tiles.remove(&entryway_pos);
                // @TODO: Remove entryway record from paved tile.
            }
            Tile::Entrance { building_pos, .. } => {
                // Delete entrance and building.
                self.tiles.remove(&location);
                self.tiles.remove(&building_pos);
                // @TODO: Remove entryway record from paved tile.
                // @TODO: Merge with building implementation.
            }
            Tile::Paving { entryways_pos, .. } => {
                // Delete entrances, their buildings and this paving.
                for entryway_pos in entryways_pos {
                    self.delete(entryway_pos);
                }
                self.tiles.remove(&location);
                // @TODO: Remove record from neighbouring paved tiles.
            }
            Tile::Rails { .. } => {
                unimplemented!();
            }
        }
    }

    pub fn get(&self, location: Coord2) -> Option<&Tile> {
        self.tiles.get(&location)
    }

    pub fn is_bounded(&self, location: Coord2) -> bool {
        location.x < self.dimensions.x && location.y < self.dimensions.y
    }

    pub fn can_build(&self, location: Coord2) -> bool {
        for neighbour in location.neighbours() {
            // Must not be:
            // - Road
            // - Building
            // - Entryway
            // - Railway
            // At least for now, anything new we add will also count.
            if self.get(neighbour) != None {
                return false;
            }
        }
        // Must have a nearby road to build an entrance from.
        !self.roads_to_enter_from(location).is_empty()
    }

    pub fn can_pave(&self, location: Coord2) -> bool {
        for neighbour in location.neighbours() {
            // Must not be:
            // - Building
            // - Entryway
            // - Railway
            // At least for now, anything new we add will also count.
            match self.get(neighbour) {
                None |
                Some(&Tile::Paving { .. }) => {}
                _ => return false,
            }
        }
        !self.neighbouring_pavings(location).is_empty()
    }

    fn neighbouring_pavings(&self, location: Coord2) -> HashSet<Coord2> {
        location
            .neighbours()
            .into_iter()
            .filter(|l| match self.get(*l) {
                        Some(&Tile::Paving { .. }) => true,
                        _ => false,
                    })
            .collect()
    }

    // pub fn can_rail(&self, location: Coord2) -> bool {}

    fn roads_to_enter_from(&self, location: Coord2) -> Vec<Coord2> {
        let mut locations = Vec::with_capacity(4);
        if location.y >= 2 {
            let northward_road = location - Coord2 { x: 0, y: 2 };
            locations.push(northward_road);
        }
        let southward_road = location + Coord2 { x: 0, y: 2 };
        locations.push(southward_road);
        let eastward_road = location + Coord2 { x: 2, y: 0 };
        locations.push(eastward_road);
        if location.x >= 2 {
            let westward_road = location - Coord2 { x: 2, y: 0 };
            locations.push(westward_road);
        }
        locations
            .into_iter()
            .filter(|l| match self.get(*l) {
                        Some(&Tile::Paving { .. }) => true,
                        _ => false,
                    })
            .collect()
    }

    fn entryways_to_build_from(&self, location: Coord2) -> Vec<(Coord2, Coord2, Orientation)> {
        let nsew_nearby_roads = self.roads_to_enter_from(location);
        let mut entryway_coords = Vec::with_capacity(nsew_nearby_roads.len());
        for nsew_nearby_road in nsew_nearby_roads {
            let orientation = Orientation::between_coord2s(location, nsew_nearby_road)
                                  .expect("Nearby roads must not be the cell itself.");
            let entryway_coord;
            if nsew_nearby_road.x == location.x && nsew_nearby_road.y != location.y {
                if nsew_nearby_road.y < location.y {
                    entryway_coord = location - Orientation::Vertical.as_offset();
                } else if nsew_nearby_road.y > location.y {
                    entryway_coord = location + Orientation::Vertical.as_offset();
                } else {
                    unreachable!();
                }
            } else if nsew_nearby_road.y == location.y && nsew_nearby_road.x != location.x {
                if nsew_nearby_road.x < location.x {
                    entryway_coord = location - Orientation::Horizontal.as_offset();
                } else if nsew_nearby_road.x > location.x {
                    entryway_coord = location + Orientation::Horizontal.as_offset();
                } else {
                    unreachable!();
                }
            } else {
                continue;
            }
            entryway_coords.push((nsew_nearby_road, entryway_coord, orientation));
        }
        entryway_coords
    }
}

impl fmt::Display for TileMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in 0..self.dimensions.y {
            for x in 0..self.dimensions.x {
                if let Some(tile) = self.get(Coord2 { x, y }) {
                    write!(f, "{}", tile)?;
                } else {
                    write!(f, " ")?;
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}
