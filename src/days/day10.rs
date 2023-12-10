use std::collections::{HashMap, HashSet};

use nom::{
    character::complete::{line_ending, not_line_ending},
    combinator::map,
    multi::separated_list0,
    IResult,
};
use owo_colors::{OwoColorize, Style};

use crate::days::Day;

#[derive(Debug, Eq, PartialEq)]
pub enum Dir {
    North,
    East,
    South,
    West,
    NorthEast,
    SouthEast,
    SouthWest,
    NorthWest,
}

const DIRS: [Dir; 4] = [Dir::North, Dir::East, Dir::South, Dir::West];
const DIAG_DIRS: [Dir; 4] = [
    Dir::NorthEast,
    Dir::SouthEast,
    Dir::SouthWest,
    Dir::NorthWest,
];

pub struct Day10;

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct Point {
    pub x: isize,
    pub y: isize,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Default)]
pub struct Pipe {
    pub north: bool,
    pub east: bool,
    pub south: bool,
    pub west: bool,
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
            Dir::NorthEast => Self {
                x: self.x + 1,
                y: self.y - 1,
            },
            Dir::SouthEast => Self {
                x: self.x + 1,
                y: self.y + 1,
            },
            Dir::SouthWest => Self {
                x: self.x - 1,
                y: self.y + 1,
            },
            Dir::NorthWest => Self {
                x: self.x - 1,
                y: self.y - 1,
            },
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
        let Some(this) = grid.get(&pos) else {
            unreachable!();
        };
        for dir in DIRS {
            // only look in directions where the pipe is connected to
            match (&dir, this) {
                (Dir::North, Tile::Pipe(t)) if !t.north => {
                    continue;
                }
                (Dir::East, Tile::Pipe(t)) if !t.east => {
                    continue;
                }
                (Dir::South, Tile::Pipe(t)) if !t.south => {
                    continue;
                }
                (Dir::West, Tile::Pipe(t)) if !t.west => {
                    continue;
                }
                _ => {}
            }
            let next_pos = pos.at_dir(&dir);
            let Some(next) = grid.get(&next_pos) else {
                continue;
            };
            // tile at the north position must be connected on the south side, etc.
            match (dir, next) {
                (Dir::North, Tile::Pipe(t)) if t.south => {
                    if pipes.contains(&next_pos) {
                        continue;
                    }
                    pipes.insert(next_pos.clone());
                    pos = next_pos;
                    break;
                }
                (Dir::East, Tile::Pipe(t)) if t.west => {
                    if pipes.contains(&next_pos) {
                        continue;
                    }
                    pipes.insert(next_pos.clone());
                    pos = next_pos;
                    break;
                }
                (Dir::South, Tile::Pipe(t)) if t.north => {
                    if pipes.contains(&next_pos) {
                        continue;
                    }
                    pipes.insert(next_pos.clone());
                    pos = next_pos;
                    break;
                }
                (Dir::West, Tile::Pipe(t)) if t.east => {
                    if pipes.contains(&next_pos) {
                        continue;
                    }
                    pipes.insert(next_pos.clone());
                    pos = next_pos;
                    break;
                }
                (_, Tile::Start) => {
                    // Avoid early exit if we re-visit the start tile immediately after starting to look
                    if pipes.len() < 3 {
                        continue;
                    }
                    // We went around the loop
                    break 'outer;
                }
                _ => {}
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
                        '|' => Tile::Pipe(Pipe {
                            north: true,
                            east: false,
                            south: true,
                            west: false,
                        }),
                        '-' => Tile::Pipe(Pipe {
                            north: false,
                            east: true,
                            south: false,
                            west: true,
                        }),
                        'L' => Tile::Pipe(Pipe {
                            north: true,
                            east: true,
                            south: false,
                            west: false,
                        }),
                        'J' => Tile::Pipe(Pipe {
                            north: true,
                            east: false,
                            south: false,
                            west: true,
                        }),
                        '7' => Tile::Pipe(Pipe {
                            north: false,
                            east: false,
                            south: true,
                            west: true,
                        }),
                        'F' => Tile::Pipe(Pipe {
                            north: false,
                            east: true,
                            south: true,
                            west: false,
                        }),
                        '.' => Tile::Ground,
                        'S' => Tile::Start,
                        _ => unimplemented!(),
                    })
                    .collect()
            }),
        )(input)
    }

    type Output1 = usize;

    #[allow(clippy::cast_possible_wrap, clippy::too_many_lines)]
    fn part_1(input: &Self::Input) -> Self::Output1 {
        let grid = get_grid_hashmap(input);
        let pipes = get_loop_positions(&grid);
        pipes.len() / 2
    }

    type Output2 = usize;

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
        let mut start_pipe = Pipe::default();
        for dir in DIRS {
            let Some(pipe_pos) = pipes.get(&start_pos.at_dir(&dir)) else {
                continue;
            };
            let Some(Tile::Pipe(neighbor)) = grid.get(pipe_pos) else {
                unreachable!("this tile should exist and be a pipe");
            };
            // this is one of the pipes connected to start
            if dir == Dir::North && neighbor.south {
                start_pipe.north = true;
            } else if dir == Dir::East && neighbor.west {
                start_pipe.east = true;
            } else if dir == Dir::South && neighbor.north {
                start_pipe.south = true;
            } else if dir == Dir::West && neighbor.east {
                start_pipe.west = true;
            }
        }
        let start_tile = Tile::Pipe(start_pipe);
        grid.insert(start_pos.clone(), &start_tile);

        let size_y = input.len() as isize;
        let size_x = input.first().unwrap().len() as isize;

        let mut inside_count = 0;
        for y in 0..size_y {
            let mut inside: bool = false;
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
                    if pipe.north || pipe.south {
                        if pipe.north && pipe.south {
                            inside = !inside;
                            maybe_switch = None;
                        } else if let Some(last_vert) = maybe_switch {
                            if (last_vert == Dir::North && pipe.south)
                                || (last_vert == Dir::South && pipe.north)
                            {
                                inside = !inside;
                            }
                            maybe_switch = None;
                        } else {
                            maybe_switch = pipe
                                .south
                                .then_some(Dir::South)
                                .or(pipe.north.then_some(Dir::North));
                        }
                    }

                    let style = if inside {
                        Style::new().yellow()
                    } else {
                        Style::new().blue()
                    };

                    // draw
                    if pipe.north {
                        if pipe.east {
                            print!("{}", "└".style(style));
                        } else if pipe.south {
                            print!("{}", "|".style(style));
                        } else if pipe.west {
                            print!("{}", "┘".style(style));
                        }
                    } else if pipe.east {
                        if pipe.south {
                            print!("{}", "┌".style(style));
                        } else if pipe.west {
                            print!("{}", "-".style(style));
                        }
                    } else if pipe.south && pipe.west {
                        print!("{}", "┐".style(style));
                    }
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

        /* // flood the outside, starting in the corners

        let mut outside = HashSet::<Point>::new();
        outside.insert(Point { x: 0, y: 0 });
        outside.insert(Point {
            x: 0,
            y: size_y - 1,
        });
        outside.insert(Point {
            x: size_x - 1,
            y: 0,
        });
        outside.insert(Point {
            x: size_x - 1,
            y: size_y - 1,
        });

        // stack for BFS
        let mut stack = VecDeque::<Point>::new();
        stack.push_back(Point { x: 0, y: 0 });
        stack.push_back(Point {
            x: 0,
            y: size_y - 1,
        });
        stack.push_back(Point {
            x: size_x - 1,
            y: 0,
        });
        stack.push_back(Point {
            x: size_x - 1,
            y: size_y - 1,
        });
        while let Some(pos) = stack.pop_front() {
            let Some(tile) = grid.get(&pos) else {
                unreachable!("we should only add existing tiles to the stack");
            };
            for dir in DIRS {
                let next_pos = pos.at_dir(&dir);
                let Some(_) = grid.get(&next_pos) else {
                    continue;
                };
                if pipes.contains(&pos) {
                    // only look in the directions allowed by the pipe
                    let Tile::Pipe(pipe) = tile else {
                        unreachable!("pipe loop contains tile that is not a pipe?");
                    };
                    if (!pipe.north && dir == Dir::North)
                        || (!pipe.east && dir == Dir::East)
                        || (!pipe.south && dir == Dir::South)
                        || (!pipe.west && dir == Dir::West)
                    {
                        continue;
                    }
                }

                if outside.contains(&next_pos) {
                    continue;
                }
                outside.insert(next_pos.clone());
                stack.push_back(next_pos.clone());
            }
            for dir in DIAG_DIRS {
                let next_pos = pos.at_dir(&dir);
                let Some(_) = grid.get(&next_pos) else {
                    continue;
                };
                if outside.contains(&next_pos) {
                    continue;
                }
                if !pipes.contains(&pos) && !pipes.contains(&next_pos) {
                    outside.insert(next_pos.clone());
                    stack.push_back(next_pos.clone());
                }
            }
        }

        for y in 0..size_y {
            for x in 0..size_x {
                let pos = Point { x, y };
                if outside.contains(&pos) {
                    if pipes.contains(&pos) {
                        let Some(Tile::Pipe(pipe)) = grid.get(&pos) else {
                            unreachable!();
                        };
                        if pipe.north {
                            if pipe.east {
                                print!("└");
                            } else if pipe.south {
                                print!("|");
                            } else if pipe.west {
                                print!("┘");
                            }
                        } else if pipe.east {
                            if pipe.south {
                                print!("┌");
                            } else if pipe.west {
                                print!("-");
                            }
                        } else if pipe.south && pipe.west {
                            print!("┐");
                        }
                    } else {
                        print!("o");
                    }
                } else {
                    print!("i");
                }
            }
            println!();
        }
        (size_y * size_x) as usize - outside.len() */
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
