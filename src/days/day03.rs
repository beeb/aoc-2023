use std::collections::HashMap;

use itertools::Itertools;
use nom::{
    character::complete::{line_ending, not_line_ending},
    combinator::map,
    multi::separated_list0,
    IResult,
};

use crate::days::Day;

pub struct Day03;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Point {
    x: usize,
    y: usize,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct NumberPos {
    x_from: usize,
    x_to: usize,
    y: usize,
}

fn get_symbols(input: &[Vec<char>]) -> HashMap<Point, char> {
    input
        .iter()
        .enumerate()
        .flat_map(|(y, line)| {
            line.iter()
                .enumerate()
                .filter_map(|(x, char)| {
                    if *char == '.' || char.is_ascii_digit() {
                        return None;
                    }
                    Some((Point { x, y }, *char))
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn get_numbers(input: &[Vec<char>]) -> HashMap<Point, usize> {
    input
        .iter()
        .enumerate()
        .flat_map(|(y, line)| {
            let mut numbers = Vec::new();
            let mut x = 0;
            while x < line.len() {
                let char = line[x];
                if !char.is_ascii_digit() {
                    x += 1;
                    continue;
                }
                let number: String = line
                    .iter()
                    .skip(x)
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                let numlen = number.len();
                let num: usize = number.parse::<usize>().unwrap();
                numbers.push((Point { x, y }, num));
                if numlen > 1 {
                    // let's store the last position of the number too, for later checking if it's neighboring a star
                    numbers.push((
                        Point {
                            x: x + numlen - 1,
                            y,
                        },
                        num,
                    ));
                }
                x += numlen;
            }
            numbers
        })
        .collect()
}

fn get_stars(input: &[Vec<char>]) -> Vec<Point> {
    input
        .iter()
        .enumerate()
        .flat_map(|(y, line)| {
            line.iter()
                .enumerate()
                .filter_map(|(x, char)| {
                    if *char != '*' {
                        return None;
                    }
                    Some(Point { x, y })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn adjascent_symbol(
    symbols: &HashMap<Point, char>,
    number_x: usize,
    number_y: usize,
    number_len: usize,
) -> bool {
    let x_from = number_x.checked_sub(1).unwrap_or(number_x);
    let x_to = number_x + number_len;
    let y_from = number_y.checked_sub(1).unwrap_or(number_y);
    let y_to = number_y + 1;
    for y in y_from..=y_to {
        for x in x_from..=x_to {
            if y == number_y && x >= number_x && x < number_x + number_len {
                continue;
            }
            if symbols.contains_key(&Point { x, y }) {
                return true;
            }
        }
    }
    false
}

fn adjascent_numbers(numbers: &HashMap<Point, usize>, star_pos: &Point) -> Option<(usize, usize)> {
    let mut res = Vec::new();
    let x_from = star_pos.x.checked_sub(1).unwrap_or(star_pos.x);
    let x_to = star_pos.x + 1;
    let y_from = star_pos.y.checked_sub(1).unwrap_or(star_pos.y);
    let y_to = star_pos.y + 1;
    for y in y_from..=y_to {
        for x in x_from..=x_to {
            if y == star_pos.y && x == star_pos.x {
                continue;
            }
            if let Some(num) = numbers.get(&Point { x, y }) {
                // Avoid adding the same number twice in case it's start and end positions are neighboring the star
                // FIXME: in our case there aren't two separate numbers with the same value that are next to the same
                // star, but we could miss one if that were the case
                if res.contains(num) {
                    continue;
                }
                res.push(*num);
            }
        }
    }
    res.into_iter().collect_tuple()
}

impl Day for Day03 {
    type Input = Vec<Vec<char>>;

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        separated_list0(
            line_ending,
            map(not_line_ending, |s: &str| s.chars().collect::<Vec<_>>()),
        )(input)
    }

    type Output1 = usize;

    /// Part 1 took 0.3709ms
    fn part_1(input: &Self::Input) -> Self::Output1 {
        let symbols = get_symbols(input);
        input
            .iter()
            .enumerate()
            .flat_map(|(y, line)| {
                let mut numbers = Vec::new();
                let mut x = 0;
                while x < line.len() {
                    let char = line[x];
                    if !char.is_ascii_digit() {
                        x += 1;
                        continue;
                    }
                    let number: String = line
                        .iter()
                        .skip(x)
                        .take_while(|c| c.is_ascii_digit())
                        .collect();
                    if adjascent_symbol(&symbols, x, y, number.len()) {
                        numbers.push(number.parse::<usize>().unwrap());
                        x += number.len();
                    } else {
                        x += 1;
                    }
                }
                numbers
            })
            .sum()
    }

    type Output2 = usize;

    /// Part 2 took 0.3155ms
    fn part_2(input: &Self::Input) -> Self::Output2 {
        let stars = get_stars(input);
        let numbers = get_numbers(input);
        stars
            .iter()
            .filter_map(|star_pos| adjascent_numbers(&numbers, star_pos))
            .map(|(num1, num2)| num1 * num2)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        let input = "467..114..
...*......
..35..633.
......#...
617*......
.....+.58.
..592.....
......755.
...$.*....
.664.598..";

        let parsed = Day03::parse(input).unwrap().1;
        assert_eq!(Day03::part_1(&parsed), 4361);
    }

    #[test]
    fn test_part2() {
        let input = "467..114..
...*......
..35..633.
......#...
617*......
.....+.58.
..592.....
......755.
...$.*....
.664.598..";

        let parsed = Day03::parse(input).unwrap().1;
        assert_eq!(Day03::part_2(&parsed), 467_835);
    }
}
