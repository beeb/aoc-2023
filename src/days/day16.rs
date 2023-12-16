use std::collections::{HashMap, VecDeque};

use nom::{
    character::complete::{line_ending, one_of},
    multi::{many1, separated_list0},
    IResult,
};

use crate::days::Day;

pub struct Day16;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Dir {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug)]
pub enum Tile {
    Empty,
    Vertical,
    Horizontal,
    Slash,
    BackSlash,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Point {
    x: isize,
    y: isize,
}

#[allow(clippy::cast_possible_wrap)]
impl Point {
    fn at_dir(&self, dir: Dir, width: usize, height: usize) -> Option<Point> {
        let at = match dir {
            Dir::Top => Point {
                x: self.x,
                y: self.y - 1,
            },
            Dir::Right => Point {
                x: self.x + 1,
                y: self.y,
            },
            Dir::Bottom => Point {
                x: self.x,
                y: self.y + 1,
            },
            Dir::Left => Point {
                x: self.x - 1,
                y: self.y,
            },
        };
        if at.x < 0 || at.x > (width as isize) - 1 || at.y < 0 || at.y > (height as isize) - 1 {
            return None;
        }
        Some(at)
    }
}

#[derive(Debug)]
pub struct Grid {
    grid: HashMap<Point, Tile>,
    width: usize,
    height: usize,
}

fn get_beams(grid: &Grid, start_pos: Point, start_dir: Dir) -> HashMap<Point, Vec<Dir>> {
    let mut beams = HashMap::<Point, Vec<Dir>>::new();
    let mut stack = VecDeque::<(Point, Dir)>::new();
    stack.push_front((start_pos, start_dir));
    while let Some((pos, dir)) = stack.pop_front() {
        // mark visited
        match beams.get_mut(&pos) {
            Some(dirs) => {
                if dirs.contains(&dir) {
                    continue;
                }
                dirs.push(dir);
            }
            None => {
                beams.insert(pos.clone(), vec![dir]);
            }
        }
        let tile = grid.grid.get(&pos).unwrap();
        match (tile, dir) {
            (Tile::Empty, _)
            | (Tile::Vertical, Dir::Top | Dir::Bottom)
            | (Tile::Horizontal, Dir::Left | Dir::Right) => {
                // continue in the same dir
                if let Some(next) = pos.at_dir(dir, grid.width, grid.height) {
                    stack.push_back((next, dir));
                }
            }
            (Tile::Vertical, Dir::Right | Dir::Left) => {
                // split beam if we come perpendicular
                if let Some(next) = pos.at_dir(Dir::Top, grid.width, grid.height) {
                    stack.push_back((next, Dir::Top));
                }
                if let Some(next) = pos.at_dir(Dir::Bottom, grid.width, grid.height) {
                    stack.push_back((next, Dir::Bottom));
                }
            }
            (Tile::Horizontal, Dir::Top | Dir::Bottom) => {
                // split beam if we come perpendicular
                if let Some(next) = pos.at_dir(Dir::Left, grid.width, grid.height) {
                    stack.push_back((next, Dir::Left));
                }
                if let Some(next) = pos.at_dir(Dir::Right, grid.width, grid.height) {
                    stack.push_back((next, Dir::Right));
                }
            }
            (Tile::Slash, Dir::Left) | (Tile::BackSlash, Dir::Right) => {
                // continue down
                if let Some(next) = pos.at_dir(Dir::Bottom, grid.width, grid.height) {
                    stack.push_back((next, Dir::Bottom));
                }
            }
            (Tile::Slash, Dir::Bottom) | (Tile::BackSlash, Dir::Top) => {
                // continue left
                if let Some(next) = pos.at_dir(Dir::Left, grid.width, grid.height) {
                    stack.push_back((next, Dir::Left));
                }
            }
            (Tile::Slash, Dir::Right) | (Tile::BackSlash, Dir::Left) => {
                // continue up
                if let Some(next) = pos.at_dir(Dir::Top, grid.width, grid.height) {
                    stack.push_back((next, Dir::Top));
                }
            }
            (Tile::Slash, Dir::Top) | (Tile::BackSlash, Dir::Bottom) => {
                // continue right
                if let Some(next) = pos.at_dir(Dir::Right, grid.width, grid.height) {
                    stack.push_back((next, Dir::Right));
                }
            }
        }
    }
    beams
}

impl Day for Day16 {
    type Input = Grid;

    #[allow(clippy::cast_possible_wrap)]
    fn parse(input: &str) -> IResult<&str, Self::Input> {
        let (_, tiles) = separated_list0(line_ending, many1(one_of(".|-/\\")))(input)?;
        let height = tiles.len();
        let width = tiles.first().unwrap().len();
        let mut grid = HashMap::<Point, Tile>::new();
        for (y, row) in tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                let point = Point {
                    x: x as isize,
                    y: y as isize,
                };
                let tile = match tile {
                    '.' => Tile::Empty,
                    '|' => Tile::Vertical,
                    '-' => Tile::Horizontal,
                    '/' => Tile::Slash,
                    '\\' => Tile::BackSlash,
                    _ => unreachable!(),
                };
                grid.insert(point, tile);
            }
        }
        Ok((
            "",
            Grid {
                grid,
                width,
                height,
            },
        ))
    }

    type Output1 = usize;

    fn part_1(input: &Self::Input) -> Self::Output1 {
        let beams = get_beams(input, Point { x: 0, y: 0 }, Dir::Right);
        beams.len()
    }

    type Output2 = usize;

    #[allow(clippy::cast_possible_wrap)]
    fn part_2(input: &Self::Input) -> Self::Output2 {
        let mut energized = Vec::<usize>::new();
        for y in 0..input.height {
            energized.push(
                get_beams(
                    input,
                    Point {
                        x: 0,
                        y: y as isize,
                    },
                    Dir::Right,
                )
                .len(),
            );
            energized.push(
                get_beams(
                    input,
                    Point {
                        x: (input.width as isize) - 1,
                        y: y as isize,
                    },
                    Dir::Left,
                )
                .len(),
            );
        }
        for x in 0..input.width {
            energized.push(
                get_beams(
                    input,
                    Point {
                        x: x as isize,
                        y: 0,
                    },
                    Dir::Bottom,
                )
                .len(),
            );
            energized.push(
                get_beams(
                    input,
                    Point {
                        x: x as isize,
                        y: (input.height as isize) - 1,
                    },
                    Dir::Top,
                )
                .len(),
            );
        }
        energized.iter().max().copied().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = ".|...\\....
|.-.\\.....
.....|-...
........|.
..........
.........\\
..../.\\\\..
.-.-/..|..
.|....-|.\\
..//.|....";

    #[test]
    fn test_part1() {
        let parsed = Day16::parse(INPUT).unwrap().1;
        assert_eq!(Day16::part_1(&parsed), 46);
    }

    /*     #[test]
    fn test_part2() {
        let parsed = Day15::parse(INPUT).unwrap().1;
        assert_eq!(Day15::part_2(&parsed), 145);
    } */
}
