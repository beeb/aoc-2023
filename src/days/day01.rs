use nom::{
    character::complete::{alphanumeric1, line_ending},
    combinator::map,
    multi::separated_list0,
    IResult,
};

use crate::days::Day;

pub struct Day01;

impl Day for Day01 {
    type Input = Vec<String>;

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        separated_list0(line_ending, map(alphanumeric1, |s: &str| s.to_string()))(input)
    }

    type Output1 = usize;

    /// Part 1 took 0.049463ms
    fn part_1(input: &Self::Input) -> Self::Output1 {
        input
            .iter()
            .map(|l| {
                let digits: Vec<usize> = l
                    .chars()
                    .filter_map(|char| match char {
                        ch if ch.is_ascii_digit() => Some(ch.to_digit(10).unwrap() as usize),
                        _ => None,
                    })
                    .collect();

                digits.first().unwrap_or(&0) * 10 + digits.last().unwrap_or(&0)
            })
            .sum()
    }

    type Output2 = usize;

    /// Part 2 took 0.164261ms
    fn part_2(input: &Self::Input) -> Self::Output2 {
        input
            .iter()
            .map(|line| {
                let digits: Vec<_> = line
                    .chars()
                    .enumerate()
                    .filter_map(|(i, ch)| {
                        if ch.is_ascii_digit() {
                            Some(ch.to_digit(10).unwrap() as usize)
                        } else {
                            match &line[i..] {
                                s if s.starts_with("one") => Some(1),
                                s if s.starts_with("two") => Some(2),
                                s if s.starts_with("three") => Some(3),
                                s if s.starts_with("four") => Some(4),
                                s if s.starts_with("five") => Some(5),
                                s if s.starts_with("six") => Some(6),
                                s if s.starts_with("seven") => Some(7),
                                s if s.starts_with("eight") => Some(8),
                                s if s.starts_with("nine") => Some(9),
                                _ => None,
                            }
                        }
                    })
                    .collect();
                digits.first().unwrap_or(&0) * 10 + digits.last().unwrap_or(&0)
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        let input = "1abc2
pqr3stu8vwx
a1b2c3d4e5f
treb7uchet";

        let parsed = Day01::parse(input).unwrap().1;
        assert_eq!(Day01::part_1(&parsed), 142);
    }

    #[test]
    fn test_part2() {
        let input = "two1nine
eightwothree
abcone2threexyz
xtwone3four
4nineeightseven2
zoneight234
7pqrstsixteen";

        let parsed = Day01::parse(input).unwrap().1;
        assert_eq!(Day01::part_2(&parsed), 281);
    }
}
