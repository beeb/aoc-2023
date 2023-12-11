use itertools::Itertools;
use nom::{
    character::complete::{line_ending, not_line_ending},
    combinator::map,
    multi::separated_list0,
    IResult,
};

use crate::days::Day;

pub struct Day11;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone)]
pub struct Point(isize, isize);

impl Point {
    /// Manhattan distance between two points
    fn dist(&self, other: &Self) -> isize {
        (self.0.max(other.0) - self.0.min(other.0)) + (self.1.max(other.1) - self.1.min(other.1))
    }
}

/// Transpose a Vec of Vecs
fn transpose<T>(v: &[Vec<T>]) -> Vec<Vec<T>>
where
    T: Clone,
{
    assert!(!v.is_empty());
    (0..v[0].len())
        .map(|i| v.iter().map(|inner| inner[i].clone()).collect_vec())
        .collect()
}

/// Get a list of coordinates of the galaxies, taking into account the expansion
#[allow(clippy::cast_possible_wrap)]
fn get_galaxies(input: &[Vec<char>], expansion: isize) -> Vec<Point> {
    let mut original_galaxies = Vec::with_capacity(500);
    // keep track of empty rows while iterating
    let mut empty_rows = vec![];
    for (y, row) in input.iter().enumerate() {
        let mut empty_row = true;
        for (x, &c) in row.iter().enumerate() {
            if c == '#' {
                // we found a galaxy, so the row cannot be empty
                empty_row = false;
                original_galaxies.push(Point(x as isize, y as isize));
            }
        }
        if empty_row {
            empty_rows.push(y);
        }
    }
    // transpose the matrix to find the empty columns
    let tp = transpose(input);
    let empty_cols = tp
        .into_iter()
        .enumerate()
        .filter_map(|(x, col)| col.iter().all(|c| *c != '#').then_some(x))
        .collect_vec();

    // adjust the coordinates to reflect the expansion of empty cols and rows
    original_galaxies
        .into_iter()
        .map(|g| {
            // how many rows and columns are empty above and to the left of this galaxy?
            let empty_rows_above = empty_rows
                .iter()
                .take_while(|&&y| (y as isize) < g.1)
                .count() as isize;
            let empty_cols_left = empty_cols
                .iter()
                .take_while(|&&x| (x as isize) < g.0)
                .count() as isize;
            // for an expansion of two, we double each empty col/row
            // so we add their number multiplied by expansion - 1 to the respective coordinates
            Point(
                g.0 + empty_cols_left * (expansion - 1),
                g.1 + empty_rows_above * (expansion - 1),
            )
        })
        .collect()
}

impl Day for Day11 {
    type Input = Vec<Vec<char>>;

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        separated_list0(
            line_ending,
            map(not_line_ending, |s: &str| s.chars().collect_vec()),
        )(input)
    }

    type Output1 = isize;

    /// Part 1 took 119.083µs
    fn part_1(input: &Self::Input) -> Self::Output1 {
        let galaxies = get_galaxies(input, 2);
        galaxies
            .iter()
            .tuple_combinations()
            .map(|(a, b)| a.dist(b))
            .sum()
    }

    type Output2 = isize;

    /// Part 2 took 95.91µs
    fn part_2(input: &Self::Input) -> Self::Output2 {
        let expansion = if cfg!(test) { 10 } else { 1_000_000 };
        let galaxies = get_galaxies(input, expansion);
        galaxies
            .iter()
            .tuple_combinations()
            .map(|(a, b)| a.dist(b))
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = "...#......
.......#..
#.........
..........
......#...
.#........
.........#
..........
.......#..
#...#.....";

    #[test]
    fn test_part1() {
        let parsed = Day11::parse(INPUT).unwrap().1;
        assert_eq!(Day11::part_1(&parsed), 374);
    }

    #[test]
    fn test_part2() {
        let parsed = Day11::parse(INPUT).unwrap().1;
        assert_eq!(Day11::part_2(&parsed), 1030);
    }
}
