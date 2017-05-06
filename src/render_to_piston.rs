use std::sync::{Arc, RwLock, RwLockReadGuard};
use piston_window::*;
use piston_window::types::Color;
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
                clear([1.0; 4], g);
                for x in 0..tile_map.width() {
                    let l = Coord2 { x, y };
                    if let Some(tile) = tile_map.get(l) {
                        Self::draw_tile(c, g, tile_map.clone(), l, tile);
                    }
                }
            });
    }

    fn draw_tile(c: Context, g: &mut G2d, tile_map: TileMap, l: Coord2, tile: &Tile) {
        let x = l.x * PPU;
        let y = l.y * PPU;
        rectangle(Self::tile_color(tile),
                  [x as f64, y as f64, PPU as f64, PPU as f64],
                  c.transform,
                  g);
    }

    fn tile_color(tile: &Tile) -> Color {
        //[1.0, 0.0, 0.0, 1.0]
        match *tile {
            Tile::Building { building, .. } => [0.0, 1.0, 0.0, 1.0],
            Tile::Entrance { .. } => [0.0, 0.0, 0.0, 0.5],
            Tile::Paving { .. } => [0.0, 0.0, 0.0, 1.0],
            Tile::Rails { .. } => [1.0, 0.0, 0.0, 1.0],
        }
    }
}


// fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//     for y in 0..self.dimensions.y {
//         for x in 0..self.dimensions.x {
//             if let Some(tile) = self.get(Coord2 { x, y }) {
//                 write!(f, "{}", tile)?;
//             } else {
//                 write!(f, " ")?;
//             }
//         }
//         write!(f, "\n")?;
//     }
//     Ok(())
// }
