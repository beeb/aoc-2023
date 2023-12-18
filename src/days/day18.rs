use nom::{
    bytes::complete::tag,
    character::complete::{char, hex_digit1, i64, line_ending, one_of, space1},
    combinator::map,
    multi::separated_list0,
    sequence::{preceded, terminated, tuple},
    IResult,
};

use crate::days::Day;

pub struct Day18;

#[derive(Debug)]
pub enum Dir {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug)]
pub struct Instruction {
    dir: Dir,
    dist: i64,
    color: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Point {
    y: i64,
    x: i64,
}

#[allow(clippy::cast_possible_wrap)]
fn get_trench_coordinates(input: &[Instruction], part2: bool) -> (Vec<Point>, i64) {
    let mut trench = Vec::<Point>::new();
    // save the total distance
    let mut total_dist = 0i64;
    let mut x = 0i64;
    let mut y = 0i64;
    for instr in input {
        let mut dist = instr.dist;
        if part2 {
            dist = u64::from_str_radix(&instr.color[..5], 16).unwrap() as i64;
        }
        total_dist += dist;
        if part2 {
            match &instr.color.chars().nth(5).unwrap() {
                '3' => y -= dist,
                '0' => x += dist,
                '1' => y += dist,
                '2' => x -= dist,
                _ => unreachable!(),
            }
        } else {
            match instr.dir {
                Dir::Up => y -= dist,
                Dir::Right => x += dist,
                Dir::Down => y += dist,
                Dir::Left => x -= dist,
            }
        }
        trench.push(Point { y, x });
    }
    (trench, total_dist)
}

fn pool_area(trench: &[Point], perimeter: i64) -> i64 {
    let mut double_area = 0i64;
    let len = trench.len();
    // algorithm to find double of the area of a non-intersecting polygon without holes
    // the area can be negative depending on the clockwise or anti-clockwise direction of the sequence
    for i in 0..len {
        let j = (i + 1) % len;
        let this = trench.get(i).unwrap();
        let other = trench.get(j).unwrap();
        double_area += this.x * other.y;
        double_area -= this.y * other.x;
    }
    // the area is counting half of the trench volumes only, so we need to add the other half
    // is the +1 always needed?
    double_area.abs() / 2 + perimeter / 2 + 1
}

impl Day for Day18 {
    type Input = Vec<Instruction>;

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        separated_list0(
            line_ending,
            map(
                tuple((
                    one_of("UDLR"),
                    space1,
                    i64,
                    space1,
                    preceded(tag("(#"), terminated(hex_digit1, char(')'))),
                )),
                |(dir, _, dist, _, color): (char, &str, i64, &str, &str)| {
                    let dir = match dir {
                        'U' => Dir::Up,
                        'R' => Dir::Right,
                        'D' => Dir::Down,
                        'L' => Dir::Left,
                        _ => unreachable!(),
                    };
                    Instruction {
                        dir,
                        dist,
                        color: color.to_string(),
                    }
                },
            ),
        )(input)
    }

    type Output1 = i64;

    fn part_1(input: &Self::Input) -> Self::Output1 {
        let (trench, perimeter) = get_trench_coordinates(input, false);
        pool_area(&trench, perimeter)
    }

    type Output2 = i64;

    #[allow(clippy::cast_possible_wrap)]
    fn part_2(input: &Self::Input) -> Self::Output2 {
        let (trench, perimeter) = get_trench_coordinates(input, true);
        pool_area(&trench, perimeter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const INPUT: &str = "R 6 (#70c710)
D 5 (#0dc571)
L 2 (#5713f0)
D 2 (#d2c081)
R 2 (#59c680)
D 2 (#411b91)
L 5 (#8ceee2)
U 2 (#caa173)
L 1 (#1b58a2)
U 2 (#caa171)
R 2 (#7807d2)
U 3 (#a77fa3)
L 2 (#015232)
U 2 (#7a21e3)";

    #[test]
    fn test_part1() {
        let parsed = Day18::parse(INPUT).unwrap().1;
        assert_eq!(Day18::part_1(&parsed), 62);
    }

    #[test]
    fn test_part2() {
        let parsed = Day18::parse(INPUT).unwrap().1;
        assert_eq!(Day18::part_2(&parsed), 952_408_144_115);
    }
}
