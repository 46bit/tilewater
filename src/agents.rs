use std::collections::*;
use std::fmt::Debug;
use uuid::Uuid;
use piston_window::types::Color;
use super::*;

#[derive(Debug)]
pub struct Agents {
    ticks_per_unit: u64,
    ticks_this_unit: u64,
    agents: HashMap<Uuid, Agent>,
}

impl Agents {
    pub fn new(ticks_per_unit: u64) -> Agents {
        Agents {
            ticks_per_unit: ticks_per_unit,
            ticks_this_unit: 0,
            agents: HashMap::new(),
        }
    }

    pub fn insert(&mut self, agent: Agent) {
        self.agents.insert(agent.state.id, agent);
    }

    pub fn decide(&mut self, map: &Map) {
        println!("decide1");
        for agent in self.agents.values_mut() {
            let agent_state_clone = agent.state.clone();
            agent.state.action = agent.decider.decide_action(&agent_state_clone, map);
        }
        println!("decide2");
    }

    pub fn update(&mut self, map: &Map) {
        self.ticks_this_unit += 1;
        for agent in self.agents.values_mut() {
            match agent.state.action {
                AgentAction::Idle => {
                    println!("idel");
                }
                AgentAction::Move(direction) => {
                    let direction_offset = direction.as_offset();
                    println!("{:?}", agent.subunit_position);
                    agent.subunit_position.0 += (direction_offset.0 as f64) /
                                                (self.ticks_per_unit as f64);
                    agent.subunit_position.1 += (direction_offset.1 as f64) /
                                                (self.ticks_per_unit as f64);
                    println!("{:?}", agent.subunit_position);
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

    pub fn render_info(&mut self) -> Vec<((f64, f64), Color)> {
        self.agents
            .values()
            .map(|a| (a.subunit_position, [0.0, 0.0, 0.0, 1.0]))
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
    fn decide_action(&mut self, agent: &AgentState, map: &Map) -> AgentAction;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ResidentState {
    MovingIn,
    AtHome,
    GoingToShop(Coord2),
    Shopping(Coord2),
    GoingToDrink(Coord2),
    Drinking(Coord2),
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
    fn decide_action(&mut self, agent: &AgentState, map: &Map) -> AgentAction {
        let (state, action) = match self.state {
            ResidentState::MovingIn => {
                println!("{:?} {:?}", agent.position, self.home);
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
            ResidentState::AtHome => (ResidentState::AtHome, AgentAction::Idle),
            _ => unimplemented!(),
        };
        self.state = state;
        action
    }
}
