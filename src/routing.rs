use std::collections::*;
use std::cmp::{min, max, Ordering};
use super::*;

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
    if map.get(start_pos).is_none() || map.get(goal_pos).is_none() {
        return Route::NotRouteable;
    }

    let mut closed = HashSet::new();
    let mut open = HashSet::new();
    open.insert(start_pos);
    let mut came_from = HashMap::new();

    let mut g_score = HashMap::new();
    g_score.insert(start_pos, 0);
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
        let current = match map.get(current_pos) {
            Some(current) => current,
            None => continue,
        };
        for neighbour_pos in tile_neighbours(current) {
            if closed.contains(&neighbour_pos) {
                continue;
            }
            let tentative_g_score = g_score[&current_pos] + 1;
            if !open.contains(&neighbour_pos) {
                open.insert(neighbour_pos);
            } else if tentative_g_score >= g_score[&neighbour_pos] {
                continue;
            }
            came_from.insert(neighbour_pos, current_pos);
            g_score.insert(neighbour_pos, tentative_g_score);
            f_score.push(FScoreItem {
                             f_score: -(tentative_g_score +
                                        heuristic_cost_estimate(neighbour_pos, goal_pos)),
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
pub fn route_to_any(map: &Map, start_pos: Coord2, goals_pos: Vec<Coord2>) -> Route {
    let mut routes = vec![];

    for goal_pos in goals_pos {
        match route(map, start_pos, goal_pos) {
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct FScoreItem<T>
    where T: PartialEq + Eq
{
    f_score: i64,
    item: T,
}

impl<T> PartialOrd for FScoreItem<T>
    where T: PartialEq + Eq
{
    fn partial_cmp(&self, other: &FScoreItem<T>) -> Option<Ordering> {
        Some(self.f_score.cmp(&other.f_score))
    }
}

impl<T> Ord for FScoreItem<T>
    where T: PartialEq + Eq
{
    fn cmp(&self, other: &FScoreItem<T>) -> Ordering {
        self.f_score.cmp(&other.f_score)
    }
}

fn tile_neighbours(tile: &Tile) -> Vec<Coord2> {
    match *tile {
        Tile::Building(BuildingTile { ref entryway_pos, .. }) => vec![*entryway_pos],
        Tile::Entrance(EntranceTile {
                           ref road_pos,
                           ref building_pos,
                           ..
                       }) => vec![*road_pos, *building_pos],
        Tile::Paving(PavingTile {
                         ref entryways_pos,
                         ref pavings_pos,
                     }) => {
            // This is inherently opinionated. We prioritise local entryways
            // over other pavings.
            entryways_pos
                .iter()
                .cloned()
                .chain(pavings_pos.iter().cloned())
                .collect()
        }
        // We don't route things along railways. Yet.
        Tile::Rails(RailsTile { .. }) => vec![],
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

fn heuristic_cost_estimate(current_pos: Coord2, goal_pos: Coord2) -> i64 {
    let x_diff = max(goal_pos.x, current_pos.x) - min(goal_pos.x, current_pos.x);
    let y_diff = max(goal_pos.y, current_pos.y) - min(goal_pos.y, current_pos.y);
    (x_diff + y_diff) as i64
}
