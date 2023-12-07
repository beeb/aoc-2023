use itertools::Itertools;
use nom::{
    character::complete::{anychar, char, line_ending, u64},
    combinator::map,
    multi::{count, separated_list0},
    sequence::separated_pair,
    IResult,
};

use crate::days::Day;

pub struct Day07;

#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub enum Card {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Card {
    fn custom_cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Jack, _) => std::cmp::Ordering::Less,
            (_, Self::Jack) => std::cmp::Ordering::Greater,
            (a, b) => a.cmp(b),
        }
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug)]
pub enum Pattern {
    HighCard,
    Pair,
    TwoPairs,
    ThreeKind,
    FullHouse,
    FourKind,
    FiveKind,
}

#[derive(Debug)]
pub struct Hand {
    pub cards: [Card; 5],
    pub bid: u64,
}

impl Hand {
    fn find_pattern(&self) -> Pattern {
        let counts: Vec<usize> = self
            .cards
            .iter()
            .counts()
            .iter()
            .sorted_by(|(_, &a), (_, &b)| b.cmp(&a))
            .map(|(_, &count)| count)
            .collect();

        match counts.first().unwrap() {
            5 => Pattern::FiveKind,
            4 => Pattern::FourKind,
            3 => match counts.get(1).unwrap() {
                2 => Pattern::FullHouse,
                _ => Pattern::ThreeKind,
            },
            2 => match counts.get(1).unwrap() {
                2 => Pattern::TwoPairs,
                _ => Pattern::Pair,
            },
            _ => Pattern::HighCard,
        }
    }

    fn find_pattern_joker(&self) -> Pattern {
        let counts_map = self.cards.iter().counts();
        let jokers_count = *counts_map.get(&Card::Jack).unwrap_or(&0);
        let counts: Vec<usize> = counts_map
            .iter()
            .sorted_by(|(_, &a), (_, &b)| b.cmp(&a))
            .filter_map(|(&card, &count)| {
                if matches!(card, &Card::Jack) {
                    None
                } else {
                    Some(count)
                }
            })
            .collect();

        match counts.first().unwrap_or(&0) {
            5 => Pattern::FiveKind,
            4 => {
                if jokers_count >= 1 {
                    Pattern::FiveKind
                } else {
                    Pattern::FourKind
                }
            }
            3 => match (counts.get(1).unwrap_or(&0), jokers_count) {
                (_, 2) => Pattern::FiveKind,
                (_, 1) => Pattern::FourKind,
                (2, _) => Pattern::FullHouse,
                (1, _) => Pattern::ThreeKind,
                (_, _) => unreachable!("Invalid hand"),
            },
            2 => match (counts.get(1).unwrap_or(&0), jokers_count) {
                (_, 3) => Pattern::FiveKind,
                (_, 2) => Pattern::FourKind,
                (2, 1) => Pattern::FullHouse,
                (_, 1) => Pattern::ThreeKind,
                (2, 0) => Pattern::TwoPairs,
                (_, 0) => Pattern::Pair,
                (_, _) => Pattern::HighCard,
            },
            1 => match (counts.get(1).unwrap_or(&0), jokers_count) {
                (_, 4) => Pattern::FiveKind,
                (_, 3) => Pattern::FourKind,
                (_, 2) => Pattern::ThreeKind,
                (_, 1) => Pattern::Pair,
                (_, _) => Pattern::HighCard,
            },
            _ => match jokers_count {
                5 => Pattern::FiveKind,
                4 => Pattern::FourKind,
                3 => Pattern::ThreeKind,
                2 => Pattern::Pair,
                _ => Pattern::HighCard,
            },
        }
    }
}

impl From<char> for Card {
    fn from(value: char) -> Self {
        match value {
            '2' => Self::Two,
            '3' => Self::Three,
            '4' => Self::Four,
            '5' => Self::Five,
            '6' => Self::Six,
            '7' => Self::Seven,
            '8' => Self::Eight,
            '9' => Self::Nine,
            'T' => Self::Ten,
            'J' => Self::Jack,
            'Q' => Self::Queen,
            'K' => Self::King,
            'A' => Self::Ace,
            _ => unimplemented!("Invalid card: {}", value),
        }
    }
}

impl Eq for Hand {}

impl PartialEq for Hand {
    fn eq(&self, other: &Self) -> bool {
        self.cards == other.cards
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_pattern = self.find_pattern();
        let other_pattern = other.find_pattern();

        if self_pattern != other_pattern {
            return self_pattern.cmp(&other_pattern);
        }

        let self_cards = self.cards.iter();
        let other_cards = other.cards.iter();

        for (self_card, other_card) in self_cards.zip(other_cards) {
            if self_card != other_card {
                return self_card.cmp(other_card);
            }
        }
        std::cmp::Ordering::Equal
    }
}

impl Hand {
    fn custom_cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_pattern = self.find_pattern_joker();
        let other_pattern = other.find_pattern_joker();

        if self_pattern != other_pattern {
            return self_pattern.cmp(&other_pattern);
        }

        let self_cards = self.cards.iter();
        let other_cards = other.cards.iter();

        for (self_card, other_card) in self_cards.zip(other_cards) {
            if self_card != other_card {
                return self_card.custom_cmp(other_card);
            }
        }
        std::cmp::Ordering::Equal
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Day for Day07 {
    type Input = Vec<Hand>;

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        separated_list0(
            line_ending,
            map(
                separated_pair(count(anychar, 5), char(' '), u64),
                |(cards, bid)| {
                    let cards: [char; 5] = cards[..5].try_into().unwrap();
                    Hand {
                        cards: cards.map(Into::into),
                        bid,
                    }
                },
            ),
        )(input)
    }

    type Output1 = usize;

    #[allow(clippy::cast_possible_truncation)]
    fn part_1(input: &Self::Input) -> Self::Output1 {
        input
            .iter()
            .sorted()
            .enumerate()
            .map(|(i, hand)| (i + 1) * hand.bid as usize)
            .sum()
    }

    type Output2 = usize;

    #[allow(clippy::cast_possible_truncation)]
    fn part_2(input: &Self::Input) -> Self::Output2 {
        input
            .iter()
            .sorted_by(|a, b| a.custom_cmp(b))
            .enumerate()
            .map(|(i, hand)| (i + 1) * hand.bid as usize)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = "32T3K 765
T55J5 684
KK677 28
KTJJT 220
QQQJA 483";

    #[test]
    fn test_part1() {
        let parsed = Day07::parse(INPUT).unwrap().1;
        assert_eq!(Day07::part_1(&parsed), 6440);
    }

    #[test]
    fn test_part2() {
        let parsed = Day07::parse(INPUT).unwrap().1;
        assert_eq!(Day07::part_2(&parsed), 5905);
    }
}
