#![feature(inclusive_range_syntax)]

extern crate clap;
extern crate rand;
extern crate tilewater;
extern crate piston_window;

use std::io::prelude::*;
use std::thread;
use std::cmp::{min, max};
use std::time::Duration;
use std::sync::{Arc, RwLock};

//use clap::{Arg, App};
use rand::{Rng, OsRng};
use piston_window::*;
use tilewater::*;

fn main() {
    let mut rng: Box<Rng> = Box::new(OsRng::new().expect("Could not start the PRNG."));

    let mut window: PistonWindow = WindowSettings::new("Hello Piston!", [640, 480])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut tile_map = TileMap::new(Coord2 { x: 80, y: 30 });

    for y in 0...29 {
        let p = Coord2 { x: 40, y: y };
        if tile_map.can_pave(p) {
            tile_map.pave(p);
        }
    }
    for x in 40...48 {
        let p = Coord2 { x: x, y: 2 };
        if tile_map.can_pave(p) {
            tile_map.pave(p);
        }
    }
    for y in 2...14 {
        let p = Coord2 { x: 48, y: y };
        if tile_map.can_pave(p) {
            tile_map.pave(p);
        }
    }
    for x in 48...55 {
        let p = Coord2 { x: x, y: 5 };
        if tile_map.can_pave(p) {
            tile_map.pave(p);
        }
    }
    for y in 5...19 {
        let p = Coord2 { x: 55, y: y };
        if tile_map.can_pave(p) {
            tile_map.pave(p);
        }
    }
    for x in 55...84 {
        let p = Coord2 { x: x, y: 8 };
        if tile_map.can_pave(p) {
            tile_map.pave(p);
        }
    }
    for x in 40...64 {
        let p = Coord2 { x: x, y: 19 };
        if tile_map.can_pave(p) {
            tile_map.pave(p);
        }
    }

    let bs = vec![(42, 4, 'f'),
                  (42, 6, 'f'),
                  (38, 9, 'f'),
                  (42, 9, 't'),
                  (42, 11, 'f'),
                  (42, 13, 'f'),
                  (42, 15, 'f'),
                  (42, 17, 'h'),
                  (42, 19, 'h'),
                  (38, 17, 's'),
                  (44, 4, 'h'),
                  (46, 4, 'h'),
                  (46, 7, 'h'),
                  (46, 9, 'h'),
                  (46, 11, 'h'),
                  (46, 13, 'h'),
                  (48, 16, 'h'),
                  (50, 8, 'h'),
                  (50, 10, 'h'),
                  (50, 12, 'h'),
                  (50, 14, 'h'),
                  (50, 3, 'h'),
                  (52, 3, 'h'),
                  (54, 3, 'h'),
                  (57, 6, 't'),
                  (59, 6, 's'),
                  (57, 10, 'h'),
                  (59, 10, 'h'),
                  (57, 12, 'h'),
                  (57, 14, 'h'),
                  (57, 17, 'h'),
                  (59, 17, 'h'),
                  (44, 17, 'h'),
                  (46, 17, 'h'),
                  (49, 17, 'h'),
                  (51, 17, 'h'),
                  (53, 17, 'h')];
    for b in bs {
        let c = Coord2 { x: b.0, y: b.1 };
        if tile_map.can_build(c) {
            tile_map.build(c, Building::new(b.2));
        }
    }
    println!("{}", tile_map);

    let mut tile_map = Arc::new(RwLock::new(tile_map));

    let mut renderer = RenderToPiston::new(window, tile_map);
    renderer.render_loop();

    // for i in 0..20 {
    //     let p = Coord2 { x: 40, y: i + 1 };
    //     if map.can_pave(p) {
    //         map.pave(p);
    //     }

    //     if i >= 2 {
    //         let b = Coord2 { x: 42, y: i - 2 };
    //         if map.can_build(b) {
    //             map.build(b, Building::new('h'));
    //         }
    //     }

    //     println!("{}", map);
    //     std::io::stdout().flush().unwrap();
    //     thread::sleep(Duration::from_millis(100));
    //     println!("\n\n\n\n");
    // }
}
