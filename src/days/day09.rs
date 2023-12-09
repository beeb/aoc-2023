use itertools::Itertools;
use nom::{
    character::complete::{i64, line_ending, space1},
    multi::{separated_list0, separated_list1},
    IResult,
};

use crate::days::Day;

pub struct Day09;

fn extrapolate(sensor: &[i64], part2: bool) -> i64 {
    let mut stack: Vec<Vec<i64>> = Vec::with_capacity(100);
    stack.push(sensor.into());
    let mut extrapolated = loop {
        let diff: Vec<_> = stack
            .last()
            .unwrap()
            .iter()
            .tuple_windows()
            .map(|(a, b)| b - a)
            .collect();
        match diff.iter().all_equal_value() {
            Err(None) => unimplemented!("empty list"),
            Err(Some(_)) => {
                // not all values are equal, continue looking
                stack.push(diff);
            }
            Ok(value) => {
                // all values are equal, let's get the value to add to the last list
                break *value;
            }
        }
    };
    // extrapolate the previous list until we reach the original sensor list
    while let Some(list) = stack.pop() {
        if part2 {
            extrapolated = list.first().unwrap() - extrapolated;
        } else {
            extrapolated += list.last().unwrap();
        }
    }
    extrapolated
}

impl Day for Day09 {
    type Input = Vec<Vec<i64>>;

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        separated_list0(line_ending, separated_list1(space1, i64))(input)
    }

    type Output1 = i64;

    /// Part 1 took 78.5µs
    fn part_1(input: &Self::Input) -> Self::Output1 {
        input.iter().map(|sensor| extrapolate(sensor, false)).sum()
    }

    type Output2 = i64;

    /// Part 2 took 75.8µs
    fn part_2(input: &Self::Input) -> Self::Output2 {
        input.iter().map(|sensor| extrapolate(sensor, true)).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = "0 3 6 9 12 15
1 3 6 10 15 21
10 13 16 21 30 45";

    #[test]
    fn test_part1() {
        let parsed = Day09::parse(INPUT).unwrap().1;
        assert_eq!(Day09::part_1(&parsed), 114);
    }

    #[test]
    fn test_part2() {
        let parsed = Day09::parse(INPUT).unwrap().1;
        assert_eq!(Day09::part_2(&parsed), 2);
    }
}
