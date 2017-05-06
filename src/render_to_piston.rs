use std::sync::{Arc, RwLock};
use piston_window::*;
use super::*;

const PPU: u64 = 10;

pub struct RenderToPiston {
    window: PistonWindow,
    tile_map: Arc<RwLock<TileMap>>,
}

impl RenderToPiston {
    pub fn new(window: PistonWindow, tile_map: Arc<RwLock<TileMap>>) -> RenderToPiston {
        RenderToPiston { window, tile_map }
    }

    pub fn render_loop(&mut self) {
        while let Some(e) = self.window.next() {
            self.draw(&e);
        }
    }

    fn draw(&mut self, e: &Event) {

        //     clear([1.0; 4], g);
        //     rectangle([1.0, 0.0, 0.0, 1.0], // red
        //               [0.0, 0.0, 100.0, 100.0],
        //               c.transform,
        //               g);
        // });

        let tile_map = self.tile_map.read().unwrap();
        self.window
            .draw_2d(e, |c, g| for y in 0..tile_map.height() {
                clear([0.9; 4], g);
                for x in 0..tile_map.width() {
                    let l = Coord2 { x, y };
                    if let Some(tile) = tile_map.get(l) {
                        Self::draw_tile(c, g, tile_map.clone(), l, tile);
                    }
                }
            });
    }

    fn draw_tile(c: Context, g: &mut G2d, _: TileMap, l: Coord2, tile: &Tile) {
        match *tile {
            Tile::Building { ref building, .. } => Self::draw_building(c, g, l, building),
            Tile::Entrance { orientation, .. } => Self::draw_entrance(c, g, l, orientation),
            Tile::Paving { .. } => Self::draw_paving(c, g, l),
            Tile::Rails { orientation, .. } => Self::draw_rails(c, g, l, orientation),
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
                rectangle([0.0, 0.0, 0.0, 1.0],
                          [(x + PPU / 2 - 1) as f64, y as f64, 2.0, PPU as f64],
                          c.transform,
                          g);
            }
            Orientation::Horizontal => {
                rectangle([0.0, 0.0, 0.0, 1.0],
                          [x as f64, (y + PPU / 2 - 1) as f64, PPU as f64, 2.0],
                          c.transform,
                          g);
            }
        }
    }

    fn draw_paving(c: Context, g: &mut G2d, l: Coord2) {
        let x = l.x * PPU;
        let y = l.y * PPU;
        rectangle([0.0, 0.0, 0.0, 1.0],
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
                for x2 in 0..PPU {
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
