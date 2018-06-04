use super::*;
use std::cmp::{max, min, Ordering};
use std::collections::*;

const COST_OF_AN_EMPTY_TILE: f64 = 30.0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    NotRouteable,
    Complete,
    Tiles(Vec<Coord2>),
}

/// Implementation of A* search on `Map`.
///
/// Derived from https://en.wikipedia.org/wiki/A*_search_algorithm#Pseudocode
pub fn route(map: &Map, start_pos: Coord2, goal_pos: Coord2) -> Route {
    if start_pos == goal_pos {
        return Route::Complete;
    }
    if map.get(goal_pos).is_none() {
        return Route::NotRouteable;
    }

    let mut closed = HashSet::new();
    let mut open = HashSet::new();
    open.insert(start_pos);
    let mut came_from = HashMap::new();

    let mut g_score = HashMap::new();
    g_score.insert(start_pos, 0.0);
    // Max heap thus use negative of costs.
    let mut f_score = BinaryHeap::new();
    f_score.push(FScoreItem {
        f_score: -heuristic_cost_estimate(start_pos, goal_pos),
        item: start_pos,
    });

    while !open.is_empty() {
        let current_pos = match f_score.pop() {
            Some(FScoreItem { item, .. }) => item,
            None => unreachable!(),
        };
        if current_pos == goal_pos {
            return Route::Tiles(reconstruct_path(came_from, current_pos));
        }

        open.remove(&current_pos);
        closed.insert(current_pos);
        let current = map.get(current_pos);
        for (neighbour_pos, neighbour_move_cost) in tile_neighbours(current, current_pos, map) {
            if closed.contains(&neighbour_pos) {
                continue;
            }
            let tentative_g_score = g_score[&current_pos] + neighbour_move_cost;
            if !open.contains(&neighbour_pos) {
                open.insert(neighbour_pos);
            } else if tentative_g_score >= g_score[&neighbour_pos] {
                continue;
            }
            came_from.insert(neighbour_pos, current_pos);
            g_score.insert(neighbour_pos, tentative_g_score);
            f_score.push(FScoreItem {
                f_score: -(tentative_g_score + heuristic_cost_estimate(neighbour_pos, goal_pos)),
                item: neighbour_pos,
            });
        }
    }

    Route::NotRouteable
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteDirection {
    NotRouteable,
    Complete,
    Direction(Direction),
}

pub fn direction_of_route(map: &Map, start_pos: Coord2, goal_pos: Coord2) -> RouteDirection {
    match route(map, start_pos, goal_pos) {
        Route::NotRouteable => RouteDirection::NotRouteable,
        Route::Complete => RouteDirection::Complete,
        Route::Tiles(route) => {
            // Find direction from route[0] to route[1].
            match Direction::between_coord2s(route[0], route[1]) {
                Some(direction) => RouteDirection::Direction(direction),
                None => RouteDirection::NotRouteable,
            }
        }
    }
}

/// Find route to closest of several possible destinations.
///
/// This could be optimised with an adaptation of A* search.
pub fn route_to_any(map: &Map, start_pos: Coord2, goals_pos: &Vec<Coord2>) -> Route {
    let mut routes = vec![];

    for goal_pos in goals_pos {
        match route(map, start_pos, *goal_pos) {
            Route::NotRouteable => {}
            Route::Complete => return Route::Complete,
            Route::Tiles(route) => {
                routes.push(route);
            }
        }
    }

    match routes.into_iter().min_by(|x, y| x.len().cmp(&y.len())) {
        Some(route) => Route::Tiles(route),
        None => Route::NotRouteable,
    }
}

pub fn route_to_building(map: &Map, start_pos: Coord2, building: Building) -> Route {
    if let Some(buildings_pos) = map.buildings.get(&building) {
        route_to_any(map, start_pos, buildings_pos)
    } else {
        Route::NotRouteable
    }
}

pub fn closest_building(map: &Map, start_pos: Coord2, building: Building) -> Option<Coord2> {
    match route_to_building(map, start_pos, building) {
        Route::Tiles(route) => route.last().cloned(),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq)]
struct FScoreItem<T>
where
    T: PartialEq + Eq,
{
    f_score: f64,
    item: T,
}

impl<T> Eq for FScoreItem<T>
where
    T: PartialEq + Eq,
{
}

impl<T> PartialOrd for FScoreItem<T>
where
    T: PartialEq + Eq,
{
    fn partial_cmp(&self, other: &FScoreItem<T>) -> Option<Ordering> {
        self.f_score.partial_cmp(&other.f_score)
    }
}

impl<T> Ord for FScoreItem<T>
where
    T: PartialEq + Eq,
{
    fn cmp(&self, other: &FScoreItem<T>) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}

fn tile_neighbours(tile: Option<&Tile>, location: Coord2, map: &Map) -> Vec<(Coord2, f64)> {
    match tile.clone() {
        Some(Tile::Building(BuildingTile {
            ref entryway_pos, ..
        })) => vec![(*entryway_pos, 0.1)],
        Some(Tile::Entrance(EntranceTile {
            ref road_pos,
            ref building_pos,
            ..
        })) => vec![(*road_pos, 0.1), (*building_pos, 0.1)],
        Some(Tile::Paving(PavingTile {
            ref entryways_pos,
            ref pavings_pos,
        })) => {
            // This is inherently opinionated. We prioritise local entryways
            // over other pavings.
            let mut neighbours_with_costs: Vec<(Coord2, f64)> = entryways_pos
                .iter()
                .cloned()
                .map(|e| (e, 0.1))
                .chain(pavings_pos.iter().map(|p| (*p, 1.0)))
                .collect();
            // To maintain entryway-before-paving priority order, we add empty
            // tiles in a second pass.
            let mut neighbours: HashSet<Coord2> = neighbours_with_costs
                .clone()
                .into_iter()
                .map(|(location, _)| location)
                .collect();
            for neighbour in location.neighbours() {
                if !neighbours.contains(&neighbour) && map.can_pave(neighbour) {
                    neighbours.insert(neighbour);
                    neighbours_with_costs.push((neighbour, COST_OF_AN_EMPTY_TILE));
                }
            }
            neighbours_with_costs
        }
        // We don't route things along railways. Yet.
        Some(Tile::Rails(RailsTile { .. })) => vec![],
        None => location
            .neighbours()
            .into_iter()
            .filter(|neighbour| map.can_walk(*neighbour))
            .map(|neighbour| (neighbour, COST_OF_AN_EMPTY_TILE))
            .collect(),
    }
}

fn reconstruct_path(came_from: HashMap<Coord2, Coord2>, mut current_pos: Coord2) -> Vec<Coord2> {
    let mut path = vec![current_pos];
    while came_from.contains_key(&current_pos) {
        current_pos = came_from[&current_pos];
        path.push(current_pos);
    }
    path.reverse();
    path
}

fn heuristic_cost_estimate(current_pos: Coord2, goal_pos: Coord2) -> f64 {
    let x_diff = max(goal_pos.x, current_pos.x) - min(goal_pos.x, current_pos.x);
    let y_diff = max(goal_pos.y, current_pos.y) - min(goal_pos.y, current_pos.y);
    (x_diff + y_diff) as f64
}
