use std::collections::HashMap;

use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, char, line_ending, one_of, u64},
    combinator::{map, opt},
    multi::separated_list1,
    sequence::{preceded, separated_pair, terminated, tuple},
    IResult,
};
use ranges::{GenericRange, OperationResult};

use crate::days::Day;

pub struct Day19;

#[derive(Debug, Clone)]
pub struct Part {
    x: u64,
    m: u64,
    a: u64,
    s: u64,
}

#[derive(Debug, Clone)]
pub enum Action {
    Goto(String),
    Accepted,
    Rejected,
}

/// A condition, either greater than or less than, with the first item being the parameter (x, m, a or s) and second
/// being the value to use for the comparison
#[derive(Debug, Clone, Copy)]
pub enum Condition {
    Lt(char, u64),
    Gt(char, u64),
}

#[derive(Debug, Clone)]
pub struct Rule {
    cond: Option<Condition>,
    action: Action,
}

#[derive(Debug, Clone)]
pub struct Workflow {
    name: String,
    rules: Vec<Rule>,
}

impl Part {
    /// Sum of the four parameters
    fn score(&self) -> u64 {
        self.x + self.m + self.a + self.s
    }
}

/// Check if a part is accepted after being processed by all the workflows
fn is_part_accepted(workflow: &str, part: &Part, workflows: &HashMap<String, Workflow>) -> bool {
    let workflow = workflows.get(workflow).unwrap();
    // apply each rule until the part is accepted or rejected
    for rule in &workflow.rules {
        // check if there is a condition, and if yes, whether it's true
        let cond_true = match rule.cond {
            Some(cond) => match cond {
                // condider on which value we need to branch and perform the comparison
                Condition::Lt(param, value) => match param {
                    'x' => part.x < value,
                    'm' => part.m < value,
                    'a' => part.a < value,
                    's' => part.s < value,
                    _ => unimplemented!(),
                },
                Condition::Gt(param, value) => match param {
                    'x' => part.x > value,
                    'm' => part.m > value,
                    'a' => part.a > value,
                    's' => part.s > value,
                    _ => unimplemented!(),
                },
            },
            // when no condition, it's always true
            None => true,
        };
        if cond_true {
            // in case the condition was matched (or we are at the last rule), then categorize accordingly
            return match &rule.action {
                Action::Goto(w) => is_part_accepted(w, part, workflows), // recursively find out if accepted
                Action::Accepted => true,
                Action::Rejected => false,
            };
        }
    }
    false
}

/// Calculate the overlap of two ranges, returning an empty range if none
fn range_overlap(a: GenericRange<u64>, b: GenericRange<u64>) -> GenericRange<u64> {
    match a & b {
        OperationResult::Empty | OperationResult::Double(_, _) => GenericRange::from(0..0),
        OperationResult::Single(r) => r,
    }
}

/// Calculate how many values are in a range
fn len(range: GenericRange<u64>) -> u64 {
    range.into_iter().count() as u64
}

/// Check how many combinations of values are accepted considering the set of workflows
///
/// Ranges for each parameter are passed to the function, and are initialized at 1..=4000
#[allow(clippy::too_many_lines)]
fn combinations(
    workflow: &str,
    workflows: &HashMap<String, Workflow>,
    x: GenericRange<u64>,
    m: GenericRange<u64>,
    a: GenericRange<u64>,
    s: GenericRange<u64>,
) -> u64 {
    // make our input ranges mutable
    let (mut x, mut m, mut a, mut s) = (x, m, a, s);
    let workflow = workflows.get(workflow).unwrap();
    // sum of all combinations
    let mut sum: u64 = 0;
    for rule in &workflow.rules {
        // first set for "true" condition, second set for "false"
        let (x1, m1, a1, s1, x2, m2, a2, s2) = if let Some(cond) = rule.cond {
            match cond {
                // check the overlap between the acceptable range, and the range that was filtered in previous steps
                // we return two disjoint ranges, one if the condition was true, the other if it was false
                Condition::Lt(param, value) => match param {
                    'x' => (
                        range_overlap(x, GenericRange::new_less_than(value)),
                        m,
                        a,
                        s,
                        range_overlap(x, GenericRange::new_at_least(value)),
                        m,
                        a,
                        s,
                    ),
                    'm' => (
                        x,
                        range_overlap(m, GenericRange::new_less_than(value)),
                        a,
                        s,
                        x,
                        range_overlap(m, GenericRange::new_at_least(value)),
                        a,
                        s,
                    ),
                    'a' => (
                        x,
                        m,
                        range_overlap(a, GenericRange::new_less_than(value)),
                        s,
                        x,
                        m,
                        range_overlap(a, GenericRange::new_at_least(value)),
                        s,
                    ),
                    's' => (
                        x,
                        m,
                        a,
                        range_overlap(s, GenericRange::new_less_than(value)),
                        x,
                        m,
                        a,
                        range_overlap(s, GenericRange::new_at_least(value)),
                    ),
                    _ => unimplemented!(),
                },
                Condition::Gt(param, value) => match param {
                    'x' => (
                        range_overlap(x, GenericRange::new_greater_than(value)),
                        m,
                        a,
                        s,
                        range_overlap(x, GenericRange::new_at_most(value)),
                        m,
                        a,
                        s,
                    ),
                    'm' => (
                        x,
                        range_overlap(m, GenericRange::new_greater_than(value)),
                        a,
                        s,
                        x,
                        range_overlap(m, GenericRange::new_at_most(value)),
                        a,
                        s,
                    ),
                    'a' => (
                        x,
                        m,
                        range_overlap(a, GenericRange::new_greater_than(value)),
                        s,
                        x,
                        m,
                        range_overlap(a, GenericRange::new_at_most(value)),
                        s,
                    ),
                    's' => (
                        x,
                        m,
                        a,
                        range_overlap(s, GenericRange::new_greater_than(value)),
                        x,
                        m,
                        a,
                        range_overlap(s, GenericRange::new_at_most(value)),
                    ),
                    _ => unimplemented!(),
                },
            }
        } else {
            // we had no condition, so we are at the last filter and simply consider each case
            let comb = match &rule.action {
                Action::Goto(wf) => combinations(wf, workflows, x, m, a, s),
                Action::Accepted => len(x) * len(m) * len(a) * len(s),
                Action::Rejected => 0,
            };
            sum += comb;
            break;
        };
        // here we had a condition so we split into two range sets
        // the first set is for when the condition was true and we process with the rule's action
        let true_comb = match &rule.action {
            Action::Goto(wf) => combinations(wf, workflows, x1, m1, a1, s1),
            Action::Accepted => len(x1) * len(m1) * len(a1) * len(s1),
            Action::Rejected => 0,
        };
        sum += true_comb;
        // the second set continues to get processed by the following rules
        (x, m, a, s) = (x2, m2, a2, s2);
    }
    sum
}

fn parse_condition(input: &str) -> IResult<&str, Condition> {
    let (rest, (param, comp, value)) = tuple((one_of("xmas"), one_of("<>"), u64))(input)?;
    let cond = match comp {
        '<' => Condition::Lt(param, value),
        '>' => Condition::Gt(param, value),
        _ => unimplemented!(),
    };
    Ok((rest, cond))
}

fn parse_rule(input: &str) -> IResult<&str, Rule> {
    map(
        tuple((opt(terminated(parse_condition, char(':'))), alpha1)),
        |(cond, action)| {
            let action = match action {
                "A" => Action::Accepted,
                "R" => Action::Rejected,
                s => Action::Goto(s.to_string()),
            };
            Rule { cond, action }
        },
    )(input)
}

fn parse_workflow(input: &str) -> IResult<&str, Workflow> {
    let (rest, (name, rules)) = tuple((
        alpha1,
        preceded(
            char('{'),
            terminated(separated_list1(char(','), parse_rule), char('}')),
        ),
    ))(input)?;
    Ok((
        rest,
        Workflow {
            name: name.to_string(),
            rules,
        },
    ))
}

fn parse_part(input: &str) -> IResult<&str, Part> {
    let (rest, (x, m, a, s)) = preceded(
        char('{'),
        terminated(
            tuple((
                preceded(tag("x="), u64),
                preceded(tag(",m="), u64),
                preceded(tag(",a="), u64),
                preceded(tag(",s="), u64),
            )),
            char('}'),
        ),
    )(input)?;
    Ok((rest, Part { x, m, a, s }))
}

impl Day for Day19 {
    type Input = (HashMap<String, Workflow>, Vec<Part>);

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        separated_pair(
            map(separated_list1(line_ending, parse_workflow), |workflows| {
                workflows.into_iter().map(|w| (w.name.clone(), w)).collect()
            }),
            tag("\n\n"),
            separated_list1(line_ending, parse_part),
        )(input)
    }

    type Output1 = u64;

    /// Part 1 took 35.646µs
    fn part_1(input: &Self::Input) -> Self::Output1 {
        let (workflows, parts) = input;
        parts
            .iter()
            .filter_map(|p| {
                if is_part_accepted("in", p, workflows) {
                    Some(p.score())
                } else {
                    None
                }
            })
            .sum()
    }

    type Output2 = u64;

    /// Part 2 took 2.352422ms
    fn part_2(input: &Self::Input) -> Self::Output2 {
        let (workflows, _) = input;
        combinations(
            "in",
            workflows,
            GenericRange::from(1..=4000),
            GenericRange::from(1..=4000),
            GenericRange::from(1..=4000),
            GenericRange::from(1..=4000),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const INPUT: &str = "px{a<2006:qkq,m>2090:A,rfg}
pv{a>1716:R,A}
lnx{m>1548:A,A}
rfg{s<537:gd,x>2440:R,A}
qs{s>3448:A,lnx}
qkq{x<1416:A,crn}
crn{x>2662:A,R}
in{s<1351:px,qqz}
qqz{s>2770:qs,m<1801:hdj,R}
gd{a>3333:R,R}
hdj{m>838:A,pv}

{x=787,m=2655,a=1222,s=2876}
{x=1679,m=44,a=2067,s=496}
{x=2036,m=264,a=79,s=2244}
{x=2461,m=1339,a=466,s=291}
{x=2127,m=1623,a=2188,s=1013}";

    #[test]
    fn test_part1() {
        let parsed = Day19::parse(INPUT).unwrap().1;
        assert_eq!(Day19::part_1(&parsed), 19114);
    }

    #[test]
    fn test_part2() {
        let parsed = Day19::parse(INPUT).unwrap().1;
        assert_eq!(Day19::part_2(&parsed), 167_409_079_868_000);
    }
}
