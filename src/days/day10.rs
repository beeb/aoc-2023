use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use nom::{
    character::complete::{line_ending, not_line_ending},
    combinator::map,
    multi::separated_list0,
    IResult,
};
use owo_colors::{OwoColorize, Style};

use crate::days::Day;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Dir {
    North,
    East,
    South,
    West,
}

const DIRS: [Dir; 4] = [Dir::North, Dir::East, Dir::South, Dir::West];

pub struct Day10;

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct Point {
    pub x: isize,
    pub y: isize,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Pipe {
    NorthEast,
    NorthSouth,
    NorthWest,
    EastSouth,
    EastWest,
    SouthWest,
}

#[derive(Debug)]
pub enum Tile {
    Pipe(Pipe),
    Ground,
    Start,
}

impl Point {
    fn at_dir(&self, dir: &Dir) -> Self {
        match dir {
            Dir::North => Self {
                x: self.x,
                y: self.y - 1,
            },
            Dir::East => Self {
                x: self.x + 1,
                y: self.y,
            },
            Dir::South => Self {
                x: self.x,
                y: self.y + 1,
            },
            Dir::West => Self {
                x: self.x - 1,
                y: self.y,
            },
        }
    }
}

impl Pipe {
    fn north(&self) -> bool {
        matches!(self, Pipe::NorthEast | Pipe::NorthSouth | Pipe::NorthWest)
    }
    fn east(&self) -> bool {
        matches!(self, Pipe::NorthEast | Pipe::EastSouth | Pipe::EastWest)
    }
    fn south(&self) -> bool {
        matches!(self, Pipe::NorthSouth | Pipe::EastSouth | Pipe::SouthWest)
    }
    fn west(&self) -> bool {
        matches!(self, Pipe::NorthWest | Pipe::EastWest | Pipe::SouthWest)
    }
    fn from_dirs(dir1: &Dir, dir2: &Dir) -> Self {
        match (dir1, dir2) {
            (Dir::North, Dir::East) => Pipe::NorthEast,
            (Dir::North, Dir::South) => Pipe::NorthSouth,
            (Dir::North, Dir::West) => Pipe::NorthWest,
            (Dir::East, Dir::South) => Pipe::EastSouth,
            (Dir::East, Dir::West) => Pipe::EastWest,
            (Dir::South, Dir::West) => Pipe::SouthWest,
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Display for Pipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pipe::NorthEast => write!(f, "└"),
            Pipe::NorthSouth => write!(f, "|"),
            Pipe::NorthWest => write!(f, "┘"),
            Pipe::EastSouth => write!(f, "┌"),
            Pipe::EastWest => write!(f, "—"),
            Pipe::SouthWest => write!(f, "┐"),
        }
    }
}

#[allow(clippy::cast_possible_wrap)]
fn get_grid_hashmap(grid: &[Vec<Tile>]) -> HashMap<Point, &Tile> {
    grid.iter()
        .enumerate()
        .flat_map(move |(y, row)| {
            row.iter().enumerate().map(move |(x, tile)| {
                (
                    Point {
                        x: x as isize,
                        y: y as isize,
                    },
                    tile,
                )
            })
        })
        .collect()
}

fn get_loop_positions(grid: &HashMap<Point, &Tile>) -> HashSet<Point> {
    let mut pos: Point = grid
        .iter()
        .find(|(_, &t)| matches!(t, Tile::Start))
        .unwrap()
        .0
        .clone();

    let mut pipes: HashSet<Point> = HashSet::with_capacity(1000);
    pipes.insert(pos.clone());
    'outer: loop {
        let Some(tile) = grid.get(&pos) else {
            unreachable!();
        };
        for dir in DIRS {
            // only look in directions where the pipe is connected to
            if let Tile::Pipe(pipe) = tile {
                if (!pipe.north() && dir == Dir::North)
                    || (!pipe.east() && dir == Dir::East)
                    || (!pipe.south() && dir == Dir::South)
                    || (!pipe.west() && dir == Dir::West)
                {
                    continue;
                }
            }

            let next_pos = pos.at_dir(&dir);
            let Some(next) = grid.get(&next_pos) else {
                continue;
            };
            if let Tile::Pipe(pipe) = next {
                // Tile at the north position must be connected on the south side, etc
                if (dir == Dir::North && pipe.south())
                    || (dir == Dir::East && pipe.west())
                    || (dir == Dir::South && pipe.north())
                    || (dir == Dir::West && pipe.east())
                {
                    if pipes.contains(&next_pos) {
                        continue;
                    }
                    pipes.insert(next_pos.clone());
                    pos = next_pos;
                    break;
                }
            }
            if let Tile::Start = next {
                // Avoid early exit if we re-visit the start tile immediately after starting to look
                if pipes.len() < 3 {
                    continue;
                }
                // We went around the loop
                break 'outer;
            }
        }
    }
    pipes
}

impl Day for Day10 {
    type Input = Vec<Vec<Tile>>;

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        separated_list0(
            line_ending,
            map(not_line_ending, |s: &str| {
                s.chars()
                    .map(|c| match c {
                        '|' => Tile::Pipe(Pipe::NorthSouth),
                        '-' => Tile::Pipe(Pipe::EastWest),
                        'L' => Tile::Pipe(Pipe::NorthEast),
                        'J' => Tile::Pipe(Pipe::NorthWest),
                        '7' => Tile::Pipe(Pipe::SouthWest),
                        'F' => Tile::Pipe(Pipe::EastSouth),
                        '.' => Tile::Ground,
                        'S' => Tile::Start,
                        _ => unimplemented!(),
                    })
                    .collect()
            }),
        )(input)
    }

    type Output1 = usize;

    /// Part 1 took 2.935601ms
    #[allow(clippy::cast_possible_wrap, clippy::too_many_lines)]
    fn part_1(input: &Self::Input) -> Self::Output1 {
        let grid = get_grid_hashmap(input);
        let pipes = get_loop_positions(&grid);
        pipes.len() / 2
    }

    type Output2 = usize;

    /// Part 2 took 3.7294ms (without drawing)
    #[allow(
        clippy::too_many_lines,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss
    )]
    fn part_2(input: &Self::Input) -> Self::Output2 {
        let mut grid = get_grid_hashmap(input);
        let pipes = get_loop_positions(&grid);

        // replace starting tile by the corresponding pipe
        let (start_pos, _) = grid
            .iter()
            .find(|(_, &t)| matches!(t, Tile::Start))
            .unwrap();
        // find the two directions the starting pipe is connected to
        let (dir1, dir2): (&Dir, &Dir) = DIRS
            .iter()
            .filter(|dir| {
                let pos = start_pos.at_dir(dir);
                match grid.get(&pos) {
                    Some(Tile::Pipe(pipe)) => match (dir, pipe) {
                        (Dir::North, p) if p.south() => true,
                        (Dir::East, p) if p.west() => true,
                        (Dir::South, p) if p.north() => true,
                        (Dir::West, p) if p.east() => true,
                        _ => false,
                    },
                    _ => false,
                }
            })
            .sorted()
            .collect_tuple()
            .unwrap();
        let start_pipe = Pipe::from_dirs(dir1, dir2);
        let start_tile = Tile::Pipe(start_pipe);
        grid.insert(start_pos.clone(), &start_tile);

        let size_y = input.len() as isize;
        let size_x = input.first().unwrap().len() as isize;

        // iterate through the grid and keep track of whether we are inside or outside the loop
        // We have to switch from inside to outside or vice versa when we encounter a vertical pipe, or when a corner
        // pipe follows another corner pipe with the complementary vertical segment.
        // E.g. we are outside, we find a └ pipe, we are still outside, but if we then encounter a ┐ pipe later,
        // then we switch to inside. On the countrary, if we later encounter a ┘ pipe, then we stay outside.
        let mut inside_count = 0;
        for y in 0..size_y {
            let mut inside: bool = false; // we start at each row outside the loop
            let mut maybe_switch: Option<Dir> = None;
            for x in 0..size_x {
                let pos = Point { x, y };
                let Some(tile) = grid.get(&pos) else {
                    unreachable!("there are tiles in this range");
                };

                let style = if inside {
                    Style::new().yellow()
                } else {
                    Style::new().blue()
                };
                if pipes.contains(&pos) {
                    let Tile::Pipe(pipe) = tile else {
                        unreachable!("pipe loop contains tile that is not a pipe");
                    };
                    if pipe.north() || pipe.south() {
                        if pipe.north() && pipe.south() {
                            // fully vertical pipe, we switch
                            inside = !inside;
                            maybe_switch = None;
                        } else if let Some(last_vert) = maybe_switch {
                            // we had encountered a pipe with a vertical segment earlier
                            if (last_vert == Dir::North && pipe.south())
                                || (last_vert == Dir::South && pipe.north())
                            {
                                // if the current vertical segment is complementary, we switch
                                inside = !inside;
                            }
                            maybe_switch = None; // in all cases, we reset the pending state
                        } else {
                            // it's the first time we encounter a vertical pipe segment, let's register it in the
                            // pending state
                            maybe_switch = pipe
                                .south()
                                .then_some(Dir::South)
                                .or(pipe.north().then_some(Dir::North));
                        }
                    }

                    // drawing the grid an state of the right of the current tile (inside = yellow, outside = blue)
                    let style = if inside {
                        Style::new().yellow()
                    } else {
                        Style::new().blue()
                    };
                    print!("{}", pipe.style(style));
                } else if inside {
                    inside_count += 1;
                    print!("{}", "i".style(style));
                } else {
                    print!("{}", "o".style(style));
                }
            }
            println!();
        }
        inside_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        let input = "..F7.
.FJ|.
SJ.L7
|F--J
LJ...";
        let parsed = Day10::parse(input).unwrap().1;
        assert_eq!(Day10::part_1(&parsed), 8);
    }

    #[test]
    fn test_part2() {
        let input = ".F----7F7F7F7F-7....
.|F--7||||||||FJ....
.||.FJ||||||||L7....
FJL7L7LJLJ||LJ.L-7..
L--J.L7...LJS7F-7L7.
....F-J..F7FJ|L7L7L7
....L7.F7||L7|.L7L7|
.....|FJLJ|FJ|F7|.LJ
....FJL-7.||.||||...
....L---J.LJ.LJLJ...";

        let parsed = Day10::parse(input).unwrap().1;
        assert_eq!(Day10::part_2(&parsed), 8);
    }
}
