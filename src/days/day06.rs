use std::{f64::EPSILON, ops::RangeInclusive};

use nom::{
    bytes::complete::tag,
    character::complete::{line_ending, space1, u64},
    combinator::map,
    multi::separated_list1,
    sequence::{separated_pair, tuple},
    IResult,
};

use crate::days::Day;

pub struct Day06;

#[derive(Debug)]
pub struct Race {
    total_time: u64,
    record_distance: u64,
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
fn winning_interval(race: &Race) -> RangeInclusive<u64> {
    // formula for winning the race: (t - x) * x > r, where t is the race total time and r is the previous record
    // this gives a lower bound of 0.5 * (t - sqrt(t^2 - 4*r)) and a higher bound of 0.5*(t + sqrt(t^2 - 4*r))
    let t = race.total_time as f64;
    let r = race.record_distance as f64;
    let sqrt = (t * t - 4. * r).sqrt();
    let mut low = 0.5 * (t - sqrt);
    let mut high = 0.5 * (t + sqrt);
    // since we have to go strictly farther to win, in case of an integer bound we have to add/subtract one
    if (low.ceil() - low).abs() < EPSILON {
        low += 0.1;
    }
    if (high.floor() - high).abs() < EPSILON {
        high -= 0.1;
    }
    (low.ceil() as u64)..=(high.floor() as u64)
}

fn interval_length(i: RangeInclusive<u64>) -> u64 {
    i.end() - i.start() + 1
}

#[allow(clippy::cast_precision_loss)]
fn parse_times(input: &str) -> IResult<&str, Vec<u64>> {
    map(
        tuple((tag("Time:"), space1, separated_list1(space1, u64))),
        |(_, _, numbers)| numbers,
    )(input)
}

#[allow(clippy::cast_precision_loss)]
fn parse_distances(input: &str) -> IResult<&str, Vec<u64>> {
    map(
        tuple((tag("Distance:"), space1, separated_list1(space1, u64))),
        |(_, _, numbers)| numbers,
    )(input)
}

impl Day for Day06 {
    type Input = Vec<Race>;

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        map(
            separated_pair(parse_times, line_ending, parse_distances),
            |(times, distances)| {
                times
                    .iter()
                    .zip(distances)
                    .map(|(&time, dist)| Race {
                        total_time: time,
                        record_distance: dist,
                    })
                    .collect()
            },
        )(input)
    }

    type Output1 = u64;

    /// Part 1 took 2.9µs (6.1µs with parsing)
    fn part_1(input: &Self::Input) -> Self::Output1 {
        input
            .iter()
            .map(|r| interval_length(winning_interval(r)))
            .product()
    }

    type Output2 = u64;

    /// took 1.7µs (4.9µs with parsing)
    fn part_2(input: &Self::Input) -> Self::Output2 {
        let race = input
            .iter()
            .fold((String::new(), String::new()), |acc, race| {
                (
                    format!("{}{}", acc.0, race.total_time),
                    format!("{}{}", acc.1, race.record_distance),
                )
            });
        let race = Race {
            total_time: race.0.parse().unwrap(),
            record_distance: race.1.parse().unwrap(),
        };
        interval_length(winning_interval(&race))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        let input = "Time:      7  15   30
Distance:  9  40  200";

        let parsed = Day06::parse(input).unwrap().1;
        assert_eq!(Day06::part_1(&parsed), 288);
    }

    #[test]
    fn test_part2() {
        let input = "Time:      7  15   30
Distance:  9  40  200";

        let parsed = Day06::parse(input).unwrap().1;
        assert_eq!(Day06::part_2(&parsed), 71503);
    }
}
