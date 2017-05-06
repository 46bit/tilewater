use std::collections::*;
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tile {
    Building {
        building: Building,
        entryway_coord: Coord2,
    },
    Entrance {
        road_coord: Coord2,
        building_coord: Coord2,
        orientation: Orientation,
    },
    Paving {
        entryway_coords: HashSet<Coord2>,
        paving_coords: HashSet<Coord2>,
    },
    Rails {
        rail_coords: HashSet<Coord2>,
        orientation: Orientation,
        is_station: bool,
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
            Tile::Paving { .. } => write!(f, "█"),
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
    dimensions: Coord2,
    tiles: HashMap<Coord2, Tile>,
}

impl TileMap {
    pub fn new(dimensions: Coord2) -> TileMap {
        let mut tiles = HashMap::new();
        tiles.insert(Coord2 {
                         x: dimensions.x / 2,
                         y: 0,
                     },
                     Tile::Paving {
                         entryway_coords: HashSet::new(),
                         paving_coords: HashSet::new(),
                     });
        TileMap { dimensions, tiles }
    }

    pub fn height(&self) -> u64 {
        self.dimensions.y
    }

    pub fn width(&self) -> u64 {
        self.dimensions.x
    }

    pub fn build(&mut self, location: Coord2, building: Building) {
        let (road_coord, entryway_coord, orientation) = self.nsew_entryways(location)[0];
        self.tiles
            .insert(entryway_coord,
                    Tile::Entrance {
                        road_coord: road_coord,
                        building_coord: location,
                        orientation: orientation,
                    });
        self.tiles
            .insert(location,
                    Tile::Building {
                        building: building,
                        entryway_coord: entryway_coord,
                    });
    }

    pub fn pave(&mut self, location: Coord2) {
        self.tiles
            .insert(location,
                    Tile::Paving {
                        entryway_coords: HashSet::new(),
                        paving_coords: HashSet::new(),
                    });
    }

    pub fn rail(&mut self, location: Coord2) {
        self.tiles
            .insert(location,
                    Tile::Rails {
                        rail_coords: HashSet::new(),
                        orientation: Orientation::Horizontal,
                        is_station: false,
                    });
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
        !self.nsew_nearby_roads(location).is_empty()
    }

    pub fn can_pave(&self, location: Coord2) -> bool {
        let mut has_road_neighbour = false;
        for neighbour in location.neighbours() {
            // Must not be:
            // - Building
            // - Entryway
            // - Railway
            // At least for now, anything new we add will also count.
            match self.get(neighbour) {
                None => {}
                Some(&Tile::Paving { .. }) => {
                    has_road_neighbour = true;
                }
                _ => return false,
            }
        }
        has_road_neighbour
    }

    // pub fn can_rail(&self, location: Coord2) -> bool {}

    fn nsew_nearby_roads(&self, location: Coord2) -> Vec<Coord2> {
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

    fn nsew_entryways(&self, location: Coord2) -> Vec<(Coord2, Coord2, Orientation)> {
        let nsew_nearby_roads = self.nsew_nearby_roads(location);
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
