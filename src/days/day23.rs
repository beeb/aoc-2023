use std::collections::{BTreeMap, HashMap};

use itertools::Itertools;
use nom::{
    character::complete::{line_ending, not_line_ending},
    multi::separated_list0,
    IResult,
};
use pathfinding::grid::Grid;
use petgraph::{algo::all_simple_paths, prelude::*};

use crate::days::Day;

pub struct Day23;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Slope {
    Up,
    Right,
    Down,
    Left,
}

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
        if let Some(slope) = slopes.get(&path) {
            match slope {
                Slope::Up => {
                    if path.1 == 0 {
                        continue;
                    }
                    if let Some(b) = path_indices.get(&(path.0, path.1 - 1)) {
                        graph.add_edge(*a, *b, ());
                    }
                }
                Slope::Right => {
                    if let Some(b) = path_indices.get(&(path.0 + 1, path.1)) {
                        graph.add_edge(*a, *b, ());
                    }
                }
                Slope::Down => {
                    if let Some(b) = path_indices.get(&(path.0, path.1 + 1)) {
                        graph.add_edge(*a, *b, ());
                    }
                }
                Slope::Left => {
                    if path.0 == 0 {
                        continue;
                    }
                    if let Some(b) = path_indices.get(&(path.0 - 1, path.1)) {
                        graph.add_edge(*a, *b, ());
                    }
                }
            }
        } else {
            #[allow(clippy::cast_possible_wrap)]
            let path_isize = (path.0 as isize, path.1 as isize);
            for n in grid.neighbours(path) {
                #[allow(clippy::cast_possible_wrap)]
                let n_isize = (n.0 as isize, n.1 as isize);
                if let Some(slope) = slopes.get(&n) {
                    // check if slope of n points towards path
                    match ((n_isize.0 - path_isize.0, n_isize.1 - path_isize.1), slope) {
                        ((-1, 0), Slope::Right)
                        | ((0, -1), Slope::Down)
                        | ((1, 0), Slope::Left)
                        | ((0, 1), Slope::Up) => {
                            continue;
                        }
                        _ => {}
                    }
                }
                let b = path_indices.get(&n).unwrap();
                if !graph.contains_edge(*a, *b) {
                    graph.add_edge(*a, *b, ());
                }
                if !graph.contains_edge(*b, *a) {
                    graph.add_edge(*b, *a, ());
                }
            }
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

    fn part_1(input: &Self::Input) -> Self::Output1 {
        let (grid, slopes, start, end) = input;
        let (graph, path_indices) = get_graph(grid, slopes);
        let start_node = path_indices.get(start).unwrap();
        let end_node = path_indices.get(end).unwrap();

        all_simple_paths::<Vec<_>, _>(&graph, *start_node, *end_node, 10, None)
            .map(|path| path.len() - 1)
            .max()
            .unwrap()
    }

    type Output2 = usize;

    fn part_2(_input: &Self::Input) -> Self::Output2 {
        unimplemented!("part_2")
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

    // #[test]
    // fn test_part2() {
    //     let parsed = Day11::parse(INPUT).unwrap().1;
    //     assert_eq!(Day11::part_2(&parsed), 1030);
    // }
}
