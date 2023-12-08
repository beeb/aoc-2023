use std::collections::HashMap;

use itertools::{FoldWhile, Itertools};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, char, line_ending},
    combinator::map,
    multi::{many1, separated_list0},
    sequence::{preceded, separated_pair, terminated},
    IResult,
};
use num::Integer;

use crate::days::Day;

pub struct Day08;

#[derive(Debug)]
pub enum Dir {
    Left,
    Right,
}

#[derive(Debug)]
pub struct Node {
    pub name: String,
    pub left: String,
    pub right: String,
}

/// Count how many steps from a start node until the end condition is met
///
/// For part 1, end condition is that the node is named 'ZZZ'. For part 2, any node that ends with 'Z'
fn count_steps(
    instructions: &[Dir],
    nodes: &HashMap<String, Node>,
    start_node: &Node,
    part2: bool,
) -> usize {
    let (count, _) = instructions
        .iter()
        .cycle()
        .enumerate()
        .fold_while((0, start_node), |(_, n), (i, instr)| {
            let next = match instr {
                Dir::Left => nodes.get(&n.left).unwrap(),
                Dir::Right => nodes.get(&n.right).unwrap(),
            };
            if (!part2 && next.name == "ZZZ") || (part2 && next.name.ends_with('Z')) {
                FoldWhile::Done((i + 1, next))
            } else {
                FoldWhile::Continue((i + 1, next))
            }
        })
        .into_inner();

    count
}

fn parse_instructions(input: &str) -> IResult<&str, Vec<Dir>> {
    many1(map(alt((char('L'), char('R'))), |c| match c {
        'L' => Dir::Left,
        'R' => Dir::Right,
        _ => unimplemented!(),
    }))(input)
}

fn parse_node(input: &str) -> IResult<&str, Node> {
    map(
        separated_pair(
            alphanumeric1::<&str, _>,
            tag(" = "),
            preceded(
                char('('),
                terminated(
                    separated_pair(alphanumeric1, tag(", "), alphanumeric1),
                    char(')'),
                ),
            ),
        ),
        |(name, (left, right))| Node {
            name: name.to_string(),
            left: left.to_string(),
            right: right.to_string(),
        },
    )(input)
}

impl Day for Day08 {
    type Input = (Vec<Dir>, HashMap<String, Node>);

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        map(
            separated_pair(
                parse_instructions,
                tag("\n\n"),
                separated_list0(line_ending, parse_node),
            ),
            |(instructions, nodes)| {
                (
                    instructions,
                    nodes.into_iter().map(|n| (n.name.clone(), n)).collect(),
                )
            },
        )(input)
    }

    type Output1 = usize;

    /// Part 1 took 407.9µs
    fn part_1(input: &Self::Input) -> Self::Output1 {
        // We stored all the nodes in a HashMap with the node name as the key
        let (instructions, nodes) = input;
        // Use the "AAA" node as a starting point
        count_steps(instructions, nodes, nodes.get("AAA").unwrap(), false)
    }

    type Output2 = usize;

    /// Part 2 took 2.4632ms
    fn part_2(input: &Self::Input) -> Self::Output2 {
        let (instructions, nodes) = input;
        // Find all the starting nodes (ending with 'A') and count how long until we reach and end node for each
        let counts: Vec<usize> = nodes
            .iter()
            .filter_map(|(k, v)| k.ends_with('A').then_some(v))
            .map(|n| count_steps(instructions, nodes, n, true))
            .collect();
        // Get the lowest common multiplier between all the counts
        counts.into_iter().reduce(|acc, e| acc.lcm(&e)).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        let input = "RL

AAA = (BBB, CCC)
BBB = (DDD, EEE)
CCC = (ZZZ, GGG)
DDD = (DDD, DDD)
EEE = (EEE, EEE)
GGG = (GGG, GGG)
ZZZ = (ZZZ, ZZZ)";

        let parsed = Day08::parse(input).unwrap().1;
        assert_eq!(Day08::part_1(&parsed), 2);
    }

    #[test]
    fn test_part2() {
        let input = "LR

11A = (11B, XXX)
11B = (XXX, 11Z)
11Z = (11B, XXX)
22A = (22B, XXX)
22B = (22C, 22C)
22C = (22Z, 22Z)
22Z = (22B, 22B)
XXX = (XXX, XXX)";

        let parsed = Day08::parse(input).unwrap().1;
        assert_eq!(Day08::part_2(&parsed), 6);
    }
}
