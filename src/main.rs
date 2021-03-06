extern crate clap;
extern crate piston_window;
extern crate tilewater;

//use std::thread;
//use std::time::Duration;
use std::sync::{mpsc, Arc, RwLock};
//use clap::{Arg, App};
use piston_window::*;
use tilewater::*;

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Tilewater", [800, 800])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let ups = 40;
    window.set_ups(ups);
    window.set_max_fps(60);
    let mut agents = Agents::new(ups / 5);

    let mut map = Map::new(Coord2 { x: 80, y: 80 });

    for y in 1..31 {
        let p = Coord2 { x: 40, y: y };
        if map.can_pave(p) {
            map.pave(p);
        }
    }
    for x in 40..49 {
        let p = Coord2 { x: x, y: 3 };
        if map.can_pave(p) {
            map.pave(p);
        }
    }
    for y in 3..16 {
        let p = Coord2 { x: 48, y: y };
        if map.can_pave(p) {
            map.pave(p);
        }
    }
    for x in 48..56 {
        let p = Coord2 { x: x, y: 6 };
        if map.can_pave(p) {
            map.pave(p);
        }
    }
    for y in 6..21 {
        let p = Coord2 { x: 55, y: y };
        if map.can_pave(p) {
            map.pave(p);
        }
    }
    for x in 55..69 {
        let p = Coord2 { x: x, y: 9 };
        if map.can_pave(p) {
            map.pave(p);
        }
    }
    for x in 40..65 {
        let p = Coord2 { x: x, y: 20 };
        if map.can_pave(p) {
            map.pave(p);
        }
    }

    for x in 0..80 {
        let c = Coord2 { x: x, y: 0 };
        //if map.can_rail(c) {
        map.rail(c);
        //}
    }

    let bs = vec![
        (42, 4, 'f'),
        (42, 6, 'f'),
        (38, 9, 'f'),
        (42, 9, 's'),
        (42, 11, 'f'),
        (42, 13, 'f'),
        (42, 15, 'f'),
        (42, 17, 'h'),
        (38, 17, 'g'),
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
        (57, 6, 's'),
        (59, 6, 'g'),
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
        (53, 17, 'h'),
    ];

    let (passenger_tx, passenger_rx) = mpsc::channel();
    for b in bs {
        let c = Coord2 { x: b.0, y: b.1 + 1 };
        if map.can_build(c) {
            let building = Building::from_code(b.2).unwrap();
            map.build(c, building);
            if building == Building::House {
                let decider = ResidentDecider::new(c);
                let agent = Agent::new(Coord2 { x: 40, y: 2 }, Box::new(decider));
                //agents.insert(AgentKind::Resident, agent);
                passenger_tx.send(agent).unwrap();
            }
        } else {
            println!("{:?}", c);
            panic!("Unbuildable seed building.");
        }
    }

    let train_decider = TrainDecider::new(Coord2 { x: 42, y: 0 }, passenger_rx);
    let train_agent = Agent::new(Coord2 { x: 0, y: 0 }, Box::new(train_decider));
    agents.insert(AgentKind::Train, train_agent);
    //println!("{}", map);

    let map = Arc::new(RwLock::new(map));
    RenderToPiston::new(agents, window, map, passenger_tx).render_loop();

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
