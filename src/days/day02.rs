use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{line_ending, space1, u64},
    combinator::map,
    multi::separated_list0,
    sequence::tuple,
    IResult,
};

use crate::days::Day;

const RED: u64 = 12;
const GREEN: u64 = 13;
const BLUE: u64 = 14;

pub struct Day02;

#[derive(Debug)]
pub struct Game {
    pub id: u64,
    pub rounds: Vec<GameRound>,
}

#[derive(Debug, Default)]
pub struct GameRound {
    pub red: u64,
    pub green: u64,
    pub blue: u64,
}

fn parse_game_round(input: &str) -> IResult<&str, GameRound> {
    map(
        separated_list0(
            tag(", "),
            tuple((u64, space1, alt((tag("red"), tag("green"), tag("blue"))))),
        ),
        |cubes| {
            let mut round = GameRound::default();
            for (qty, _, color) in cubes {
                match color {
                    "red" => round.red = qty,
                    "green" => round.green = qty,
                    "blue" => round.blue = qty,
                    _ => unreachable!("invalid color"),
                }
            }
            round
        },
    )(input)
}

fn parse_game(input: &str) -> IResult<&str, Game> {
    map(
        tuple((
            tag("Game "),
            u64,
            tag(": "),
            separated_list0(tag("; "), parse_game_round),
        )),
        |(_, id, _, rounds)| Game { id, rounds },
    )(input)
}

impl Day for Day02 {
    type Input = Vec<Game>;

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        separated_list0(line_ending, parse_game)(input)
    }

    type Output1 = u64;

    /// Part 1 took 0.00244ms
    fn part_1(input: &Self::Input) -> Self::Output1 {
        input
            .iter()
            .filter_map(|game| {
                if game
                    .rounds
                    .iter()
                    .all(|round| round.red <= RED && round.green <= GREEN && round.blue <= BLUE)
                {
                    Some(game.id)
                } else {
                    None
                }
            })
            .sum()
    }

    type Output2 = u64;

    /// Part 2 took 0.005161ms
    fn part_2(input: &Self::Input) -> Self::Output2 {
        input
            .iter()
            .map(|game| {
                let red = game.rounds.iter().map(|r| r.red).max().unwrap();
                let green = game.rounds.iter().map(|r| r.green).max().unwrap();
                let blue = game.rounds.iter().map(|r| r.blue).max().unwrap();
                red * green * blue
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        let input = "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green";

        let parsed = Day02::parse(input).unwrap().1;
        assert_eq!(Day02::part_1(&parsed), 8);
    }

    #[test]
    fn test_part2() {
        let input = "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green";

        let parsed = Day02::parse(input).unwrap().1;
        assert_eq!(Day02::part_2(&parsed), 2286);
    }
}
