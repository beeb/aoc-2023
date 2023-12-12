use std::collections::HashMap;

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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
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

/// How many solutions are there, considering a subset of the hotsprings and groups where we skip items at the beggining
///
/// Solved recursively and caching results
fn count_solutions<'a>(
    cache: &mut HashMap<(&'a [HotSpring], &'a [usize]), usize>,
    springs: &'a [HotSpring],
    groups: &'a [usize],
    first_is_broken: bool,
) -> usize {
    if springs.is_empty() && groups.is_empty() {
        return 1;
    } else if springs.is_empty() {
        // not a valid solution
        return 0;
    }

    // check cache and return if possible
    if let Some(res) = cache.get(&(springs, groups)) {
        return *res;
    }

    // check the first item in the subset of springs
    match (springs.first(), first_is_broken) {
        (Some(HotSpring::Working), _) => {
            // if first item is working, then we simply continue looking starting at the next spring
            let res = count_solutions(cache, &springs[1..], groups, false);
            cache.insert((springs, groups), res);
            res
        }
        (Some(HotSpring::Broken), _) | (Some(HotSpring::Unknown), true) => {
            // if the first item is broken (or is unknown but we want to consider it as broken)
            if groups.is_empty() {
                // if we have no more groups to "assign", we have no more options to consider
                cache.insert((springs, groups), 0);
                return 0;
            }

            // length of the first group to be assigned
            let first_group_length = *groups.first().unwrap();

            // number of contiguous positions we have at our disposal to fit this group (broken or unknown)
            let maybe_broken = springs
                .iter()
                .take_while(|s| !matches!(s, HotSpring::Working))
                .count();
            if maybe_broken < first_group_length {
                // first group wouldn't fit first space, dead end
                cache.insert((springs, groups), 0);
                return 0;
            }
            // group would fit, let's investigate

            // rest after we assign the first group to the start of the space
            let rest = &springs[first_group_length..];
            // let's consider the first item of that rest
            match rest.first() {
                None => {
                    // We are at the end of the row, there is nothing left after we assign the group.
                    // Let's consider the group assigned and count the options for the remaining groups.
                    // This will either be 0 or 1 depending on if all the groups have been assigned or not.
                    // If we have assigned the last group, then the call below will return 1.
                    // If we still ahve leftover groups but no more springs, then this is not a valid solution and the
                    // call below will return 0.
                    let res = count_solutions(cache, rest, &groups[1..], false);
                    cache.insert((springs, groups), res);
                    res
                }
                Some(HotSpring::Broken) => {
                    // Not allowed, there should be a gap after the group
                    cache.insert((springs, groups), 0);
                    0
                }
                Some(HotSpring::Working | HotSpring::Unknown) => {
                    // if unknown, needs to be a working spring anyway (due to us assigning a group of broken springs),
                    // so we can skip the first item
                    let res = count_solutions(cache, &rest[1..], &groups[1..], false);
                    cache.insert((springs, groups), res);
                    res
                }
            }
        }
        (Some(HotSpring::Unknown), false) => {
            // If the first item is unknown, it might be working or broken.
            // Let's count the solutions when it's working (same as above)
            let res_if_working = count_solutions(cache, &springs[1..], groups, false);
            // In case the first item would be broken, we override the first element's type by passing `true` as the
            // last param, we will consider it just like a broken spring (same as above).
            let res_if_broken = count_solutions(cache, springs, groups, true); // consider the first one broken

            // The number of valid options is the sum of those cases
            let res = res_if_working + res_if_broken;
            cache.insert((springs, groups), res);
            res
        }
        (None, _) => unreachable!(),
    }
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

    /// Part 1 took 5.584ms
    fn part_1(input: &Self::Input) -> Self::Output1 {
        input
            .iter()
            .map(|row| {
                let mut cache = HashMap::new();
                count_solutions(&mut cache, &row.springs, &row.groups, false)
            })
            .sum()
    }

    type Output2 = usize;

    /// Part 2 took 220.059002ms
    fn part_2(input: &Self::Input) -> Self::Output2 {
        input
            .iter()
            .map(|row| {
                let mut cache = HashMap::new();
                let mut springs = repeat_n(row.springs.iter().copied().collect_vec(), 5)
                    .map(|mut a| {
                        // add the separator (a "unknown" spring)
                        a.push(HotSpring::Unknown);
                        a
                    })
                    .collect_vec()
                    .concat();
                // drop the last unknown separator
                springs.pop();
                let groups = row
                    .groups
                    .iter()
                    .cycle()
                    .take(row.groups.len() * 5)
                    .copied()
                    .collect_vec();
                count_solutions(&mut cache, &springs, &groups, false)
            })
            .sum()
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
    fn test_part2() {
        let parsed = Day12::parse(INPUT).unwrap().1;
        assert_eq!(Day12::part_2(&parsed), 525_152);
    }
}
