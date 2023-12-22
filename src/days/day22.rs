use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

use itertools::Itertools;
use nom::{
    character::complete::{char, line_ending, u64},
    combinator::map,
    multi::separated_list0,
    sequence::{separated_pair, tuple},
    IResult,
};
use petgraph::prelude::*;

use crate::days::Day;

pub struct Day22;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Voxel {
    pub z: usize,
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum Dir {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Brick {
    pub begin: Voxel,
    pub end: Voxel,
}

impl Brick {
    /// Since bricks are a single row of blocks, we can define a main direction
    fn dir(&self) -> Dir {
        if self.begin.x != self.end.x {
            Dir::X
        } else if self.begin.y != self.end.y {
            Dir::Y
        } else {
            Dir::Z
        }
    }
}

/// Iterator over the blocks of a brick
pub struct BrickIntoIterator {
    brick: Brick,
    dir: Dir,
    current: Voxel,
}

impl IntoIterator for Brick {
    type Item = Voxel;

    type IntoIter = BrickIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        BrickIntoIterator {
            brick: self,
            dir: self.dir(),
            current: self.begin,
        }
    }
}

impl Iterator for BrickIntoIterator {
    type Item = Voxel;

    /// Get the next voxel (block) in a brick iterator
    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.current;
        if curr > self.brick.end {
            return None;
        }
        let next = match self.dir {
            Dir::X => Voxel {
                x: self.current.x + 1,
                y: self.current.y,
                z: self.current.z,
            },
            Dir::Y => Voxel {
                x: self.current.x,
                y: self.current.y + 1,
                z: self.current.z,
            },
            Dir::Z => Voxel {
                x: self.current.x,
                y: self.current.y,
                z: self.current.z + 1,
            },
        };
        self.current = next;
        Some(curr)
    }
}

#[allow(clippy::cast_possible_truncation)]
fn parse_voxel(input: &str) -> IResult<&str, Voxel> {
    let (rest, (x, _, y, _, z)) = tuple((u64, char(','), u64, char(','), u64))(input)?;
    Ok((
        rest,
        Voxel {
            x: x as usize,
            y: y as usize,
            z: z as usize,
        },
    ))
}

// fn print_bricks(grid: &BTreeSet<Voxel>) {
//     let max_z = grid.iter().map(|v| v.z).max().unwrap();
//     let max_x = grid.iter().map(|v| v.x).max().unwrap();
//     let max_y = grid.iter().map(|v| v.y).max().unwrap();
//     for z in (1..=max_z).rev() {
//         for x in 0..=max_x {
//             let count = grid.iter().filter(|v| v.x == x && v.z == z).count();
//             if count > 0 {
//                 print!("{count}");
//             } else {
//                 print!(".");
//             }
//         }
//         print!("   ");
//         for y in 0..=max_y {
//             let count = grid.iter().filter(|v| v.y == y && v.z == z).count();
//             if count > 0 {
//                 print!("{count}");
//             } else {
//                 print!(".");
//             }
//         }
//         println!();
//     }
//     println!("-----------------");
// }

/// Make the bricks fall
fn settle(bricks: &mut [Brick], grid: &mut BTreeSet<Voxel>) {
    for brick in bricks.iter_mut() {
        // Check how far we can move down on the Z axis before reaching an obstacle
        // We start at an offset of 1 and continue until an obstacle is reached
        let move_z = (1..1000)
            .take_while(|i| match brick.dir() {
                // for a vertical brick, we only need to consider the "begin" voxel
                Dir::Z => {
                    let Some(new_z) = brick.begin.z.checked_sub(*i) else {
                        return false;
                    };
                    if new_z == 0 {
                        return false;
                    }
                    !grid.contains(&Voxel {
                        x: brick.begin.x,
                        y: brick.begin.y,
                        z: new_z,
                    })
                }
                // for horizontal bricks, we need to check how far can each block go
                Dir::X | Dir::Y => brick.into_iter().all(|v| {
                    let Some(new_z) = v.z.checked_sub(*i) else {
                        return false;
                    };
                    if new_z == 0 {
                        return false;
                    }
                    !grid.contains(&Voxel {
                        x: v.x,
                        y: v.y,
                        z: v.z - i,
                    })
                }),
            })
            .count();
        // shift the brick down by the calculated amount, reflecting the changes in the global grid
        for voxel in brick.into_iter() {
            grid.remove(&voxel);
        }
        brick.begin.z -= move_z;
        brick.end.z -= move_z;
        grid.extend(brick.into_iter());
    }
    // Sort the bricks so we can still iterate from low-Z to high-Z
    bricks.sort();
}

/// Create a graph where the nodes are bricks, and the edges represent "support". If a brick has contact to a brick
/// one layer up, then a directed edge joins them (from bottom brick to to brick).
fn get_graph<'a>(
    bricks: &'a [Brick],
    grid: &'a BTreeSet<Voxel>,
) -> (Graph<&'a Brick, ()>, HashMap<&'a Brick, NodeIndex>) {
    let mut graph = Graph::<&Brick, ()>::new();
    let mut node_indices = HashMap::<&Brick, NodeIndex>::new();
    // add all brick references to the graph as nodes
    for brick in bricks {
        let idx = graph.add_node(brick);
        node_indices.insert(brick, idx);
    }
    // for each brick, check the blocks above, identify which brick they belong to, and create the edges
    for brick in bricks {
        match brick.dir() {
            // for vertical bricks, we only need to consider the "end" block
            Dir::Z => {
                // check if the block above is populated
                let above = Voxel {
                    x: brick.end.x,
                    y: brick.end.y,
                    z: brick.end.z + 1,
                };
                if grid.contains(&above) {
                    // find which brick it belongs to
                    let other = bricks
                        .iter()
                        .skip_while(|b| b.begin.z < above.z)
                        .find(|b| b.into_iter().any(|v| v.x == above.x && v.y == above.y))
                        .unwrap();

                    graph.add_edge(
                        *node_indices.get(brick).unwrap(),
                        *node_indices.get(other).unwrap(),
                        (),
                    );
                }
            }
            // for horizontal bricks, we consider each block
            Dir::X | Dir::Y => {
                // check if the blocks above are populated
                let voxels = brick.into_iter().collect_vec();
                for voxel in &voxels {
                    let above = Voxel {
                        x: voxel.x,
                        y: voxel.y,
                        z: voxel.z + 1,
                    };
                    if grid.contains(&above) {
                        // find which brick it belongs to
                        let other = bricks
                            .iter()
                            .skip_while(|b| b.begin.z < above.z)
                            .find(|b| b.into_iter().any(|v| v.x == above.x && v.y == above.y))
                            .unwrap();
                        let a = *node_indices.get(brick).unwrap();
                        let b = *node_indices.get(other).unwrap();
                        if !graph.contains_edge(a, b) {
                            // only add an edge if one is not already existing
                            graph.add_edge(a, b, ());
                        }
                    }
                }
            }
        }
    }
    (graph, node_indices)
}

impl Day for Day22 {
    type Input = Vec<Brick>;

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        separated_list0(
            line_ending,
            map(
                separated_pair(parse_voxel, char('~'), parse_voxel),
                |(begin, end)| Brick { begin, end },
            ),
        )(input)
    }

    type Output1 = usize;

    /// Part 1 took 7.5043ms
    fn part_1(input: &Self::Input) -> Self::Output1 {
        let mut bricks = input.iter().sorted().copied().collect_vec();
        let mut grid = BTreeSet::<Voxel>::new();
        for brick in &bricks {
            grid.extend(brick.into_iter());
        }

        settle(&mut bricks, &mut grid);
        let (supports, node_indices) = get_graph(&bricks, &grid);

        // println!("{:?}", Dot::with_config(&supports, &[Config::EdgeNoLabel]));

        // Check which bricks only have children with more than 1 parent (i.e. they would not move if removed)
        node_indices
            .iter()
            .filter(|(_, &brick_idx)| {
                supports
                    .neighbors_directed(brick_idx, Direction::Outgoing)
                    .all(|child| {
                        let parents = supports.edges_directed(child, Direction::Incoming).count();
                        parents > 1
                    })
            })
            .count()
    }

    type Output2 = usize;

    /// Part 2 took 15.0054ms
    fn part_2(input: &Self::Input) -> Self::Output2 {
        let mut bricks = input.iter().sorted().copied().collect_vec();
        let mut grid = BTreeSet::<Voxel>::new();
        for brick in &bricks {
            grid.extend(brick.into_iter());
        }
        settle(&mut bricks, &mut grid);
        let (supports, node_indices) = get_graph(&bricks, &grid);

        // println!("{:?}", Dot::with_config(&supports, &[Config::EdgeNoLabel]));

        let mut total = 0;

        // check how many bricks would fall for each brick that we would remove
        for (_, brick_idx) in node_indices {
            // BFS to visit all nodes starting at the considered brick
            let mut falling = HashSet::<NodeIndex>::new();
            let mut stack = VecDeque::<NodeIndex>::new();
            stack.push_back(brick_idx);
            while let Some(nx) = stack.pop_front() {
                if !falling.insert(nx) {
                    continue; // was already visited
                }
                // consider all of the children
                for n in supports.neighbors_directed(nx, Direction::Outgoing) {
                    // for this child, check if its parents would fall
                    if !supports
                        .neighbors_directed(n, Direction::Incoming)
                        .all(|i| falling.contains(&i))
                    {
                        // one or more of its parents would not fall, so this child would not fall either
                        continue;
                    }
                    // this child would fall too, let's mark it
                    stack.push_back(n);
                }
            }
            // since the start node (brick that we are considering) should not be counted, we subtract one
            total += falling.len() - 1;
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const INPUT: &str = "1,0,1~1,2,1
0,0,2~2,0,2
0,2,3~2,2,3
0,0,4~0,2,4
2,0,5~2,2,5
0,1,6~2,1,6
1,1,8~1,1,9";

    #[test]
    fn test_iter() {
        let brick = Brick {
            begin: Voxel { x: 2, y: 0, z: 5 },
            end: Voxel { x: 2, y: 2, z: 5 },
        };
        let mut iter = brick.into_iter();
        assert_eq!(iter.next(), Some(Voxel { x: 2, y: 0, z: 5 }));
        assert_eq!(iter.next(), Some(Voxel { x: 2, y: 1, z: 5 }));
        assert_eq!(iter.next(), Some(Voxel { x: 2, y: 2, z: 5 }));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_part1() {
        let parsed = Day22::parse(INPUT).unwrap().1;
        assert_eq!(Day22::part_1(&parsed), 5);
    }
    #[test]
    fn test_part2() {
        let parsed = Day22::parse(INPUT).unwrap().1;
        assert_eq!(Day22::part_2(&parsed), 7);
    }
}
