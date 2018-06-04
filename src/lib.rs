extern crate piston_window;
extern crate rand;
extern crate rayon;
extern crate uuid;

mod agents;
mod map;
mod render_to_piston;
mod routing;
mod tile;

pub use agents::*;
pub use map::*;
pub use render_to_piston::*;
pub use routing::*;
pub use tile::*;

use std::fmt;
use std::ops::{Add, Mul, Sub};
use piston_window::Key;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Coord2 {
    pub x: u64,
    pub y: u64,
}

impl Coord2 {
    pub fn flip(&self) -> Coord2 {
        Coord2 {
            x: self.y,
            y: self.x,
        }
    }

    pub fn neighbours(&self) -> Vec<Coord2> {
        let mut neighbours = Vec::with_capacity(4);
        if self.y > 0 {
            let north = Coord2 {
                x: self.x,
                y: self.y - 1,
            };
            neighbours.push(north);
        }
        if self.x > 0 {
            let west = Coord2 {
                x: self.x - 1,
                y: self.y,
            };
            neighbours.push(west);
        }
        let south = Coord2 {
            x: self.x,
            y: self.y + 1,
        };
        neighbours.push(south);
        let east = Coord2 {
            x: self.x + 1,
            y: self.y,
        };
        neighbours.push(east);
        neighbours
    }
}

impl Add for Coord2 {
    type Output = Coord2;

    fn add(self, other: Coord2) -> Coord2 {
        Coord2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Coord2 {
    type Output = Coord2;

    fn sub(self, other: Coord2) -> Coord2 {
        Coord2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

// Allows multiplying all components of `Coord2` by a `u64` scalar.
impl<T> Mul<T> for Coord2
where
    u64: Mul<T, Output = u64>,
    T: Clone,
{
    type Output = Coord2;

    fn mul(self, rhs: T) -> Coord2 {
        Coord2 {
            x: self.x * rhs.clone(),
            y: self.y * rhs,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Orientation {
    Vertical,
    Horizontal,
}

impl Orientation {
    pub fn from_coord2(location: Coord2) -> Option<Orientation> {
        if location.x == 0 && location.y > 0 {
            Some(Orientation::Vertical)
        } else if location.x > 0 && location.y == 0 {
            Some(Orientation::Horizontal)
        } else {
            None
        }
    }

    pub fn between_coord2s(a: Coord2, b: Coord2) -> Option<Orientation> {
        if a.x == b.x && a.y != b.y {
            Some(Orientation::Vertical)
        } else if a.x != b.x && a.y == b.y {
            Some(Orientation::Horizontal)
        } else {
            None
        }
    }

    pub fn as_offset(&self) -> Coord2 {
        if *self == Orientation::Vertical {
            Coord2 { x: 0, y: 1 }
        } else {
            Coord2 { x: 1, y: 0 }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub fn between_coord2s(a: Coord2, b: Coord2) -> Option<Direction> {
        if (a.x == b.x && a.y == b.y) || (a.x != b.x && a.y != b.y) {
            None
        } else if a.x > b.x {
            Some(Direction::West)
        } else if a.x < b.x {
            Some(Direction::East)
        } else if a.y > b.y {
            Some(Direction::North)
        } else if a.y < b.y {
            Some(Direction::South)
        } else {
            unreachable!();
        }
    }

    pub fn as_offset(&self) -> (i64, i64) {
        match *self {
            Direction::North => (0, -1),
            Direction::East => (1, 0),
            Direction::South => (0, 1),
            Direction::West => (-1, 0),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Path {
    pub start: Coord2,
    pub orientation: Orientation,
    pub length: u64,
}

impl Path {
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn contains(&self, target: Coord2) -> bool {
        for coord in *self {
            if coord == target {
                return true;
            }
        }
        false
    }

    pub fn neighbours(&self) -> Vec<Coord2> {
        // @TODO: This can be optimised by using cells to left/right or north/south based
        // upon orientation, if needed.
        self.into_iter()
            .flat_map(|c| c.neighbours())
            .filter(|c| !self.contains(*c))
            .collect()
    }

    // pub fn neighbours_diag(&self) -> Vec<Coord2> {
    //     let before = self.start - self.orientation.as_offset();
    //     let expanded_path = Path {
    //         start: before,
    //         orientation: self.orientation,
    //         length: self.length + 1,
    //     };

    //     let after = self.end() + offset;
    //     let flipped_offset = offset.flip();
    //     before + flipped_offset
    //     before - flipped_offset
    //     after + flipped_offset
    //     after - flipped_offset

    //     let neighbours = self.neighbours();
    // }

    pub fn end(&self) -> Coord2 {
        self.start + self.orientation.as_offset() * self.length
    }

    pub fn advance(self) -> Option<Path> {
        let new_path = Path {
            start: self.start + self.orientation.as_offset(),
            orientation: self.orientation,
            length: self.length.saturating_sub(1),
        };
        if new_path.length > 0 {
            Some(new_path)
        } else {
            None
        }
    }
}

impl IntoIterator for Path {
    type Item = Coord2;
    type IntoIter = ::std::vec::IntoIter<Coord2>;

    fn into_iter(self) -> Self::IntoIter {
        let mut rem = self;
        let mut coords = vec![];
        while self.length > 0 {
            coords.push(rem.start);
            if let Some(r) = rem.advance() {
                rem = r;
            } else {
                break;
            }
        }
        coords.into_iter()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Building {
    House,
    Saloon,
    Factory,
    GeneralStore,
    TrainStation,
}

impl Building {
    pub fn from_key(code: Key) -> Option<Building> {
        match code {
            Key::H => Some(Building::House),
            Key::S => Some(Building::Saloon),
            Key::F => Some(Building::Factory),
            Key::G => Some(Building::GeneralStore),
            Key::R => Some(Building::TrainStation),
            _ => None,
        }
    }

    pub fn from_code(code: char) -> Option<Building> {
        match code {
            'h' => Some(Building::House),
            's' => Some(Building::Saloon),
            'f' => Some(Building::Factory),
            'g' => Some(Building::GeneralStore),
            'r' => Some(Building::TrainStation),
            _ => None,
        }
    }

    pub fn code(&self) -> char {
        match *self {
            Building::House => 'h',
            Building::Saloon => 's',
            Building::Factory => 'f',
            Building::GeneralStore => 'g',
            Building::TrainStation => 'r',
        }
    }
}

impl fmt::Display for Building {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
