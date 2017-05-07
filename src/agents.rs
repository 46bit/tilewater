use std::collections::*;
use std::fmt::Debug;
use uuid::Uuid;
use rand::Rng;
use super::*;

pub struct Agents {
    ticks_per_unit: u64,
    ticks_this_unit: u64,
    agents: HashMap<Uuid, Agent>,
    rng: Box<Rng>,
}

impl Agents {
    pub fn new(ticks_per_unit: u64, rng: Box<Rng>) -> Agents {
        Agents {
            ticks_per_unit: ticks_per_unit,
            ticks_this_unit: 0,
            agents: HashMap::new(),
            rng: rng,
        }
    }

    pub fn insert(&mut self, agent: Agent) {
        self.agents.insert(agent.state.id, agent);
    }

    pub fn decide(&mut self, map: &Map) {
        for agent in self.agents.values_mut() {
            let agent_state_clone = agent.state.clone();
            agent.state.action = agent
                .decider
                .decide_action(&agent_state_clone, map, &mut self.rng);
        }
    }

    pub fn update(&mut self, map: &Map) {
        self.ticks_this_unit += 1;
        for agent in self.agents.values_mut() {
            match agent.state.action {
                AgentAction::Idle => {}
                AgentAction::Move(direction) => {
                    let direction_offset = direction.as_offset();
                    agent.subunit_position.0 += (direction_offset.0 as f64) /
                                                (self.ticks_per_unit as f64);
                    agent.subunit_position.1 += (direction_offset.1 as f64) /
                                                (self.ticks_per_unit as f64);
                    if self.ticks_this_unit == self.ticks_per_unit {
                        agent.state.position.x =
                            ((agent.state.position.x as i64) + direction_offset.0) as u64;
                        agent.state.position.y =
                            ((agent.state.position.y as i64) + direction_offset.1) as u64;
                        agent.subunit_position.0 = agent.state.position.x as f64;
                        agent.subunit_position.1 = agent.state.position.y as f64;
                    }
                }
            }
        }
        if self.ticks_this_unit == self.ticks_per_unit {
            self.ticks_this_unit = 0;
            self.decide(map);
        }
    }

    pub fn agent_subunit_positions(&mut self) -> Vec<(f64, f64)> {
        self.agents
            .values()
            .map(|a| a.subunit_position)
            .collect()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AgentAction {
    /// Make no large-scale movement this turn.
    Idle,
    /// Move 1 square in this direction before the next action is decided.
    Move(Direction),
}

#[derive(Debug)]
pub struct Agent {
    state: AgentState,
    subunit_position: (f64, f64),
    decider: Box<Decider>,
}

impl Agent {
    pub fn new(position: Coord2, decider: Box<Decider>) -> Agent {
        Agent {
            state: AgentState {
                id: Uuid::new_v4(),
                position: position,
                // @TODO: Decide new action on instantiation or not?
                action: AgentAction::Idle,
            },
            subunit_position: (position.x as f64, position.y as f64),
            decider: decider,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentState {
    id: Uuid,
    position: Coord2,
    action: AgentAction,
}

pub trait Decider: Debug {
    fn decide_action(&mut self, agent: &AgentState, map: &Map, rng: &mut Box<Rng>) -> AgentAction;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ResidentState {
    MovingIn,
    GoingHome,
    AtHome,
    GoingToShop(Coord2),
    Shopping,
    GoingToDrink(Coord2),
    Drinking,
    GoingToWork,
    Working,
}

#[derive(Clone, Debug)]
pub struct ResidentDecider {
    home: Coord2,
    work: Option<Coord2>,
    state: ResidentState,
}

impl ResidentDecider {
    pub fn new(home: Coord2) -> ResidentDecider {
        ResidentDecider {
            home: home,
            work: None,
            state: ResidentState::MovingIn,
        }
    }
}

impl Decider for ResidentDecider {
    fn decide_action(&mut self, agent: &AgentState, map: &Map, rng: &mut Box<Rng>) -> AgentAction {
        let (state, action) = match self.state {
            ResidentState::MovingIn | ResidentState::GoingHome => {
                if agent.position == self.home {
                    (ResidentState::AtHome, AgentAction::Idle)
                } else {
                    match direction_of_route(map, agent.position, self.home) {
                        RouteDirection::Direction(direction) => {
                            (ResidentState::MovingIn, AgentAction::Move(direction))
                        }
                        _ => panic!("No direction of route decided."),
                    }
                }
            }
            ResidentState::AtHome => {
                let go_drinking = map.buildings.contains_key(&Building::Saloon) &&
                                  rng.gen_weighted_bool(60);
                let go_shopping = map.buildings.contains_key(&Building::GeneralStore) &&
                                  rng.gen_weighted_bool(40);
                if go_shopping == go_drinking {
                    (ResidentState::AtHome, AgentAction::Idle)
                } else if go_shopping {
                    let home_tile = map.get(self.home).and_then(Tile::as_building).unwrap();
                    let start_direction =
                        Direction::between_coord2s(self.home, home_tile.entryway_pos).unwrap();
                    match route_to_any(map,
                                       self.home,
                                       map.buildings[&Building::GeneralStore].clone()) {
                        Route::Tiles(route) => {
                            let shop_pos = route.last().unwrap();
                            (ResidentState::GoingToShop(*shop_pos),
                             AgentAction::Move(start_direction))
                        }
                        _ => panic!("No closest general store decided."),
                    }
                } else {
                    let home_tile = map.get(self.home).and_then(Tile::as_building).unwrap();
                    let start_direction =
                        Direction::between_coord2s(self.home, home_tile.entryway_pos).unwrap();
                    match route_to_any(map, self.home, map.buildings[&Building::Saloon].clone()) {
                        Route::Tiles(route) => {
                            let saloon_pos = route.last().unwrap();
                            (ResidentState::GoingToDrink(*saloon_pos),
                             AgentAction::Move(start_direction))
                        }
                        _ => panic!("No closest general store decided."),
                    }
                }
            }
            ResidentState::GoingToShop(shop_pos) => {
                if agent.position == shop_pos {
                    (ResidentState::Shopping, AgentAction::Idle)
                } else {
                    match direction_of_route(map, agent.position, shop_pos) {
                        RouteDirection::Direction(direction) => {
                            (ResidentState::GoingToShop(shop_pos), AgentAction::Move(direction))
                        }
                        _ => panic!("No direction of route decided."),
                    }
                }
            }
            ResidentState::Shopping => {
                if rng.gen_weighted_bool(10) {
                    let shop_tile = map.get(agent.position)
                        .and_then(Tile::as_building)
                        .unwrap();
                    let start_direction =
                        Direction::between_coord2s(agent.position, shop_tile.entryway_pos).unwrap();
                    (ResidentState::GoingHome, AgentAction::Move(start_direction))
                } else {
                    (ResidentState::Shopping, AgentAction::Idle)
                }
            }
            ResidentState::GoingToDrink(saloon_pos) => {
                if agent.position == saloon_pos {
                    (ResidentState::Drinking, AgentAction::Idle)
                } else {
                    match direction_of_route(map, agent.position, saloon_pos) {
                        RouteDirection::Direction(direction) => {
                            (ResidentState::GoingToDrink(saloon_pos), AgentAction::Move(direction))
                        }
                        _ => panic!("No direction of route decided."),
                    }
                }
            }
            ResidentState::Drinking => {
                if rng.gen_weighted_bool(40) {
                    let saloon_tile = map.get(agent.position)
                        .and_then(Tile::as_building)
                        .unwrap();
                    let start_direction = Direction::between_coord2s(agent.position,
                                                                     saloon_tile.entryway_pos)
                            .unwrap();
                    (ResidentState::GoingHome, AgentAction::Move(start_direction))
                } else {
                    (ResidentState::Drinking, AgentAction::Idle)
                }
            }
            _ => unimplemented!(),
        };
        self.state = state;
        action
    }
}
