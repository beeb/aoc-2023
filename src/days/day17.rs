use itertools::Itertools;
use nom::{
    character::complete::{digit1, line_ending},
    combinator::map,
    multi::separated_list0,
    IResult,
};
use owo_colors::OwoColorize;
use pathfinding::prelude::astar;

use crate::days::Day;

static SIZE: i64 = if cfg!(test) { 13 } else { 141 };

pub struct Day17;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Dir {
    Up(i64),
    Right(i64),
    Down(i64),
    Left(i64),
}

impl Dir {
    fn steps(&self) -> i64 {
        match self {
            Dir::Up(steps) | Dir::Right(steps) | Dir::Down(steps) | Dir::Left(steps) => *steps,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Candidate {
    pub dir: Dir,
    pub x: i64,
    pub y: i64,
}

impl Candidate {
    /// Manhattan distance * minimum cost of 1 step = dist * 1
    fn distance(&self, other: &Self) -> i64 {
        (self.x.max(other.x) - self.x.min(other.x)) + (self.y.max(other.y) - self.y.min(other.y))
    }

    /// Super ugly but works
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn successors(&self, grid: &[Vec<i64>]) -> Vec<(Candidate, i64)> {
        // initially add all the possible moves (straight, turn left, turn right)
        let candidates = match self.dir {
            Dir::Up(steps) => [
                Candidate {
                    dir: Dir::Up(steps + 1),
                    x: self.x,
                    y: self.y - 1,
                },
                Candidate {
                    dir: Dir::Left(1),
                    x: self.x - 1,
                    y: self.y,
                },
                Candidate {
                    dir: Dir::Right(1),
                    x: self.x + 1,
                    y: self.y,
                },
            ],
            Dir::Right(steps) => [
                Candidate {
                    dir: Dir::Right(steps + 1),
                    x: self.x + 1,
                    y: self.y,
                },
                Candidate {
                    dir: Dir::Up(1),
                    x: self.x,
                    y: self.y - 1,
                },
                Candidate {
                    dir: Dir::Down(1),
                    x: self.x,
                    y: self.y + 1,
                },
            ],
            Dir::Down(steps) => [
                Candidate {
                    dir: Dir::Down(steps + 1),
                    x: self.x,
                    y: self.y + 1,
                },
                Candidate {
                    dir: Dir::Right(
                        1 + i64::from(
                            self.x == 0 && self.y == 0 && matches!(self.dir, Dir::Down(_)),
                        ),
                    ), // special case for start
                    x: self.x + 1,
                    y: self.y,
                },
                Candidate {
                    dir: Dir::Left(1),
                    x: self.x - 1,
                    y: self.y,
                },
            ],
            Dir::Left(steps) => [
                Candidate {
                    dir: Dir::Left(steps + 1),
                    x: self.x - 1,
                    y: self.y,
                },
                Candidate {
                    dir: Dir::Up(1),
                    x: self.x,
                    y: self.y - 1,
                },
                Candidate {
                    dir: Dir::Down(1),
                    x: self.x,
                    y: self.y + 1,
                },
            ],
        };
        // remove invalid moves (out of grid, straight path too long)
        candidates
            .into_iter()
            .filter_map(|c| {
                if c.dir.steps() <= 3 && c.x >= 0 && c.x < SIZE && c.y >= 0 && c.y < SIZE {
                    let x = c.x as usize;
                    let y = c.y as usize;
                    Some((c, grid[y][x]))
                } else {
                    None
                }
            })
            .collect_vec()
    }

    /// Super ugly but works
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::too_many_lines
    )]
    fn successors2(&self, grid: &[Vec<i64>]) -> Vec<(Candidate, i64)> {
        // Register the candidates (for now we allow out of grid)
        let mut candidates = Vec::with_capacity(3);
        match self.dir {
            Dir::Up(steps) => {
                // Straight. We can go at most 10 steps in the same direction
                if steps < 10 {
                    candidates.push(Candidate {
                        dir: Dir::Up(steps + 1),
                        x: self.x,
                        y: self.y - 1,
                    });
                }
                // Turn. We must go at least 4 steps in the same direction
                if steps >= 4 {
                    candidates.push(Candidate {
                        dir: Dir::Left(1),
                        x: self.x - 1,
                        y: self.y,
                    });
                    candidates.push(Candidate {
                        dir: Dir::Right(1),
                        x: self.x + 1,
                        y: self.y,
                    });
                }
            }
            Dir::Right(steps) => {
                if steps < 10 {
                    candidates.push(Candidate {
                        dir: Dir::Right(steps + 1),
                        x: self.x + 1,
                        y: self.y,
                    });
                }
                if steps >= 4 {
                    candidates.push(Candidate {
                        dir: Dir::Up(1),
                        x: self.x,
                        y: self.y - 1,
                    });
                    candidates.push(Candidate {
                        dir: Dir::Down(1),
                        x: self.x,
                        y: self.y + 1,
                    });
                }
            }
            Dir::Down(steps) => {
                // special case for start, where it could be either Dir::Down or Dir::Right
                if self.x == 0 && self.y == 0 && matches!(self.dir, Dir::Down(_)) {
                    candidates.push(Candidate {
                        dir: Dir::Right(2),
                        x: self.x + 1,
                        y: self.y,
                    });
                }
                if steps < 10 {
                    candidates.push(Candidate {
                        dir: Dir::Down(steps + 1),
                        x: self.x,
                        y: self.y + 1,
                    });
                }
                if steps >= 4 {
                    candidates.push(Candidate {
                        dir: Dir::Right(1),
                        x: self.x + 1,
                        y: self.y,
                    });
                    candidates.push(Candidate {
                        dir: Dir::Left(1),
                        x: self.x - 1,
                        y: self.y,
                    });
                }
            }
            Dir::Left(steps) => {
                if steps < 10 {
                    candidates.push(Candidate {
                        dir: Dir::Left(steps + 1),
                        x: self.x - 1,
                        y: self.y,
                    });
                }
                if steps >= 4 {
                    candidates.push(Candidate {
                        dir: Dir::Up(1),
                        x: self.x,
                        y: self.y - 1,
                    });
                    candidates.push(Candidate {
                        dir: Dir::Down(1),
                        x: self.x,
                        y: self.y + 1,
                    });
                }
            }
        };
        // filter out out of grid candidates
        candidates
            .into_iter()
            .filter_map(|c| {
                if c.x >= 0 && c.x < SIZE && c.y >= 0 && c.y < SIZE {
                    let x = c.x as usize;
                    let y = c.y as usize;
                    Some((c, grid[y][x]))
                } else {
                    None
                }
            })
            .collect_vec()
    }
}

/// Print a colorful representation of the path in the grid
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    unused
)]
fn print_path(path: &[Candidate], grid: &[Vec<i64>]) {
    let gradient = colorous::PLASMA;
    for (y, row) in grid.iter().enumerate() {
        for (x, value) in row.iter().enumerate() {
            let color = gradient.eval_rational(*value as usize, 9);
            if let Some(pos) = path.iter().find(|c| c.x == (x as i64) && c.y == (y as i64)) {
                let symbol = match pos.dir {
                    Dir::Up(_) => "^",
                    Dir::Right(_) => ">",
                    Dir::Down(_) => "v",
                    Dir::Left(_) => "<",
                };
                print!(
                    "{}",
                    symbol
                        .on_truecolor(color.r, color.g, color.b)
                        .truecolor(255, 255, 255)
                );
            } else {
                print!(
                    "{}",
                    char::from_digit(*value as u32, 10)
                        .unwrap()
                        .to_string()
                        .on_truecolor(color.r, color.g, color.b)
                        .truecolor(0, 0, 0)
                );
            }
        }
        println!();
    }
}

impl Day for Day17 {
    type Input = Vec<Vec<i64>>;

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        separated_list0(
            line_ending,
            map(digit1, |s: &str| {
                s.chars()
                    .map(|c| i64::from(c.to_digit(10).unwrap()))
                    .collect_vec()
            }),
        )(input)
    }

    type Output1 = i64;

    /// Part 1 took 43.9389ms
    fn part_1(input: &Self::Input) -> Self::Output1 {
        // We use A* to find the optimal path.
        // To see my implementation, see AoC 2022 day 12.
        // Here I used a library.
        let start = Candidate {
            dir: Dir::Down(1),
            x: 0,
            y: 0,
        };
        let goal = Candidate {
            dir: Dir::Down(0),
            x: SIZE - 1,
            y: SIZE - 1,
        };
        let result = astar(
            &start,
            |c| c.successors(input),
            |c| c.distance(&goal),
            |c| c.x == goal.x && c.y == goal.y,
        )
        .unwrap();
        //print_path(&result.0, input);
        result.1
    }

    type Output2 = i64;

    /// Part 2 took 197.254202ms
    fn part_2(input: &Self::Input) -> Self::Output2 {
        let start = Candidate {
            dir: Dir::Down(1),
            x: 0,
            y: 0,
        };
        let goal = Candidate {
            dir: Dir::Down(0),
            x: SIZE - 1,
            y: SIZE - 1,
        };
        let result = astar(
            &start,
            |c| c.successors2(input),
            |c| c.distance(&goal),
            |c| c.x == goal.x && c.y == goal.y,
        )
        .unwrap();
        //print_path(&result.0, input);
        result.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = "2413432311323
3215453535623
3255245654254
3446585845452
4546657867536
1438598798454
4457876987766
3637877979653
4654967986887
4564679986453
1224686865563
2546548887735
4322674655533";

    #[test]
    fn test_part1() {
        let parsed = Day17::parse(INPUT).unwrap().1;
        assert_eq!(Day17::part_1(&parsed), 102);
    }

    #[test]
    fn test_part2() {
        let parsed = Day17::parse(INPUT).unwrap().1;
        assert_eq!(Day17::part_2(&parsed), 94);
    }
}
