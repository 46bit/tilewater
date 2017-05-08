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
        let mut dead_agent_ids = vec![];
        for agent in self.agents.values_mut() {
            match agent.state.action {
                AgentAction::Dead => {
                    dead_agent_ids.push(agent.state.id);
                }
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
        for dead_agent_id in dead_agent_ids {
            self.agents.remove(&dead_agent_id);
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
    /// Agent is dead. Will be removed immediately.
    Dead,
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
        let home = self.home;
        match self.state {
            ResidentState::MovingIn | ResidentState::GoingHome => {
                self.going_to(agent, map, home, (ResidentState::AtHome, AgentAction::Idle))
            }
            ResidentState::AtHome => {
                // @TODO: These checks are not enough if some were created but then all were
                // deleted. Instead, use `closest` giving `None` as the fallback.
                let go_drinking = rng.gen_weighted_bool(60);
                let go_shopping = rng.gen_weighted_bool(40);
                if go_shopping == go_drinking {
                    return AgentAction::Idle;
                }

                let (building_type, state_constructor, done_state): (Building, fn(Coord2) -> ResidentState, ResidentState) = if go_drinking {
                    (Building::Saloon, ResidentState::GoingToDrink, ResidentState::Drinking)
                } else {
                    (Building::GeneralStore, ResidentState::GoingToShop, ResidentState::Shopping)
                };
                let closest = match closest_building(map, agent.position, building_type) {
                    Some(closest) => closest,
                    None => return AgentAction::Dead,
                };
                self.state = state_constructor(closest);
                self.going_to(agent, map, closest, (done_state, AgentAction::Idle))
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
                if rng.gen_weighted_bool(40) {
                    self.go_home(agent, map)
                } else {
                    AgentAction::Idle
                }
            }
            _ => unimplemented!(),
        }
    }
}
