use super::*;
use std::collections::*;

#[derive(Clone, Debug)]
pub struct Map {
    pub cursor: Coord2,
    dimensions: Coord2,
    tiles: HashMap<Coord2, Tile>,
    pub buildings: HashMap<Building, Vec<Coord2>>,
}

impl Map {
    pub fn new(dimensions: Coord2) -> Map {
        let cursor = Coord2 { x: 0, y: 0 };
        let mut tiles = HashMap::new();
        let buildings = HashMap::new();
        // @TODO: Instead spawn a station on the railway track, and an entranceway
        // - and maybe a road tile attached the entranceway, if the roadless entryway
        // causes coding problems.
        tiles.insert(
            Coord2 {
                x: dimensions.x / 2,
                y: 0,
            },
            Tile::Paving(PavingTile {
                entryways_pos: HashSet::new(),
                pavings_pos: HashSet::new(),
            }),
        );
        Map {
            cursor,
            dimensions,
            tiles,
            buildings,
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
        let building_tile = Tile::Building(BuildingTile {
            building: building,
            entryway_pos: entryway_pos,
        });
        self.tiles.insert(location, building_tile);
        self.buildings
            .entry(building)
            .or_insert_with(|| Vec::new())
            .push(location);

        // Insert the new entrance.
        let entrance_tile = Tile::Entrance(EntranceTile {
            road_pos: road_pos,
            building_pos: location,
            orientation: orientation,
        });
        self.tiles.insert(entryway_pos, entrance_tile);

        // Update road the entrance attaches, to record this new entryway.
        self.tiles
            .get_mut(&road_pos)
            .and_then(Tile::as_paving_mut)
            .expect("Known paving tile was not present.")
            .entryways_pos
            .insert(entryway_pos);

        true
    }

    pub fn pave(&mut self, location: Coord2) {
        let neighbouring_pavings = self.neighbouring_pavings(location);
        for neighbouring_paving in &neighbouring_pavings {
            self.tiles
                .get_mut(neighbouring_paving)
                .and_then(Tile::as_paving_mut)
                .expect("Known paving tile was not present.")
                .pavings_pos
                .insert(location);
        }
        let paving_tile = Tile::Paving(PavingTile {
            entryways_pos: HashSet::new(),
            pavings_pos: neighbouring_pavings,
        });
        self.tiles.insert(location, paving_tile);
    }

    pub fn rail(&mut self, location: Coord2) {
        let rail_tile = Tile::Rails(RailsTile {
            rails_pos: HashSet::new(),
            orientation: Orientation::Horizontal,
            is_station: false,
        });
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
            Tile::Building(BuildingTile { entryway_pos, .. }) => {
                self.delete_building(location, entryway_pos);
                // @TODO: Remove entryway record from paved tile.
            }
            Tile::Entrance(EntranceTile { building_pos, .. }) => {
                self.delete_building(building_pos, location);
            }
            Tile::Paving(PavingTile {
                entryways_pos,
                pavings_pos,
            }) => {
                // Delete entrances, their buildings and this paving.
                for entryway_pos in entryways_pos {
                    self.delete(entryway_pos);
                }
                // Remove record from neighbouring paved tiles.
                for paving_pos in pavings_pos {
                    self.tiles
                        .get_mut(&paving_pos)
                        .and_then(Tile::as_paving_mut)
                        .expect("Known paving tile was not present.")
                        .pavings_pos
                        .remove(&location);
                }
                // Remove paving.
                self.tiles.remove(&location);
            }
            Tile::Rails { .. } => {
                unimplemented!();
            }
        }
    }

    fn delete_building(&mut self, building_pos: Coord2, entryway_pos: Coord2) {
        // Remove record of building entryway from its access paving.
        let paving_pos = match self.tiles.get(&entryway_pos) {
            Some(&Tile::Entrance(EntranceTile { road_pos, .. })) => road_pos,
            _ => panic!("Known entryway did not exist."),
        };
        match self.tiles.get_mut(&paving_pos) {
            Some(&mut Tile::Paving(PavingTile {
                ref mut entryways_pos,
                ..
            })) => {
                entryways_pos.remove(&entryway_pos);
            }
            _ => {
                // Somewhat more plausible to reach this case than warrants an
                // unreachable.
                panic!("Known paving did not exist.");
            }
        }

        // Remove record of building.
        let building = self
            .tiles
            .get(&building_pos)
            .and_then(Tile::as_building)
            .unwrap()
            .building;
        let buildings_of_type = self.buildings.get_mut(&building).unwrap();
        let index = buildings_of_type
            .iter()
            .position(|x| *x == building_pos)
            .unwrap();
        buildings_of_type.remove(index);

        // Remove building and entryway tiles.
        self.tiles.remove(&building_pos);
        self.tiles.remove(&entryway_pos);
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
                None | Some(&Tile::Paving { .. }) => {}
                _ => return false,
            }
        }
        true
    }

    fn neighbouring_pavings(&self, location: Coord2) -> HashSet<Coord2> {
        location
            .neighbours()
            .into_iter()
            .filter(|l| self.tiles.get(l).and_then(Tile::as_paving).is_some())
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

impl fmt::Display for Map {
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
