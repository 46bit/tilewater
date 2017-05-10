use std::collections::*;
use std::fmt::Debug;
use std::mem;
use uuid::Uuid;
use rand::*;
use rayon::prelude::*;
use super::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AgentKind {
    Resident,
    Train,
}

pub struct Agents {
    ticks_per_unit: u64,
    ticks_this_unit: u64,
    agents: HashMap<AgentKind, HashMap<Uuid, Agent>>,
}

impl Agents {
    pub fn new(ticks_per_unit: u64) -> Agents {
        Agents {
            ticks_per_unit: ticks_per_unit,
            ticks_this_unit: 0,
            agents: HashMap::new(),
        }
    }

    pub fn insert(&mut self, kind: AgentKind, agent: Agent) {
        self.agents
            .entry(kind)
            .or_insert_with(|| HashMap::new())
            .insert(agent.state.id, agent);
    }

    pub fn decide(&mut self, map: &Map) {
        let mut insert_list = vec![];
        for agents_of_kind in self.agents.values_mut() {
            // @TODO: Reintroduce `par_iter_mut` using `flat_map`.
            for agent in agents_of_kind.values_mut() {
                let mut rng: Box<Rng> = Box::new(thread_rng());
                let agent_state_clone = agent.state.clone();
                agent.action = agent
                    .decider
                    .decide_action(&agent_state_clone, map, &mut rng);
                loop {
                    let mut current_action = AgentAction::Idle;
                    mem::swap(&mut agent.action, &mut current_action);
                    match current_action {
                        AgentAction::Yield(agents, then) => {
                            for agent in agents {
                                insert_list.push(agent);
                            }
                            agent.action = *then;
                        }
                        AgentAction::Jump(pos, then) => {
                            agent.state.position = pos;
                            agent.action = *then;
                        }
                        o => {
                            agent.action = o;
                            break;
                        }
                    }
                }
            }
        }
        for insert in insert_list {
            self.insert(AgentKind::Resident, insert);
        }
    }

    pub fn update(&mut self, map: &Map) {
        let ticks_per_unit = self.ticks_per_unit;
        self.ticks_this_unit += 1;
        let ticks_this_unit = self.ticks_this_unit;
        for agents_of_kind in self.agents.values_mut() {
            let dead_agent_ids: Vec<_> = agents_of_kind
                .par_iter_mut()
                .filter_map(|(_, agent)| match agent.action {
                                AgentAction::Dead => Some(agent.state.id),
                                AgentAction::Idle => None,
                                AgentAction::Move(direction) => {
                    let direction_offset = direction.as_offset();
                    agent.subunit_position.0 += (direction_offset.0 as f64) /
                                                (ticks_per_unit as f64);
                    agent.subunit_position.1 += (direction_offset.1 as f64) /
                                                (ticks_per_unit as f64);
                    if ticks_this_unit == ticks_per_unit {
                        agent.state.position.x =
                            ((agent.state.position.x as i64) + direction_offset.0) as u64;
                        agent.state.position.y =
                            ((agent.state.position.y as i64) + direction_offset.1) as u64;
                        agent.subunit_position.0 = agent.state.position.x as f64;
                        agent.subunit_position.1 = agent.state.position.y as f64;
                    }
                    None
                }
                                AgentAction::Yield(_, _) |
                                AgentAction::Jump(_, _) => unreachable!(),
                            })
                .collect();
            for dead_agent_id in dead_agent_ids {
                agents_of_kind.remove(&dead_agent_id);
            }
        }
        if self.ticks_this_unit == self.ticks_per_unit {
            self.ticks_this_unit = 0;
            self.decide(map);
        }
    }

    pub fn agent_subunit_positions(&mut self) -> HashMap<AgentKind, Vec<(f64, f64)>> {
        self.agents
            .iter()
            .map(|(k, s)| (*k, s.values().map(|a| a.subunit_position).collect()))
            .collect()
    }
}

#[derive(Debug)]
pub enum AgentAction {
    /// Agent is dead. Will be removed immediately.
    Dead,
    /// Make no large-scale movement this turn.
    Idle,
    /// Move 1 square in this direction before the next action is decided.
    Move(Direction),
    /// Yield an agent to add at the current position.
    Yield(Vec<Agent>, Box<AgentAction>),
    /// Move to a specific tile.
    Jump(Coord2, Box<AgentAction>),
}

#[derive(Debug)]
pub struct Agent {
    state: AgentState,
    action: AgentAction,
    subunit_position: (f64, f64),
    decider: Box<Decider + Send + Sync>,
}

impl Agent {
    pub fn new(position: Coord2, decider: Box<Decider + Send + Sync>) -> Agent {
        Agent {
            state: AgentState {
                id: Uuid::new_v4(),
                position: position,
            },
            // @TODO: Decide new action on instantiation or not?
            action: AgentAction::Idle,
            subunit_position: (position.x as f64, position.y as f64),
            decider: decider,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentState {
    id: Uuid,
    position: Coord2,
}

pub trait Decider: Debug {
    fn decide_action(&mut self, agent: &AgentState, map: &Map, rng: &mut Box<Rng>) -> AgentAction;

    // @TODO: Figure out how to implement respawning. This would require an agent to know
    // where it should respawn, which imposes some tough information requirements.
    //fn respawn(&mut self, agent: &AgentState, map: &Map, rng: &mut Box<Rng>) -> bool;
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

    fn going_to(&mut self,
                agent: &AgentState,
                map: &Map,
                to: Coord2,
                and_then: (ResidentState, AgentAction))
                -> AgentAction {
        match direction_of_route(map, agent.position, to) {
            RouteDirection::NotRouteable => AgentAction::Dead,
            RouteDirection::Complete => {
                self.state = and_then.0;
                and_then.1
            }
            RouteDirection::Direction(direction) => AgentAction::Move(direction),
        }
    }

    fn go_home(&mut self, agent: &AgentState, map: &Map) -> AgentAction {
        let home = self.home;
        self.state = ResidentState::GoingHome;
        self.going_to(agent, map, home, (ResidentState::AtHome, AgentAction::Idle))
    }
}

impl Decider for ResidentDecider {
    fn decide_action(&mut self, agent: &AgentState, map: &Map, rng: &mut Box<Rng>) -> AgentAction {
        // Ensure that idle residents are killed if the square they are on is deleted.
        if map.get(agent.position).is_none() {
            return AgentAction::Dead;
        }

        let home = self.home;
        match self.state {
            ResidentState::MovingIn | ResidentState::GoingHome => {
                self.going_to(agent, map, home, (ResidentState::AtHome, AgentAction::Idle))
            }
            ResidentState::AtHome => {
                let go_drinking = rng.gen_weighted_bool(60);
                let go_shopping = rng.gen_weighted_bool(40);
                let go_working = rng.gen_weighted_bool(80);

                if go_working && self.work.is_some() {
                    self.state = ResidentState::GoingToWork;
                    let and_then = (ResidentState::Working, AgentAction::Idle);
                    let work = self.work.unwrap();
                    self.going_to(agent, map, work, and_then)
                } else {
                    let (building_type, state_constructor, done_state): (Building, fn(Coord2) -> ResidentState, ResidentState) = if go_drinking {
                        (Building::Saloon, ResidentState::GoingToDrink, ResidentState::Drinking)
                    } else if go_shopping {
                        (Building::GeneralStore, ResidentState::GoingToShop, ResidentState::Shopping)
                    } else {
                        return AgentAction::Idle;
                    };
                    let closest = match closest_building(map, agent.position, building_type) {
                        Some(closest) => closest,
                        None => return AgentAction::Dead,
                    };
                    self.state = state_constructor(closest);
                    self.going_to(agent, map, closest, (done_state, AgentAction::Idle))
                }
            }
            ResidentState::GoingToShop(shop_pos) => {
                let and_then = (ResidentState::Shopping, AgentAction::Idle);
                self.going_to(agent, map, shop_pos, and_then)
            }
            ResidentState::Shopping => {
                if rng.gen_weighted_bool(10) {
                    self.go_home(agent, map)
                } else {
                    AgentAction::Idle
                }
            }
            ResidentState::GoingToDrink(saloon_pos) => {
                let and_then = (ResidentState::Drinking, AgentAction::Idle);
                self.going_to(agent, map, saloon_pos, and_then)
            }
            ResidentState::Drinking => {
                let chance_of_new_job = if self.work.is_none() { 1000 } else { 100 };
                if rng.gen_weighted_bool(chance_of_new_job) {
                    let random_factory = *rng.choose(&map.buildings[&Building::Factory]).unwrap();
                    self.work = Some(random_factory);
                    self.state = ResidentState::GoingToWork;
                    let and_then = (ResidentState::Working, AgentAction::Idle);
                    let work = self.work.unwrap();
                    self.going_to(agent, map, work, and_then)
                } else if rng.gen_weighted_bool(40) {
                    self.go_home(agent, map)
                } else {
                    AgentAction::Idle
                }
            }
            ResidentState::GoingToWork => {
                let work = self.work.unwrap();
                let and_then = (ResidentState::Working, AgentAction::Idle);
                self.going_to(agent, map, work, and_then)
            }
            ResidentState::Working => {
                if rng.gen_weighted_bool(2000) {
                    self.go_home(agent, map)
                } else {
                    AgentAction::Idle
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum TrainState {
    Arriving,
    AtPlatform(u64),
    Departing,
}

#[derive(Debug)]
pub struct TrainDecider {
    platform: Coord2,
    state: TrainState,
    pub passengers: Vec<Agent>,
}

impl TrainDecider {
    pub fn new(platform: Coord2) -> TrainDecider {
        TrainDecider {
            platform: platform,
            state: TrainState::Arriving,
            passengers: Vec::new(),
        }
    }
}

impl Decider for TrainDecider {
    fn decide_action(&mut self, agent: &AgentState, _: &Map, _: &mut Box<Rng>) -> AgentAction {
        match self.state {
            TrainState::Arriving => {
                let head_pos = Coord2 {
                    x: agent.position.x.saturating_sub(1),
                    y: agent.position.y,
                };
                if head_pos == self.platform {
                    self.state = TrainState::AtPlatform(18);
                    AgentAction::Yield(self.passengers.drain(..).collect(),
                                       Box::new(AgentAction::Idle))
                } else {
                    AgentAction::Move(Direction::East)
                }
            }
            TrainState::AtPlatform(remaining) => {
                if remaining > 0 {
                    self.state = TrainState::AtPlatform(remaining - 1);
                    AgentAction::Idle
                } else {
                    self.state = TrainState::Departing;
                    AgentAction::Move(Direction::East)
                }
            }
            TrainState::Departing => {
                if agent.position.x < 100 {
                    AgentAction::Move(Direction::East)
                } else {
                    self.state = TrainState::Arriving;
                    AgentAction::Jump(Coord2 { x: 0, y: 0 },
                                      Box::new(AgentAction::Move(Direction::East)))
                }
            }
        }
    }
}
