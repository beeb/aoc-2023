use nom::{
    bytes::complete::tag,
    character::complete::{line_ending, space1, u128, u16},
    combinator::map,
    multi::{separated_list0, separated_list1},
    sequence::tuple,
    IResult,
};

use crate::days::Day;

pub struct Day04;

#[derive(Debug)]
pub struct Card {
    pub winning: u128,
    pub numbers: u128,
}

fn parse_numbers_bitmap(input: &str) -> IResult<&str, u128> {
    let (rest, numbers) = separated_list1(space1, u128)(input)?;
    let mut bitmap = 0u128;
    for number in numbers {
        bitmap |= 1 << number;
    }
    Ok((rest, bitmap))
}

fn parse_card(input: &str) -> IResult<&str, Card> {
    map(
        tuple((
            tag("Card"),
            space1,
            u16,
            tag(":"),
            space1,
            parse_numbers_bitmap,
            tag(" |"),
            space1,
            parse_numbers_bitmap,
        )),
        |(_, _, _, _, _, winning, _, _, numbers)| Card { winning, numbers },
    )(input)
}

impl Day for Day04 {
    type Input = Vec<Card>;

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        separated_list0(line_ending, parse_card)(input)
    }

    type Output1 = usize;

    /// Part 1 took 0.000976ms
    fn part_1(input: &Self::Input) -> Self::Output1 {
        input
            .iter()
            .map(|card| {
                let intersection = (card.winning & card.numbers).count_ones();
                match intersection {
                    1.. => 1 << (intersection - 1),
                    0 => 0,
                }
            })
            .sum()
    }

    type Output2 = usize;

    /// Part 2 took 0.002815ms
    fn part_2(input: &Self::Input) -> Self::Output2 {
        // for each card index, how many numbers are matching
        let matching: Vec<usize> = input
            .iter()
            .map(|card| (card.winning & card.numbers).count_ones() as usize)
            .collect();
        let mut cards = vec![1usize; matching.len()];
        let cards_len = cards.len();
        for i in 0..cards_len {
            let card_amount = cards[i];
            let matching_numbers = matching[i];
            for j in 1..=matching_numbers {
                if i + j < cards_len {
                    cards[i + j] += card_amount;
                }
            }
        }
        cards.iter().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        let input = "Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19
Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1
Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83
Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36
Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11";

        let parsed = Day04::parse(input).unwrap().1;
        assert_eq!(Day04::part_1(&parsed), 13);
    }

    #[test]
    fn test_part2() {
        let input = "Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19
Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1
Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83
Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36
Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11";

        let parsed = Day04::parse(input).unwrap().1;
        assert_eq!(Day04::part_2(&parsed), 30);
    }
}
