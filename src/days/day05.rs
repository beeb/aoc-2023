use std::{collections::VecDeque, ops::Range};

use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::{char, line_ending, not_line_ending, space1, u64},
    combinator::map,
    multi::{separated_list0, separated_list1},
    sequence::{preceded, separated_pair},
    IResult,
};

use crate::days::Day;

pub struct Day05;

#[derive(Debug, Clone)]
pub struct Almanac {
    pub seeds: Vec<u64>,
    tables: Vec<MappingTable>,
}

#[derive(Debug, Clone)]
pub struct MappingTable {
    pub mappings: Vec<Mapping>,
}

#[derive(Debug, Clone)]
pub struct Mapping {
    pub source: Range<u64>,
    pub dest: Range<u64>,
}

impl Almanac {
    fn location(&self, seed: u64) -> u64 {
        self.tables.iter().fold(seed, |acc, table| table.dest(acc))
    }

    fn seed_ranges(&self) -> Vec<Range<u64>> {
        self.seeds
            .as_slice()
            .chunks(2)
            .filter_map(|i| {
                i.iter()
                    .collect_tuple()
                    .map(|(start, len)| *start..*start + *len)
            })
            .collect()
    }
}

impl MappingTable {
    fn dest(&self, source: u64) -> u64 {
        self.mappings
            .iter()
            .find(|&mapping| mapping.source.contains(&source))
            .map_or(source, |mapping| {
                let offset = source - mapping.source.start;
                mapping.dest.start + offset
            })
    }
}

impl From<Vec<Mapping>> for MappingTable {
    fn from(mappings: Vec<Mapping>) -> Self {
        Self { mappings }
    }
}

/// Find the overlap between two ranges
fn range_overlap(first: &Range<u64>, second: &Range<u64>) -> Option<Range<u64>> {
    if first.end >= second.start && first.start <= second.end {
        // some overlap
        if first.start < second.start {
            // either first is left or first fully contains second
            if first.end > second.end {
                // first fully contains second
                Some(second.clone())
            } else {
                // first is left
                Some(second.start..first.end)
            }
        } else if second.end > first.end {
            // second fully contains first
            Some(first.clone())
        } else {
            // second is left
            Some(first.start..second.end)
        }
    } else {
        // no overlap
        None
    }
}

/// Find all input mappings that overlap with the desired output mapping
fn compatible_mappings(input_mappings: &[Mapping], output_mapping: &Mapping) -> Vec<Mapping> {
    input_mappings
        .iter()
        .filter_map(|m| {
            let Some(overlap) = range_overlap(&output_mapping.source, &m.dest) else {
                return None;
            };
            let offset = overlap.start - m.dest.start;
            let len = overlap.end - overlap.start;
            let source_start = m.source.start + offset;
            Some(Mapping {
                source: source_start..source_start + len,
                dest: overlap,
            })
        })
        .collect()
}

fn parse_seeds(input: &str) -> IResult<&str, Vec<u64>> {
    map(
        preceded(tag("seeds: "), separated_list0(space1, u64)),
        |seeds| seeds,
    )(input)
}

fn parse_mappings(input: &str) -> IResult<&str, Vec<Mapping>> {
    map(
        separated_pair(
            not_line_ending,
            line_ending,
            separated_list1(line_ending, separated_list1(char(' '), u64)),
        ),
        |(_, items)| {
            let mut mappings: VecDeque<Mapping> = items
                .iter()
                .map(|range_info| {
                    let source_start = range_info[1];
                    let dest_start = range_info[0];
                    let len = range_info[2];
                    Mapping {
                        source: source_start..source_start + len,
                        dest: dest_start..dest_start + len,
                    }
                })
                .sorted_by(|a, b| a.dest.start.cmp(&b.dest.start))
                .collect();
            // add first identity map range if it doesn't start at 0
            if let Some(first) = mappings.front() {
                if first.dest.start > 0 {
                    mappings.push_front(Mapping {
                        source: 0..first.dest.start,
                        dest: 0..first.dest.start,
                    });
                }
            }
            // add last identity map range up to u64::MAX
            if let Some(last) = mappings.back() {
                mappings.push_back(Mapping {
                    source: last.dest.end..u64::MAX,
                    dest: last.dest.end..u64::MAX,
                });
            }
            // sort so that the range with the lowest output (dest) values comes first.
            mappings
                .into_iter()
                .sorted_by(|a, b| a.dest.start.cmp(&b.dest.start))
                .collect()
        },
    )(input)
}

impl Day for Day05 {
    type Input = Almanac;

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        map(
            separated_pair(
                parse_seeds,
                tag("\n\n"),
                separated_list0(tag("\n\n"), parse_mappings),
            ),
            |(seeds, mappings)| Almanac {
                seeds,
                tables: mappings.iter().map(|m| m.clone().into()).collect(),
            },
        )(input)
    }

    type Output1 = u64;

    /// Part 1 took 0.008237ms
    fn part_1(input: &Self::Input) -> Self::Output1 {
        input
            .seeds
            .iter()
            .map(|s| input.location(*s))
            .min()
            .unwrap()
    }

    type Output2 = u64;

    /// Part 2 took 195.56049ms
    fn part_2(input: &Self::Input) -> Self::Output2 {
        // last mappings table
        let output_table = input.tables.last().unwrap();

        // let's do a DFS to quickly find a path that connects outputs to inputs

        // stack for DFS, initialize with all the mapping ranges in the last table,
        // sorted by ascending dest.start (lower location comes first)
        let mut stack: VecDeque<(Mapping, usize)> = output_table
            .mappings
            .iter()
            .map(|m| (m.clone(), input.tables.len() - 1))
            .collect();

        while let Some((mapping, level)) = stack.pop_front() {
            if level == 0 {
                // we are at the seed-to-soil level, so let's check if there are compatible seed ranges
                let Some(seed_range) = input
                    .seed_ranges()
                    .iter()
                    .find_map(|seed_range| range_overlap(seed_range, &mapping.source))
                else {
                    // no compatible seed range, let's keep looking
                    continue;
                };
                // we have a matching seed range, so we're done, let's find the lowest location
                return seed_range.map(|s| input.location(s)).min().unwrap();
            }
            // find all compatible mappings in the previous table and add them at the front of the stack
            let input_table = &input.tables[level - 1];
            compatible_mappings(&input_table.mappings, &mapping)
                .into_iter()
                .for_each(|m| stack.push_front((m, level - 1)));
        }

        panic!("Couldn't find a suitable seed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = "seeds: 79 14 55 13

seed-to-soil map:
50 98 2
52 50 48

soil-to-fertilizer map:
0 15 37
37 52 2
39 0 15

fertilizer-to-water map:
49 53 8
0 11 42
42 0 7
57 7 4

water-to-light map:
88 18 7
18 25 70

light-to-temperature map:
45 77 23
81 45 19
68 64 13

temperature-to-humidity map:
0 69 1
1 0 69

humidity-to-location map:
60 56 37
56 93 4";

    #[test]
    fn test_part1() {
        let parsed = Day05::parse(INPUT).unwrap().1;
        assert_eq!(Day05::part_1(&parsed), 35);
    }

    #[test]
    fn test_part2() {
        let parsed = Day05::parse(INPUT).unwrap().1;
        assert_eq!(Day05::part_2(&parsed), 46);
    }
}
