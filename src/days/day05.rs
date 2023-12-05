use std::ops::Range;

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
            items
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
                .collect()
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

    fn part_2(_input: &Self::Input) -> Self::Output2 {
        unimplemented!("part_2")
    }
}
