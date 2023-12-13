use std::collections::HashMap;

use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::{line_ending, one_of},
    combinator::map,
    multi::{many1, separated_list0},
    IResult,
};

use crate::days::Day;

pub struct Day13;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Tile {
    Rock,
    Ash,
}

impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tile::Rock => write!(f, "#"),
            Tile::Ash => write!(f, "."),
        }
    }
}

fn parse_pattern(input: &str) -> IResult<&str, Vec<Vec<Tile>>> {
    separated_list0(
        line_ending,
        map(many1(one_of("#.")), |s| {
            s.iter()
                .map(|c| match c {
                    '#' => Tile::Rock,
                    '.' => Tile::Ash,
                    _ => unimplemented!(),
                })
                .collect()
        }),
    )(input)
}

fn find_vertical_mirror(pattern: &[Vec<Tile>], smudge: bool) -> Option<usize> {
    // key = number of diverging tiles, value = x
    let mut res = HashMap::<usize, usize>::new();
    let w = pattern[0].len();
    let mut x = 1;
    while x < w {
        let mut symmetrical = true;
        let mut total_diverging = 0;
        for row in pattern {
            let (left, right) = row.split_at(x);
            let diverging = right
                .iter()
                .zip(left.iter().rev())
                .filter(|(a, b)| a != b)
                .count();
            if diverging > 1 {
                // not symmetrical, let's look at the next vertical position
                x += 1;
                symmetrical = false;
                break;
            }
            total_diverging += diverging;
        }
        if symmetrical {
            res.insert(total_diverging, x);
            x += 1;
        }
    }
    if smudge {
        res.get(&1).copied()
    } else {
        res.get(&0).copied()
    }
}

fn find_horizontal_mirror(pattern: &[Vec<Tile>], smudge: bool) -> Option<usize> {
    // key = number of diverging tiles, value = y
    let mut res = HashMap::<usize, usize>::new();
    let w = pattern[0].len();
    let h = pattern.len();
    let mut y = 1;
    while y < h {
        let mut symmetrical = true;
        let mut total_diverging = 0;
        for col in (0..w).map(|x| pattern.iter().map(|row| row[x]).collect_vec()) {
            let (top, bottom) = col.split_at(y);
            let diverging = bottom
                .iter()
                .zip(top.iter().rev())
                .filter(|(a, b)| a != b)
                .count();
            if diverging > 1 {
                // not symmetrical, let's look at the next vertical position
                y += 1;
                symmetrical = false;
                break;
            }
            total_diverging += diverging;
        }
        if symmetrical {
            res.insert(total_diverging, y);
            y += 1;
        }
    }
    if smudge {
        res.get(&1).copied()
    } else {
        res.get(&0).copied()
    }
}

impl Day for Day13 {
    type Input = Vec<Vec<Vec<Tile>>>;

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        separated_list0(tag("\n\n"), parse_pattern)(input)
    }

    type Output1 = usize;

    fn part_1(input: &Self::Input) -> Self::Output1 {
        input
            .iter()
            .map(|pattern| {
                if let Some(axis) = find_horizontal_mirror(pattern, false) {
                    axis * 100
                } else if let Some(axis) = find_vertical_mirror(pattern, false) {
                    axis
                } else {
                    0
                }
            })
            .sum()
    }

    type Output2 = usize;

    fn part_2(input: &Self::Input) -> Self::Output2 {
        input
            .iter()
            .map(|pattern| {
                if let Some(axis) = find_horizontal_mirror(pattern, true) {
                    axis * 100
                } else if let Some(axis) = find_vertical_mirror(pattern, true) {
                    axis
                } else {
                    0
                }
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = "#.##..##.
..#.##.#.
##......#
##......#
..#.##.#.
..##..##.
#.#.##.#.

#...##..#
#....#..#
..##..###
#####.##.
#####.##.
..##..###
#....#..#";

    #[test]
    fn test_part1() {
        let parsed = Day13::parse(INPUT).unwrap().1;
        assert_eq!(Day13::part_1(&parsed), 405);
    }

    #[test]
    fn test_part2() {
        let parsed = Day13::parse(INPUT).unwrap().1;
        assert_eq!(Day13::part_2(&parsed), 400);
    }
}
