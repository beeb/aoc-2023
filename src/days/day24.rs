use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::{i64, line_ending, space1},
    multi::{separated_list0, separated_list1},
    sequence::{separated_pair, tuple},
    IResult,
};
use num::integer::gcd;

use crate::days::Day;

const AREA_MIN: f64 = if cfg!(test) { 7. } else { 200_000_000_000_000. };
const AREA_MAX: f64 = if cfg!(test) {
    27.
} else {
    400_000_000_000_000.
};

pub struct Day24;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct V3 {
    x: i64,
    y: i64,
    z: i64,
}

#[derive(Debug, Clone)]
pub struct HailStone {
    pos: V3,
    vel: V3,
}

impl V3 {
    fn normalize(&self) -> V3 {
        let gcd = [self.x, self.y, self.z]
            .iter()
            .fold(0, |acc, i| gcd(acc, *i));
        V3 {
            x: self.x / gcd,
            y: self.y / gcd,
            z: self.z / gcd,
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn is_parallel(&self, other: &Self) -> bool {
        self.x as f64 * other.y as f64 - other.x as f64 * self.y as f64 == 0.
    }
}

impl HailStone {
    fn pos_after(&self, time: i64) -> V3 {
        V3 {
            x: self.pos.x + time * self.vel.x,
            y: self.pos.y + time * self.vel.y,
            z: self.pos.z + time * self.vel.z,
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn intersection_with(&self, other: &HailStone) -> Option<(f64, f64)> {
        let (p1x, p1y, v1x, v1y) = (
            self.pos.x as f64,
            self.pos.y as f64,
            self.vel.x as f64,
            self.vel.y as f64,
        );
        let (p2x, p2y, v2x, v2y) = (
            other.pos.x as f64,
            other.pos.y as f64,
            other.vel.x as f64,
            other.vel.y as f64,
        );

        let t2 = ((p2y - p1y) * v1x - (p2x - p1x) * v1y) / (v2x * v1y - v2y * v1x);
        let t1 = (p2x - p1x + t2 * v2x) / v1x;

        if !t1.is_sign_positive() || !t2.is_sign_positive() {
            return None;
        }

        let x = p1x + t1 * v1x;
        let y = p1y + t1 * v1y;
        Some((x, y))
    }
}

fn parse_vec(input: &str) -> IResult<&str, V3> {
    let (rest, coords) = separated_list1(tuple((tag(","), space1)), i64)(input)?;
    let (x, y, z) = coords.into_iter().collect_tuple().unwrap();
    Ok((rest, V3 { x, y, z }))
}

fn parse_hailstone(input: &str) -> IResult<&str, HailStone> {
    let (rest, (position, speed)) =
        separated_pair(parse_vec, tuple((space1, tag("@"), space1)), parse_vec)(input)?;
    Ok((
        rest,
        HailStone {
            pos: position,
            vel: speed,
        },
    ))
}

impl Day for Day24 {
    type Input = Vec<HailStone>;

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        separated_list0(line_ending, parse_hailstone)(input)
    }

    type Output1 = usize;

    /// Part 1 took 129.3µs
    fn part_1(input: &Self::Input) -> Self::Output1 {
        input
            .iter()
            .tuple_combinations()
            .filter(|(a, b)| {
                let Some((x, y)) = a.intersection_with(b) else {
                    return false;
                };
                (AREA_MIN..=AREA_MAX).contains(&x) && (AREA_MIN..=AREA_MAX).contains(&y)
            })
            .count()
    }

    type Output2 = usize;

    fn part_2(_input: &Self::Input) -> Self::Output2 {
        unimplemented!("part_2")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = "19, 13, 30 @ -2,  1, -2
18, 19, 22 @ -1, -1, -2
20, 25, 34 @ -2, -2, -4
12, 31, 28 @ -1, -2, -1
20, 19, 15 @  1, -5, -3";

    #[test]
    fn test_part1() {
        let parsed = Day24::parse(INPUT).unwrap().1;
        assert_eq!(Day24::part_1(&parsed), 2);
    }

    #[test]
    fn test_intersection() {
        let a = HailStone {
            pos: V3 {
                x: 19,
                y: 13,
                z: 30,
            },
            vel: V3 { x: -2, y: 1, z: -2 },
        };
        let b = HailStone {
            pos: V3 {
                x: 18,
                y: 19,
                z: 22,
            },
            vel: V3 {
                x: -1,
                y: -1,
                z: -2,
            },
        };
        let (x, y) = a.intersection_with(&b).unwrap();
        assert_eq!(x, 14.333333333333334);
        assert_eq!(y, 15.333333333333332);
    }

    #[test]
    fn test_intersection_past() {
        let a = HailStone {
            pos: V3 {
                x: 19,
                y: 13,
                z: 30,
            },
            vel: V3 { x: -2, y: 1, z: -2 },
        };
        let b = HailStone {
            pos: V3 {
                x: 20,
                y: 19,
                z: 15,
            },
            vel: V3 { x: 1, y: -5, z: -3 },
        };
        assert_eq!(a.intersection_with(&b), None);
    }
}
