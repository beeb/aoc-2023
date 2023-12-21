use itertools::Itertools;
use nom::{
    character::complete::{line_ending, not_line_ending},
    multi::separated_list0,
    IResult,
};
use pathfinding::{directed::bfs::bfs_reach, grid::Grid};

use crate::days::Day;

pub struct Day21;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Pos {
    coord: (usize, usize),
    dist: usize,
}

impl Day for Day21 {
    type Input = (Grid, (usize, usize));

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        let (rest, rows) = separated_list0(line_ending, not_line_ending)(input)?;
        let mut start_x = 0;
        let mut start_y = 0;
        let mut rocks = vec![];
        for (y, row) in rows.into_iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                match c {
                    '#' => rocks.push((x, y)),
                    'S' => {
                        start_x = x;
                        start_y = y;
                    }
                    _ => {}
                }
            }
        }
        let mut grid = rocks.into_iter().collect::<Grid>();
        grid.invert();
        Ok((rest, (grid, (start_x, start_y))))
    }

    type Output1 = usize;

    fn part_1(input: &Self::Input) -> Self::Output1 {
        let (grid, start) = input;
        let start_pos = Pos {
            coord: *start,
            dist: 0,
        };
        bfs_reach(start_pos, |&n| {
            grid.neighbours(n.coord)
                .into_iter()
                .filter_map(|c| {
                    if n.dist >= 64 {
                        return None;
                    }
                    Some(Pos {
                        coord: c,
                        dist: n.dist + 1,
                    })
                })
                .collect_vec()
        })
        .filter(|n| n.dist == 64)
        .count()
    }

    type Output2 = usize;

    fn part_2(_input: &Self::Input) -> Self::Output2 {
        unimplemented!("part_2")
    }
}
