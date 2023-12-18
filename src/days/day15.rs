use itertools::Itertools;
use nom::{
    character::complete::{alpha1, char, none_of, one_of, u8},
    combinator::{map, opt},
    error::ErrorKind,
    multi::{many1, separated_list1},
    sequence::tuple,
    IResult,
};

use crate::days::Day;

pub struct Day15;

#[derive(Debug)]
pub enum Action {
    Remove,
    Add(usize),
}

#[derive(Debug)]
pub struct Instruction {
    label: String,
    box_id: usize,
    action: Action,
}

#[derive(Debug)]
pub struct Lens {
    label: String,
    focal: usize,
}

impl Instruction {
    /// Parse instruction to extract lens label, box ID and action type
    fn new(input: &str) -> Self {
        let (_, (label, action, value)) = tuple((
            alpha1::<&str, _>,
            one_of::<_, _, (&str, ErrorKind)>("-="),
            opt(u8),
        ))(input)
        .unwrap();
        let box_id = hash_string(label);
        let action = match action {
            '-' => Action::Remove,
            '=' => Action::Add(value.unwrap() as usize),
            _ => unimplemented!(),
        };
        Instruction {
            label: label.to_string(),
            box_id,
            action,
        }
    }
}

/// Get the HASH for a string
fn hash_string(instr: &str) -> usize {
    instr.as_bytes().iter().fold(0, |acc, &c| {
        let mut acc = acc + c as usize;
        acc *= 17;
        acc %= 256;
        acc
    })
}

impl Day for Day15 {
    type Input = Vec<String>;

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        separated_list1(
            char(','),
            map(many1(none_of("\n,")), |s| s.into_iter().collect()),
        )(input)
    }

    type Output1 = usize;

    /// Part 1 took 42.9µs
    fn part_1(input: &Self::Input) -> Self::Output1 {
        input.iter().map(|s| hash_string(s)).sum()
    }

    type Output2 = usize;

    /// Part 2 took 423µs
    fn part_2(input: &Self::Input) -> Self::Output2 {
        // parse the input strings into instructions
        let instructions = input.iter().map(|s| Instruction::new(s)).collect_vec();
        // initialize the boxes with empty VecDeque's
        let mut boxes: Vec<Vec<Lens>> = Vec::with_capacity(256);
        for _ in 0..256 {
            boxes.push(Vec::new());
        }
        // process all instructions
        for instr in instructions {
            let b = boxes.get_mut(instr.box_id).unwrap();
            match instr.action {
                Action::Remove => {
                    // only keep lenses which have a label different from the one in the instruction
                    b.retain(|l| l.label != instr.label);
                }
                // when adding, check if there is a lens with the same label already
                Action::Add(focal) => match b.iter_mut().find(|l| l.label == instr.label) {
                    Some(lens) => {
                        // we have a lens with the same label
                        // replace the lens at the position
                        *lens = Lens {
                            label: instr.label.clone(),
                            focal,
                        };
                    }
                    None => b.push(Lens {
                        label: instr.label.clone(),
                        focal,
                    }),
                },
            }
        }
        // calculate the total focusing power
        boxes
            .iter()
            .enumerate()
            .map(|(i, lenses)| {
                lenses
                    .iter()
                    .enumerate()
                    .map(|(j, lens)| (i + 1) * (j + 1) * lens.focal)
                    .sum::<usize>()
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7";

    #[test]
    fn test_part1() {
        let parsed = Day15::parse(INPUT).unwrap().1;
        assert_eq!(Day15::part_1(&parsed), 1320);
    }

    #[test]
    fn test_part2() {
        let parsed = Day15::parse(INPUT).unwrap().1;
        assert_eq!(Day15::part_2(&parsed), 145);
    }
}
