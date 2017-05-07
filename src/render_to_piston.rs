use std::sync::{Arc, RwLock};
use piston_window::*;
use piston_window::types::Color;
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
    agents: Agents,
    window: PistonWindow,
    map: Arc<RwLock<Map>>,
}

impl RenderToPiston {
    pub fn new(agents: Agents, window: PistonWindow, map: Arc<RwLock<Map>>) -> RenderToPiston {
        RenderToPiston {
            agents,
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

            if let Event::Update(_) = e {
                let map = self.map.read().unwrap();
                self.agents.update(&map);
            }

            if let Event::Render(_) = e {
                self.draw(&e);
            }
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
        let render_info = self.agents.render_info();
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

                for agent_render_info in &render_info {
                    let &((x, y), co) = agent_render_info;
                    Self::draw_agent(c, g, (x, y), co);
                }
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

    fn draw_agent(c: Context, g: &mut G2d, pos: (f64, f64), co: Color) {
        let ppuf = PPU as f64;
        let x = pos.0 * ppuf + (ppuf / 2.0) - 2.0;
        let y = pos.1 * ppuf + (ppuf / 2.0) - 2.0;
        ellipse(co, [x, y, 4.0, 4.0], c.transform, g);
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
