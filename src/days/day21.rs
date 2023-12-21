use std::collections::BTreeSet;

use itertools::Itertools;
use nom::{
    character::complete::{line_ending, not_line_ending},
    multi::separated_list0,
    IResult,
};
use pathfinding::{directed::bfs::bfs_reach, grid::Grid};

use crate::days::Day;

const STEPS_PART1: usize = if cfg!(test) { 6 } else { 64 };
const STEPS_PART2: usize = if cfg!(test) { 5000 } else { 26_501_365 };

pub struct Day21;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Pos {
    coord: (usize, usize),
    dist: usize,
}

fn count_pos(grid: &Grid, start_pos: &Pos, steps: usize) -> usize {
    bfs_reach(*start_pos, |n| {
        grid.neighbours(n.coord)
            .into_iter()
            .filter_map(|c| {
                if n.dist >= steps {
                    return None;
                }
                Some(Pos {
                    coord: c,
                    dist: n.dist + 1,
                })
            })
            .collect_vec()
    })
    .filter(|n| n.dist == steps)
    .count()
}

impl Day for Day21 {
    type Input = (Grid, (usize, usize), usize, usize);

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        let (rest, rows) = separated_list0(line_ending, not_line_ending)(input)?;
        let mut start_x = 0;
        let mut start_y = 0;
        let mut width = 0;
        let height = rows.len();
        let mut plots = vec![];
        for (y, row) in rows.into_iter().enumerate() {
            if width == 0 {
                width = row.len();
            }
            for (x, c) in row.chars().enumerate() {
                match c {
                    '.' => plots.push((x, y)),
                    'S' => {
                        plots.push((x, y));
                        start_x = x;
                        start_y = y;
                    }
                    _ => {}
                }
            }
        }
        let grid = plots.into_iter().collect::<Grid>();
        Ok((rest, (grid, (start_x, start_y), width, height)))
    }

    type Output1 = usize;

    /// Part 1 took 7.78602ms
    fn part_1(input: &Self::Input) -> Self::Output1 {
        let (grid, start, _, _) = input;
        let start_pos = Pos {
            coord: *start,
            dist: 0,
        };
        count_pos(grid, &start_pos, STEPS_PART1)
    }

    type Output2 = usize;

    /// Needed a bunch of help for this one...
    ///
    /// First we notice that in the grid, the path straight up/down/left/right from the start has no rocks. This
    /// means the elf can get to the edge of the grid with 65 steps in those directions. Then, we notice that there are
    /// diagonals (joining the middle point of each border of the grid) also without rocks.
    ///
    /// This means that when we go very far in either direction with a fixed number of steps (a multiple of the grid
    /// width + half of the grid width => 2 * 131 + 65 = 327 steps in the example below), we will end up with the
    /// following area visited:
    ///
    ///           ╎   ╎
    ///          ╌╆━━━╅╌
    ///           ┃/ \┃
    ///           ┃ O ┃
    ///       ╎  /┃   ┃\  ╎
    ///      ╌╆━━━╋━━━╋━━━╅╌
    ///       ┃/  ┃   ┃  \┃
    ///       ┃ O ┃ E ┃ O ┃
    ///   ╎  /┃   ┃   ┃   ┃\  ╎
    ///  ╌╆━━━╋━━━╋━━━╋━━━╋━━━╅╌
    ///   ┃/  ┃   ┃   ┃   ┃  \┃
    ///   ┃ O ┃ E ┃ S ┃ E ┃ O ┃
    ///   ┃\  ┃   ┃ O ┃   ┃  /┃
    ///  ╌╄━━━╋━━━╋━━━╋━━━╋━━━╃╌
    ///   ╎  \┃   ┃   ┃   ┃/  ╎
    ///       ┃ O ┃ E ┃ O ┃
    ///       ┃\  ┃   ┃  /┃
    ///      ╌╄━━━╋━━━╋━━━╃╌
    ///       ╎  \┃   ┃/  ╎
    ///           ┃ O ┃
    ///           ┃\ /┃
    ///          ╌╄━━━╃╌
    ///           ╎   ╎
    ///
    /// Since the number of steps is odd, we can only end up in garden plots with an odd number of moves for the center
    /// tile. But if we reach the edge of the tile, then on the next tile over we will can reach the positions that were
    /// unreachable in the center tile (those reachable with an even number of steps when considering the center tile).
    /// So, we count how many positions are reached by an even or odd number of steps in a tile.
    /// In the example above, we would have 1 full tile with odd positions (the center one), 4 tiles with even positions
    /// (the ones labelled "E"), then a bunch of incomplete tiles with odd positions, then a few stray corners of even
    /// tiles. We can calculate how many positions lie in those corners and subtract them in our final calculation.
    /// Note that the "O" incomplete tiles are missing overall 12 corners, that amount to 3 full "O" tiles for the
    /// example above. Likewise, we have 8 orphan corners for the even tiles on the outside, that make up 2 full "E"
    /// tiles.
    ///
    /// Part 2 took 5.797118ms
    fn part_2(input: &Self::Input) -> Self::Output2 {
        let (grid, start, width, _) = input;
        let start_pos = Pos {
            coord: *start,
            dist: 0,
        };

        // how many tiles are reachable
        let mut visited = BTreeSet::<(usize, usize)>::new();
        visited.insert(*start);
        let all_moves = bfs_reach(start_pos, |n| {
            grid.neighbours(n.coord)
                .into_iter()
                .filter_map(|c| {
                    if visited.contains(&c) {
                        return None;
                    }
                    visited.insert(c);
                    Some(Pos {
                        coord: c,
                        dist: n.dist + 1,
                    })
                })
                .collect_vec()
        })
        .collect_vec();

        // how many tiles are reachable by even moves
        let even_moves = all_moves.iter().filter(|m| m.dist % 2 == 0).count();

        // how many tiles are reachable by odd moves
        let odd_moves = all_moves.iter().filter(|m| m.dist % 2 == 1).count();

        // how many tiles are reachable by even moves and lie in the corners of the tile
        let even_corners = all_moves
            .iter()
            .filter(|m| m.dist % 2 == 0 && m.dist > width / 2)
            .count();

        // how many tiles are reachable by odd moves and lie in the corners of the tile
        let odd_corners = all_moves
            .iter()
            .filter(|m| m.dist % 2 == 1 && m.dist > width / 2)
            .count();

        let dim = STEPS_PART2 / width; // how many units of the grid we would be traversing if going in a straight direction for the total number of steps (= half of the diamond width)
                                       // this is equal to 202_300 in our case

        // The width of the "diamond" would be twice that value (+1). So the total number of tiles is roughly half of
        // those of a square with same width/heigth:
        // 0.5 * (2*dim+1) * (2*dim+1) = 2 * (dim + 1/2) * (dim + 1/2). The larger half of those is Odd, while the
        // other half is even.

        // all the odd tiles + all the even tiles - the missing corners of some of the odd tiles + the extra corners of
        // the incomplete even tiles
        ((dim + 1) * (dim + 1)) * odd_moves + (dim * dim) * even_moves - (dim + 1) * odd_corners
            + dim * even_corners
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const INPUT: &str = "...........
.....###.#.
.###.##..#.
..#.#...#..
....#.#....
.##..S####.
.##..#...#.
.......##..
.##.#.####.
.##..##.##.
...........";

    #[test]
    fn test_part1() {
        let parsed = Day21::parse(INPUT).unwrap().1;
        assert_eq!(Day21::part_1(&parsed), 16);
    }

    #[test]
    fn test_part2() {
        let parsed = Day21::parse(INPUT).unwrap().1;
        assert_eq!(Day21::part_2(&parsed), 16_733_044);
    }
}
