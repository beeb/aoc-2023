use std::{collections::VecDeque, ops::Range};

use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::{char, line_ending, not_line_ending, space1, u64},
    combinator::map,
    multi::{separated_list0, separated_list1},
    sequence::{separated_pair, tuple},
    IResult,
};

use crate::days::Day;

pub struct Day05;

#[derive(Debug, Clone)]
pub struct Almanac {
    pub seeds: Vec<u64>,
    pub seed_soil: MappingTable,
    pub soil_fert: MappingTable,
    pub fert_water: MappingTable,
    pub water_light: MappingTable,
    pub light_temp: MappingTable,
    pub temp_humid: MappingTable,
    pub humid_loc: MappingTable,
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
        let soil = self.seed_soil.dest(seed);
        let fert = self.soil_fert.dest(soil);
        let water = self.fert_water.dest(fert);
        let light = self.water_light.dest(water);
        let temp = self.light_temp.dest(light);
        let humid = self.temp_humid.dest(temp);
        self.humid_loc.dest(humid)
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

fn compatible_mappings(all_mappings: &[Mapping], prev_mapping: &Mapping) -> Vec<Mapping> {
    all_mappings
        .iter()
        .sorted_by(|&a, &b| a.dest.start.cmp(&b.dest.start))
        .filter_map(|m| {
            let Some(overlap) = range_overlap(&prev_mapping.source, &m.dest) else {
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
        tuple((tag("seeds: "), separated_list0(space1, u64))),
        |(_, seeds)| seeds,
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
            if let Some(first) = mappings.front() {
                if first.dest.start > 0 {
                    mappings.push_front(Mapping {
                        source: 0..first.dest.start,
                        dest: 0..first.dest.start,
                    });
                }
            }
            if let Some(last) = mappings.back() {
                mappings.push_back(Mapping {
                    source: last.dest.end..u64::MAX,
                    dest: last.dest.end..u64::MAX,
                });
            }
            mappings.into()
        },
    )(input)
}

impl Day for Day05 {
    type Input = Almanac;

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        map(
            tuple((
                parse_seeds,
                tag("\n\n"),
                separated_list0(tag("\n\n"), parse_mappings),
            )),
            |(seeds, _, mappings)| Almanac {
                seeds,
                seed_soil: mappings[0].clone().into(),
                soil_fert: mappings[1].clone().into(),
                fert_water: mappings[2].clone().into(),
                water_light: mappings[3].clone().into(),
                light_temp: mappings[4].clone().into(),
                temp_humid: mappings[5].clone().into(),
                humid_loc: mappings[6].clone().into(),
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

    fn part_2(input: &Self::Input) -> Self::Output2 {
        for loc_mapping in input
            .humid_loc
            .mappings
            .iter()
            .sorted_by(|&a, &b| a.dest.start.cmp(&b.dest.start))
        {
            for humid_mapping in compatible_mappings(&input.temp_humid.mappings, loc_mapping) {
                for temp_mapping in compatible_mappings(&input.light_temp.mappings, &humid_mapping)
                {
                    for light_mapping in
                        compatible_mappings(&input.water_light.mappings, &temp_mapping)
                    {
                        for water_mapping in
                            compatible_mappings(&input.fert_water.mappings, &light_mapping)
                        {
                            for fert_mapping in
                                compatible_mappings(&input.soil_fert.mappings, &water_mapping)
                            {
                                for soil_mapping in
                                    compatible_mappings(&input.seed_soil.mappings, &fert_mapping)
                                {
                                    let Some(seed_range) =
                                        input.seed_ranges().iter().find_map(|seed_range| {
                                            range_overlap(seed_range, &soil_mapping.source)
                                        })
                                    else {
                                        continue;
                                    };
                                    return seed_range.map(|s| input.location(s)).min().unwrap();
                                }
                            }
                        }
                    }
                }
            }
        }

        panic!("Couldn't find a suitable seed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        let input = "seeds: 79 14 55 13

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

        let parsed = Day05::parse(input).unwrap().1;
        assert_eq!(Day05::part_1(&parsed), 35);
    }

    #[test]
    fn test_part2() {
        let input = "seeds: 79 14 55 13

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

        let parsed = Day05::parse(input).unwrap().1;
        assert_eq!(Day05::part_2(&parsed), 46);
    }
}
