use std::collections::HashSet;

use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::{i64, line_ending, space1},
    multi::{separated_list0, separated_list1},
    sequence::{separated_pair, tuple},
    IResult,
};

use crate::days::Day;

const AREA_MIN: f64 = if cfg!(test) { 7. } else { 200_000_000_000_000. };
const AREA_MAX: f64 = if cfg!(test) {
    27.
} else {
    400_000_000_000_000.
};

pub struct Day24;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
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

impl HailStone {
    #[allow(clippy::cast_precision_loss, clippy::similar_names)]
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

    type Output2 = i64;

    /// Part 2 took 999.1µs
    #[allow(
        clippy::similar_names,
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation
    )]
    fn part_2(input: &Self::Input) -> Self::Output2 {
        // let's find the velocity that our rock must have, by considering pairs of hailstones that have the same
        // (large) velocity on one axis
        let mut vel_x: Option<HashSet<i64>> = None;
        let mut vel_y: Option<HashSet<i64>> = None;
        let mut vel_z: Option<HashSet<i64>> = None;
        for (a, b) in input.iter().tuple_combinations().filter(|(a, b)| {
            (a.vel.x == b.vel.x && a.vel.x.abs() > 100)
                || (a.vel.y == b.vel.y && a.vel.y.abs() > 100)
                || (a.vel.z == b.vel.z && a.vel.z.abs() > 100)
        }) {
            // a and b go at the same velocity on at least one axis, so that means their distance on that axis is
            // constant. Then, the velocity of the rock on that axis must satisfy: dist % (rock.vel - hail.vel) == 0
            // in order for it to encounter both hailstones
            let (dist, hailstone_vel, velocities) = match (a.vel.x == b.vel.x, a.vel.y == b.vel.y) {
                (true, _) => (b.pos.x - a.pos.x, a.vel.x, &mut vel_x),
                (false, true) => (b.pos.y - a.pos.y, a.vel.y, &mut vel_y),
                (false, false) => (b.pos.z - a.pos.z, a.vel.z, &mut vel_z),
            };
            let mut candidates = HashSet::<i64>::new();
            // let's check velocities in a realistic range that match the equation
            for v in -1000..1000 {
                if v == hailstone_vel {
                    // would divide by zero
                    continue;
                }
                if dist % (v - hailstone_vel) == 0 {
                    candidates.insert(v);
                }
            }
            // add the candidates to the set, or compute the intersection with previous candidates
            if let Some(velocities) = velocities {
                *velocities = velocities.intersection(&candidates).copied().collect();
            } else {
                *velocities = Some(candidates);
            }
        }
        // we now know the velocity of the rock
        let (rvx, rvy, rvz) = (
            vel_x.unwrap().into_iter().next().unwrap() as f64,
            vel_y.unwrap().into_iter().next().unwrap() as f64,
            vel_z.unwrap().into_iter().next().unwrap() as f64,
        );

        // we can take any two hailstones and subtract the rock velocity to each to find two lines where our rock
        // starting position could lie. The intersection of the two lines is our rock starting position.
        let a = input.first().unwrap();
        let b = input.get(2).unwrap();
        // find intersection on X-Y
        let ma = (a.vel.y as f64 - rvy) / (a.vel.x as f64 - rvx);
        let mb = (b.vel.y as f64 - rvy) / (b.vel.x as f64 - rvx);
        let ca = a.pos.y as f64 - (ma * a.pos.x as f64);
        let cb = b.pos.y as f64 - (mb * b.pos.x as f64);
        let rx = ((cb - ca) / (ma - mb)).round();
        let ry = (ma * rx + ca).round();
        // check at which time we intersect with a
        let time = (rx - a.pos.x as f64) / (a.vel.x as f64 - rvx);
        // what was the z position at time zero?
        let rz = a.pos.z as f64 + (a.vel.z as f64 - rvz) * time;
        rx as i64 + ry as i64 + rz as i64
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

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
        assert_relative_eq!(x, 14.333_333_333_333_334);
        assert_relative_eq!(y, 15.333_333_333_333_332);
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
