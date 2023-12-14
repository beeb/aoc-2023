use std::collections::{BTreeMap, HashMap};

use itertools::Itertools;
use nom::{
    character::complete::{line_ending, one_of},
    multi::{many1, separated_list0},
    IResult,
};

use crate::days::Day;

pub struct Day14;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Point {
    y: isize, // btreemap is sorted by key, so y is first
    x: isize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Rock {
    Round,
    Cube,
}

#[derive(Debug, Clone)]
pub struct Platform {
    pub grid: BTreeMap<Point, Rock>,
    pub width: isize,
    pub height: isize,
}

#[derive(Debug, Clone, Copy)]
pub enum Dir {
    North,
    East,
    South,
    West,
}

impl Platform {
    fn has_rock(&self, point: &Point) -> Option<&Rock> {
        self.grid.get(point)
    }

    fn pos_load(&self, point: &Point, dir: Dir) -> isize {
        match dir {
            Dir::North => self.height - point.y,
            Dir::East => point.x + 1,
            Dir::South => point.y + 1,
            Dir::West => self.width - point.x,
        }
    }

    fn next_in_dir(&self, start: &Point, dir: Dir) -> Point {
        let pos = match dir {
            Dir::North => (0..start.y)
                .rev()
                .map(|y| Point { y, x: start.x })
                .collect_vec(),
            Dir::East => ((start.x + 1)..self.width)
                .map(|x| Point { y: start.y, x })
                .collect_vec(),
            Dir::South => ((start.y + 1)..self.height)
                .map(|y| Point { y, x: start.x })
                .collect_vec(),
            Dir::West => (0..start.x)
                .rev()
                .map(|x| Point { y: start.y, x })
                .collect_vec(),
        };
        let empty_space = pos
            .into_iter()
            .take_while(|p| self.has_rock(p).is_none())
            .collect_vec();
        empty_space.last().cloned().unwrap_or(start.clone())
    }

    fn move_rocks(&mut self, dir: Dir) {
        let rounds = self
            .grid
            .iter()
            .filter(|(_, r)| matches!(r, Rock::Round))
            .sorted_by(|(a, _), (b, _)| match dir {
                Dir::North => a.y.cmp(&b.y),
                Dir::East => b.x.cmp(&a.x),
                Dir::South => b.y.cmp(&a.y),
                Dir::West => a.x.cmp(&b.x),
            })
            .map(|(p, r)| (p.clone(), *r))
            .collect_vec();
        for (p, _) in rounds {
            let new_p = self.next_in_dir(&p, dir);
            let r = self.grid.remove(&p).unwrap();
            self.grid.insert(new_p, r);
        }
    }

    fn total_load(&self, dir: Dir) -> isize {
        self.grid
            .iter()
            .filter_map(|(p, r)| match r {
                Rock::Round => Some(self.pos_load(p, dir)),
                Rock::Cube => None,
            })
            .sum()
    }
}

impl Day for Day14 {
    type Input = Platform;

    #[allow(clippy::cast_possible_wrap)]
    fn parse(input: &str) -> IResult<&str, Self::Input> {
        let (rest, elements) = separated_list0(line_ending, many1(one_of(".#O")))(input)?;
        let mut grid = BTreeMap::new();
        for (y, row) in elements.iter().enumerate() {
            for (x, elem) in row.iter().enumerate() {
                let point = Point {
                    y: y as isize,
                    x: x as isize,
                };
                match elem {
                    '.' => {}
                    '#' => {
                        grid.insert(point, Rock::Cube);
                    }
                    'O' => {
                        grid.insert(point, Rock::Round);
                    }
                    _ => unreachable!(),
                }
            }
        }
        let width = elements.first().unwrap().len() as isize;
        let height = elements.len() as isize;
        Ok((
            rest,
            Platform {
                grid,
                width,
                height,
            },
        ))
    }

    type Output1 = isize;

    /// Part 1 took 784.744µs
    fn part_1(input: &Self::Input) -> Self::Output1 {
        let mut platform = input.clone(); // get mutable copy
        platform.move_rocks(Dir::North);
        platform.total_load(Dir::North)
    }

    type Output2 = isize;

    /// Part 2 took 436.819662ms
    fn part_2(input: &Self::Input) -> Self::Output2 {
        let mut platform = input.clone(); // get mutable copy

        // There is probably a repeating pattern, where at some point the rocks would return to the same pattern every
        // N cycles.
        // Let's keep a cache of the platform state at each iteration.
        let mut cache = HashMap::<BTreeMap<Point, Rock>, usize>::new();
        for i in 0.. {
            platform.move_rocks(Dir::North);
            platform.move_rocks(Dir::West);
            platform.move_rocks(Dir::South);
            platform.move_rocks(Dir::East);
            if let Some(j) = cache.get(&platform.grid) {
                // We have seen the current configuration before!
                let modulo = i - j; // The periodicity
                let target = (1_000_000_000 - 1) % modulo;

                // Find the last one in the cache that matches the predicate i % modulo == 1B % modulo
                let elem = cache
                    .iter()
                    .sorted_by(|(_, a), (_, &b)| b.cmp(a))
                    .find(|(_, &ii)| ii % modulo == target)
                    .unwrap();
                platform.grid = elem.0.clone(); // restore the state
                break;
            }
            cache.insert(platform.grid.clone(), i);
        }
        // Check finally the north support load
        platform.total_load(Dir::North)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = "O....#....
O.OO#....#
.....##...
OO.#O....O
.O.....O#.
O.#..O.#.#
..O..#O..O
.......O..
#....###..
#OO..#....";

    #[test]
    fn test_part1() {
        let parsed = Day14::parse(INPUT).unwrap().1;
        assert_eq!(Day14::part_1(&parsed), 136);
    }

    #[test]
    fn test_part2() {
        let parsed = Day14::parse(INPUT).unwrap().1;
        assert_eq!(Day14::part_2(&parsed), 64);
    }
}
