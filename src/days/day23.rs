use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

use itertools::Itertools;
use nom::{
    character::complete::{line_ending, not_line_ending},
    multi::separated_list0,
    IResult,
};
use pathfinding::grid::Grid;
use petgraph::{
    algo::all_simple_paths,
    dot::{Config, Dot},
    prelude::*,
};

use crate::days::Day;

pub struct Day23;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Slope {
    Up,
    Right,
    Down,
    Left,
}

/// Create directed graph with authorized moves
fn get_graph(
    grid: &Grid,
    slopes: &HashMap<(usize, usize), Slope>,
) -> (Graph<(), ()>, BTreeMap<(usize, usize), NodeIndex>) {
    let mut graph = Graph::<(), ()>::new();
    let mut path_indices = BTreeMap::<(usize, usize), NodeIndex>::new();
    for path in grid {
        let idx = graph.add_node(());
        path_indices.insert(path, idx);
    }
    for path in grid {
        let a = path_indices.get(&path).unwrap();
        // when encountering a slope, we have only 1 move option (go down the slope)
        if let Some(slope) = slopes.get(&path) {
            let n = match slope {
                Slope::Up => (path.0, path.1 - 1),
                Slope::Right => (path.0 + 1, path.1),
                Slope::Down => (path.0, path.1 + 1),
                Slope::Left => (path.0 - 1, path.1),
            };
            if let Some(b) = path_indices.get(&n) {
                graph.add_edge(*a, *b, ());
            }
        } else {
            #[allow(clippy::cast_possible_wrap)]
            let path_isize = (path.0 as isize, path.1 as isize);
            // check all neighbors
            for n in grid.neighbours(path) {
                #[allow(clippy::cast_possible_wrap)]
                let n_isize = (n.0 as isize, n.1 as isize);
                // if the neighbor is a slope, we can only visit it if the slope is pointing away from us
                if let Some(slope) = slopes.get(&n) {
                    // check if slope of n points towards path
                    match ((n_isize.0 - path_isize.0, n_isize.1 - path_isize.1), slope) {
                        ((-1, 0), Slope::Right)
                        | ((0, -1), Slope::Down)
                        | ((1, 0), Slope::Left)
                        | ((0, 1), Slope::Up) => {
                            // those moves are unallowed
                            continue;
                        }
                        _ => {}
                    }
                }
                let b = path_indices.get(&n).unwrap();
                if !graph.contains_edge(*a, *b) {
                    graph.add_edge(*a, *b, ());
                }
            }
        }
    }
    (graph, path_indices)
}

/// Create undirected graph with long segments between intersections merged into one edge
fn get_graph2(
    grid: &Grid,
    start: &(usize, usize),
) -> (UnGraph<(), usize>, BTreeMap<(usize, usize), NodeIndex>) {
    // first, create graph with all possible moves
    let mut graph = UnGraph::<(), usize>::new_undirected();
    let mut path_indices = BTreeMap::<(usize, usize), NodeIndex>::new();
    for path in grid {
        let idx = graph.add_node(());
        path_indices.insert(path, idx);
    }
    let start_node = path_indices.get(start).unwrap();
    for path in grid {
        let a = path_indices.get(&path).unwrap();
        for n in grid.neighbours(path) {
            let b = path_indices.get(&n).unwrap();
            if !graph.contains_edge(*a, *b) {
                graph.add_edge(*a, *b, 1);
            }
        }
    }
    // modify graph so that segments between intersections are merged into one edge with a higher weight
    let mut stack = VecDeque::<NodeIndex>::new();
    let mut visited = BTreeSet::<NodeIndex>::new();
    stack.push_back(*start_node);
    while let Some(path) = stack.pop_front() {
        visited.insert(path);
        // add each unvisited neighbor to the stack
        for n in graph.neighbors(path) {
            if !visited.contains(&n) {
                stack.push_back(n);
            }
        }
        // check if we have only two neighbors = straight segment
        if let Some((n1, n2)) = graph.neighbors(path).collect_tuple() {
            // if we have two neighbors exactly, then let's create a new edge that skips over the current node, and
            // remove existing edges

            // get existing edges
            let ((e1_id, e1_weight), (e2_id, e2_weight)) = graph
                .edges(path)
                .map(|e| (e.id(), *(e.weight())))
                .collect_tuple()
                .unwrap();
            // add shortcut edge with the sum of both edge weights
            graph.add_edge(n1, n2, e1_weight + e2_weight);
            // disconnect the current node
            graph.remove_edge(e1_id);
            graph.remove_edge(e2_id);
        }
    }

    (graph, path_indices)
}

impl Day for Day23 {
    type Input = (
        Grid,
        HashMap<(usize, usize), Slope>,
        (usize, usize),
        (usize, usize),
    );

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        let (rest, rows) = separated_list0(line_ending, not_line_ending)(input)?;
        let height = rows.len();
        let mut start_x = 0;
        let mut end_x = 0;
        let mut path = vec![];
        let mut slopes = HashMap::<(usize, usize), Slope>::new();
        for (y, row) in rows.into_iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                match c {
                    '.' => {
                        path.push((x, y));
                        if y == 0 {
                            start_x = x;
                        } else if y == height - 1 {
                            end_x = x;
                        }
                    }
                    '^' => {
                        path.push((x, y));
                        slopes.insert((x, y), Slope::Up);
                    }
                    '>' => {
                        path.push((x, y));
                        slopes.insert((x, y), Slope::Right);
                    }
                    'v' => {
                        path.push((x, y));
                        slopes.insert((x, y), Slope::Down);
                    }
                    '<' => {
                        path.push((x, y));
                        slopes.insert((x, y), Slope::Left);
                    }
                    _ => {}
                }
            }
        }
        let grid = path.into_iter().collect::<Grid>();
        Ok((rest, (grid, slopes, (start_x, 0), (end_x, height - 1))))
    }

    type Output1 = usize;

    /// Part 1 took 12.724701ms
    fn part_1(input: &Self::Input) -> Self::Output1 {
        let (grid, slopes, start, end) = input;
        // create directed graph
        let (graph, path_indices) = get_graph(grid, slopes);
        let start_node = path_indices.get(start).unwrap();
        let end_node = path_indices.get(end).unwrap();

        // check all possible paths that visit each node at most once and check which is longest
        all_simple_paths::<Vec<_>, _>(&graph, *start_node, *end_node, 10, None)
            .map(|path| path.len() - 1)
            .max()
            .unwrap()
    }

    type Output2 = usize;

    /// Part 2 took 1.859898321s
    fn part_2(input: &Self::Input) -> Self::Output2 {
        let (grid, _, start, end) = input;
        // create undirected graph with segments between intersections merged into one edge with steps as the weigth
        let (graph, path_indices) = get_graph2(grid, start);
        let start_node = path_indices.get(start).unwrap();
        let end_node = path_indices.get(end).unwrap();

        if cfg!(test) {
            println!("{:?}", Dot::with_config(&graph, &[Config::NodeIndexLabel]));
        }

        // retrieve all paths that visit each node at most once and check which is longest
        all_simple_paths::<Vec<_>, _>(&graph, *start_node, *end_node, 0, None)
            .map(|path| {
                // get path length by summing the edge weights
                path.iter()
                    .tuple_windows()
                    .map(|(a, b)| graph.edge_weight(graph.find_edge(*a, *b).unwrap()).unwrap())
                    .sum()
            })
            .max()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = "#.#####################
#.......#########...###
#######.#########.#.###
###.....#.>.>.###.#.###
###v#####.#v#.###.#.###
###.>...#.#.#.....#...#
###v###.#.#.#########.#
###...#.#.#.......#...#
#####.#.#.#######.#.###
#.....#.#.#.......#...#
#.#####.#.#.#########v#
#.#...#...#...###...>.#
#.#.#v#######v###.###v#
#...#.>.#...>.>.#.###.#
#####v#.#.###v#.#.###.#
#.....#...#...#.#.#...#
#.#########.###.#.#.###
#...###...#...#...#.###
###.###.#.###v#####v###
#...#...#.#.>.>.#.>.###
#.###.###.#.###.#.#v###
#.....###...###...#...#
#####################.#";

    #[test]
    fn test_part1() {
        let parsed = Day23::parse(INPUT).unwrap().1;
        assert_eq!(Day23::part_1(&parsed), 94);
    }

    #[test]
    fn test_part2() {
        let parsed = Day23::parse(INPUT).unwrap().1;
        assert_eq!(Day23::part_2(&parsed), 154);
    }
}
