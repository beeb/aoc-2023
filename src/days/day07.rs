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

#[derive(Debug)]
pub struct Hand {
    pub cards: [Card; 5],
    pub bid: u64,
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

impl Card {
    /// When the Jack is the lowest denomination, we use a custom ordering compared to the default enum ordering
    fn custom_cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Jack is always the lowest
        match (self, other) {
            (Self::Jack, _) => std::cmp::Ordering::Less,
            (_, Self::Jack) => std::cmp::Ordering::Greater,
            (a, b) => a.cmp(b),
        }
    }
}

impl Hand {
    fn find_pattern(&self) -> Pattern {
        let counts: Vec<usize> = self
            .cards
            .iter()
            .counts()
            .iter()
            .sorted_by(|(_, &a), (_, &b)| b.cmp(&a)) // sort by descending count
            .map(|(_, &count)| count) // only keep count
            .collect();

        // qty of most common card
        let first = counts.first().unwrap_or(&0);
        // qty of second most common card
        let second = counts.get(1).unwrap_or(&0);

        // check qty of most common card
        match (first, second) {
            (5, _) => Pattern::FiveKind,
            (4, _) => Pattern::FourKind,
            (3, 2) => Pattern::FullHouse,
            (3, _) => Pattern::ThreeKind,
            (2, 2) => Pattern::TwoPairs,
            (2, _) => Pattern::Pair,
            _ => Pattern::HighCard,
        }
    }

    /// Ugly but it works ¯\_(ツ)_/¯
    fn find_pattern_joker(&self) -> Pattern {
        let counts_map = self.cards.iter().counts();
        let jokers_count = *counts_map.get(&Card::Jack).unwrap_or(&0);
        // get counts of all cards except jokers, sorted from highest to lowest
        let counts: Vec<usize> = counts_map
            .iter()
            .sorted_by(|(_, &a), (_, &b)| b.cmp(&a)) // sort by descending count
            .filter_map(|(&card, &count)| (!matches!(card, &Card::Jack)).then_some(count)) // only keep count (non-jack)
            .collect();

        // qty of most common card
        let first = counts.first().unwrap_or(&0);
        // qty of second most common card
        let second = counts.get(1).unwrap_or(&0);

        // yay for pattern matching
        match (first, second, jokers_count) {
            (5, _, _) | (4, _, 1) | (3, _, 2) | (2, _, 3) | (1, _, 4) | (_, _, 5) => {
                Pattern::FiveKind
            }
            (4, 1, _) | (3, _, 1) | (2, _, 2) | (1, _, 3) | (_, _, 4) => Pattern::FourKind,
            (3, 2, _) | (2, 2, 1) => Pattern::FullHouse,
            (3, _, _) | (2, _, 1) | (1, _, 2) | (_, _, 3) => Pattern::ThreeKind,
            (2, 2, 0) => Pattern::TwoPairs,
            (2, _, _) | (1, _, 1) | (_, _, 2) => Pattern::Pair,
            _ => Pattern::HighCard,
        }
    }

    /// Same as regular `cmp` but we use the special joker pattern finding and the `custom_cmp` for cards
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

/// Convert from a character to a card
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
    /// We don't compare on the bid amount, only cards
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

impl PartialOrd for Hand {
    /// We can always compare hands
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

    /// Part 1 took 2.389249ms
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

    /// Part 2 took 2.926981ms
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
