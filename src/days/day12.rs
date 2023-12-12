use itertools::{repeat_n, Itertools};
use nom::{
    branch::alt,
    character::complete::{char, line_ending, u64},
    combinator::map,
    multi::{many0, separated_list0, separated_list1},
    sequence::separated_pair,
    IResult,
};

use crate::days::Day;

pub struct Day12;

#[derive(Debug, Clone)]
pub enum HotSpring {
    Working,
    Broken,
    Unknown,
}

#[derive(Debug)]
pub struct HotSpringRow {
    pub springs: Vec<HotSpring>,
    pub groups: Vec<usize>,
}

fn parse_springs(input: &str) -> IResult<&str, Vec<HotSpring>> {
    many0(alt((
        map(char('.'), |_| HotSpring::Working),
        map(char('#'), |_| HotSpring::Broken),
        map(char('?'), |_| HotSpring::Unknown),
    )))(input)
}

#[allow(clippy::cast_possible_truncation)]
fn parse_groups(input: &str) -> IResult<&str, Vec<usize>> {
    separated_list1(char(','), map(u64, |n| n as usize))(input)
}

pub fn is_solution_valid(springs: &[HotSpring], groups: &[usize]) -> bool {
    // first check that the numbers match
    if groups.iter().sum::<usize>()
        != springs
            .iter()
            .filter(|s| matches!(s, HotSpring::Broken))
            .count()
    {
        return false;
    }
    // then check that the groups match
    let effective_groups = springs
        .split(|s| matches!(s, HotSpring::Working))
        .filter_map(|g| if g.is_empty() { None } else { Some(g.len()) })
        .collect_vec();
    effective_groups == groups
}

pub fn find_all_solutions(springs: &[HotSpring], groups: &[usize]) -> Vec<Vec<HotSpring>> {
    let unknowns = springs
        .iter()
        .filter(|s| matches!(s, HotSpring::Unknown))
        .count();
    let combinations = repeat_n([HotSpring::Working, HotSpring::Broken], unknowns)
        .multi_cartesian_product()
        .collect_vec();
    combinations
        .into_iter()
        .map(|mut knowns| {
            springs
                .iter()
                .map(|s| match s {
                    HotSpring::Unknown => knowns.pop().unwrap_or(s.clone()),
                    _ => s.clone(),
                })
                .collect_vec()
        })
        .filter(|row| is_solution_valid(row, groups))
        .collect()
}

impl Day for Day12 {
    type Input = Vec<HotSpringRow>;

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        separated_list0(
            line_ending,
            map(
                separated_pair(parse_springs, char(' '), parse_groups),
                |(springs, groups)| HotSpringRow { springs, groups },
            ),
        )(input)
    }

    type Output1 = usize;

    fn part_1(input: &Self::Input) -> Self::Output1 {
        input
            .iter()
            .map(|row| find_all_solutions(&row.springs, &row.groups).len())
            .sum()
    }

    type Output2 = usize;

    fn part_2(_input: &Self::Input) -> Self::Output2 {
        unimplemented!("part_2")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = "???.### 1,1,3
.??..??...?##. 1,1,3
?#?#?#?#?#?#?#? 1,3,1,6
????.#...#... 4,1,1
????.######..#####. 1,6,5
?###???????? 3,2,1";

    #[test]
    fn test_part1() {
        let parsed = Day12::parse(INPUT).unwrap().1;
        assert_eq!(Day12::part_1(&parsed), 21);
    }

    #[test]
    fn test_is_solutions_valid() {
        assert!(is_solution_valid(
            &[
                HotSpring::Broken,
                HotSpring::Working,
                HotSpring::Broken,
                HotSpring::Working,
                HotSpring::Broken,
                HotSpring::Broken,
                HotSpring::Broken
            ],
            &[1, 1, 3]
        ));
        assert!(!is_solution_valid(
            &[
                HotSpring::Working,
                HotSpring::Broken,
                HotSpring::Broken,
                HotSpring::Working,
                HotSpring::Broken,
                HotSpring::Broken,
                HotSpring::Broken
            ],
            &[1, 1, 3]
        ));
    }
}
