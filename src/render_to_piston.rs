use std::sync::{Arc, RwLock};
use piston_window::*;
use rand::*;
use super::*;

const PPU: u64 = 10;

pub enum Cmd {
    Up,
    Down,
    Left,
    Right,
    Pave,
    Build(Building),
    Delete,
}

pub struct RenderToPiston {
    agent: Coord2,
    agent_goal: Coord2,
    window: PistonWindow,
    map: Arc<RwLock<Map>>,
}

impl RenderToPiston {
    pub fn new(window: PistonWindow, map: Arc<RwLock<Map>>) -> RenderToPiston {
        let agent = Coord2 { x: 40, y: 2 };
        let agent_goal = Coord2 { x: 40, y: 8 };
        RenderToPiston {
            agent,
            agent_goal,
            window,
            map,
        }
    }

    pub fn render_loop(&mut self) {
        while let Some(e) = self.window.next() {
            if let Event::Input(Input::Text(ref t)) = e {
                for c in t.chars() {
                    self.char_command(c);
                }
            }

            if let Event::Render(_) = e {
                match route(&self.map.read().unwrap(), self.agent, self.agent_goal) {
                    Route::Tiles(path) => {
                        self.agent = path[1];
                    }
                    Route::Complete => {
                        let mut rng = StdRng::new().unwrap();
                        loop {
                            self.agent_goal = Coord2 {
                                x: rng.gen_range(0, 80),
                                y: rng.gen_range(0, 80),
                            };
                            let map = self.map.read().unwrap();
                            if map.get(self.agent_goal).is_some() &&
                               map.get(self.agent_goal)
                                   .and_then(Tile::as_rails)
                                   .is_none() {
                                break;
                            }
                        }
                    }
                    Route::NotRouteable => unreachable!(),
                }
            }

            self.draw(&e);
        }
    }

    fn char_command(&mut self, c: char) {
        // @TODO: Ascertain whether these codes are macOS-specific.
        // @TODO: Restructure this around returning Option from entirety.
        let cmd = match c {
            '\u{f700}' => Cmd::Up,
            '\u{f701}' => Cmd::Down,
            '\u{f702}' => Cmd::Left,
            '\u{f703}' => Cmd::Right,
            '\u{f728}' => Cmd::Delete,
            ' ' => Cmd::Pave,
            c => {
                match Building::from_code(c) {
                    Some(building) => Cmd::Build(building),
                    None => {
                        println!("Unhandled char input: {:?}", c);
                        return;
                    }
                }
            }
        };

        let mut map = self.map.write().unwrap();
        match cmd {
            Cmd::Up => {
                map.cursor.y = map.cursor.y.saturating_sub(1);
            }
            Cmd::Down => {
                map.cursor.y += 1;
            }
            Cmd::Left => {
                map.cursor.x = map.cursor.x.saturating_sub(1);
            }
            Cmd::Right => {
                map.cursor.x += 1;
            }
            Cmd::Delete => {
                let pos = map.cursor;
                map.delete(pos);
            }
            Cmd::Pave => {
                let pos = map.cursor;
                if map.can_pave(pos) {
                    map.pave(pos);
                }
            }
            Cmd::Build(building) => {
                let pos = map.cursor;
                if map.can_build(pos) {
                    map.build(pos, building);
                }
            }
        }
    }

    fn draw(&mut self, e: &Event) {
        let map = self.map.read().unwrap();
        let agent_pos = self.agent;
        self.window
            .draw_2d(e, |c, g| for y in 0..map.height() {
                clear([0.95; 4], g);
                for x in 0..map.width() {
                    let l = Coord2 { x, y };
                    if let Some(tile) = map.get(l) {
                        Self::draw_tile(c, g, map.clone(), l, tile);
                    }
                }
                Self::draw_cursor(c, g, map.cursor);
                // @TODO: Remove.
                Self::draw_cursor(c, g, agent_pos);
            });
    }

    fn draw_cursor(c: Context, g: &mut G2d, cursor: Coord2) {
        let x = cursor.x * PPU;
        let y = cursor.y * PPU;
        rectangle([0.0, 0.0, 0.0, 0.3],
                  [x as f64, y as f64, PPU as f64, PPU as f64],
                  c.transform,
                  g);
    }

    fn draw_tile(c: Context, g: &mut G2d, _: Map, l: Coord2, tile: &Tile) {
        match *tile {
            Tile::Building(BuildingTile { ref building, .. }) => {
                Self::draw_building(c, g, l, building)
            }
            Tile::Entrance(EntranceTile { orientation, .. }) => {
                Self::draw_entrance(c, g, l, orientation)
            }
            Tile::Paving(PavingTile { .. }) => Self::draw_paving(c, g, l),
            Tile::Rails(RailsTile { orientation, .. }) => Self::draw_rails(c, g, l, orientation),
        }
    }

    fn draw_building(c: Context, g: &mut G2d, l: Coord2, building: &Building) {
        (match *building {
             Building::House => Self::draw_building_house,
             Building::Saloon => Self::draw_building_saloon,
             Building::Factory => Self::draw_building_factory,
             Building::GeneralStore => Self::draw_building_general_store,
             Building::TrainStation => Self::draw_other,
         })(c, g, l)
    }

    fn draw_building_house(c: Context, g: &mut G2d, l: Coord2) {
        let x = l.x * PPU;
        let y = l.y * PPU;
        rectangle([42.0 / 255.0, 201.0 / 255.0, 111.0 / 255.0, 1.0],
                  [x as f64, y as f64, PPU as f64, PPU as f64],
                  c.transform,
                  g);
        rectangle([1.0, 1.0, 1.0, 1.0],
                  [(x + 3) as f64, (y + 3) as f64, (PPU - 6) as f64, (PPU - 6) as f64],
                  c.transform,
                  g);
    }

    fn draw_building_saloon(c: Context, g: &mut G2d, l: Coord2) {
        let x = l.x * PPU;
        let y = l.y * PPU;
        rectangle([0.0 / 255.0, 162.0 / 255.0, 255.0 / 255.0, 1.0],
                  [x as f64, y as f64, PPU as f64, PPU as f64],
                  c.transform,
                  g);
    }

    fn draw_building_factory(c: Context, g: &mut G2d, l: Coord2) {
        let x = l.x * PPU;
        let y = l.y * PPU;
        rectangle([245.0 / 255.0, 51.0 / 255.0, 31.0 / 255.0, 1.0],
                  [x as f64, y as f64, PPU as f64, PPU as f64],
                  c.transform,
                  g);
    }

    fn draw_building_general_store(c: Context, g: &mut G2d, l: Coord2) {
        let x = l.x * PPU;
        let y = l.y * PPU;
        rectangle([159.0 / 255.0, 0.0 / 255.0, 224.0 / 255.0, 1.0],
                  [x as f64, y as f64, PPU as f64, PPU as f64],
                  c.transform,
                  g);
    }

    fn draw_entrance(c: Context, g: &mut G2d, l: Coord2, orientation: Orientation) {
        let x = l.x * PPU;
        let y = l.y * PPU;
        match orientation {
            Orientation::Vertical => {
                rectangle([0.6, 0.6, 0.6, 1.0],
                          [(x + PPU / 2 - 1) as f64, y as f64, 2.0, PPU as f64],
                          c.transform,
                          g);
            }
            Orientation::Horizontal => {
                rectangle([0.6, 0.6, 0.6, 1.0],
                          [x as f64, (y + PPU / 2 - 1) as f64, PPU as f64, 2.0],
                          c.transform,
                          g);
            }
        }
    }

    fn draw_paving(c: Context, g: &mut G2d, l: Coord2) {
        let x = l.x * PPU;
        let y = l.y * PPU;
        rectangle([0.6, 0.6, 0.6, 1.0],
                  [x as f64, y as f64, PPU as f64, PPU as f64],
                  c.transform,
                  g);
    }

    fn draw_rails(c: Context, g: &mut G2d, l: Coord2, orientation: Orientation) {
        let x = l.x * PPU;
        let y = l.y * PPU;
        match orientation {
            Orientation::Vertical => {
                rectangle([0.0, 0.0, 0.0, 1.0],
                          [(x + 2) as f64, y as f64, 2.0, PPU as f64],
                          c.transform,
                          g);
                rectangle([0.0, 0.0, 0.0, 1.0],
                          [(x + PPU - 4) as f64, y as f64, 2.0, PPU as f64],
                          c.transform,
                          g);
            }
            Orientation::Horizontal => {
                for x2 in 0..(PPU - 1) {
                    if x2 % 2 == 0 {
                        rectangle([0.0, 0.0, 0.0, 0.7],
                                  [(x + x2) as f64, y as f64, 1.0, PPU as f64],
                                  c.transform,
                                  g);
                    }
                }
                rectangle([0.0, 0.0, 0.0, 0.7],
                          [x as f64, (y + 2) as f64, PPU as f64, 2.0],
                          c.transform,
                          g);
                rectangle([0.0, 0.0, 0.0, 0.7],
                          [x as f64, (y + PPU - 4) as f64, PPU as f64, 2.0],
                          c.transform,
                          g);
            }
        }
    }

    fn draw_other(c: Context, g: &mut G2d, l: Coord2) {
        let x = l.x * PPU;
        let y = l.y * PPU;
        rectangle([0.2, 0.4, 0.6, 1.0],
                  [x as f64, y as f64, PPU as f64, PPU as f64],
                  c.transform,
                  g);
    }
}
