use std::collections::{HashMap, VecDeque};

use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, line_ending, one_of},
    combinator::map,
    multi::{separated_list0, separated_list1},
    sequence::{separated_pair, tuple},
    IResult,
};
use petgraph::prelude::*;

use crate::days::Day;

pub struct Day20;

#[derive(Debug, Clone, Copy)]
pub enum Pulse {
    High,
    Low,
}

#[derive(Debug, Clone, Copy)]
pub enum State {
    On,
    Off,
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub from: NodeIndex,
    pub dest: NodeIndex,
    pub pulse: Pulse,
}

#[derive(Debug, Clone)]
pub struct FlipFlopModule {
    pub name: String,
    pub state: State,
}

#[derive(Debug, Clone)]
pub struct ConjunctionModule {
    pub name: String,
    pub input_states: HashMap<NodeIndex, Pulse>,
}

#[derive(Debug, Clone)]
pub enum Module {
    FlipFlop(FlipFlopModule),
    Conjunction(ConjunctionModule),
    Broadcaster,
    Button,
    Output(String),
}

impl FlipFlopModule {
    fn flip(&mut self) -> Pulse {
        let (new_state, pulse) = match self.state {
            State::On => (State::Off, Pulse::Low),
            State::Off => (State::On, Pulse::High),
        };
        self.state = new_state;
        pulse
    }
}

impl ConjunctionModule {
    fn record_pulse(&mut self, parent: NodeIndex, pulse: Pulse) {
        self.input_states.insert(parent, pulse);
    }

    fn get_output(&self, parents: &[NodeIndex]) -> Pulse {
        let all_high = parents
            .iter()
            .map(|i| self.input_states.get(i).unwrap_or(&Pulse::Low))
            .all(|p| matches!(p, Pulse::High));
        if all_high {
            Pulse::Low
        } else {
            Pulse::High
        }
    }
}

impl Module {
    fn name(&self) -> String {
        match self {
            Self::FlipFlop(m) => m.name.clone(),
            Self::Conjunction(m) => m.name.clone(),
            Self::Broadcaster => "broadcaster".to_string(),
            Self::Button => "button".to_string(),
            Self::Output(name) => name.clone(),
        }
    }
}

fn parse_module(input: &str) -> IResult<&str, Module> {
    alt((
        map(tag("broadcaster"), |_| Module::Broadcaster),
        map(
            tuple((one_of("%&"), alpha1::<&str, _>)),
            |(t, name)| match t {
                '%' => Module::FlipFlop(FlipFlopModule {
                    name: name.to_string(),
                    state: State::Off,
                }),
                '&' => Module::Conjunction(ConjunctionModule {
                    name: name.to_string(),
                    input_states: HashMap::new(),
                }),
                _ => unimplemented!(),
            },
        ),
    ))(input)
}

fn parse_outputs(input: &str) -> IResult<&str, Vec<String>> {
    separated_list1(tag(", "), map(alpha1::<&str, _>, ToString::to_string))(input)
}

fn create_graph(
    modules: &HashMap<String, Module>,
    outputs_map: HashMap<String, Vec<String>>,
) -> (Graph<Module, ()>, NodeIndex) {
    let mut graph = Graph::<Module, ()>::new();
    let mut node_indices = HashMap::new();
    let mut node_outputs = HashMap::new();
    let mut button_idx = None;
    for (name, outputs) in outputs_map {
        let module = modules.get(&name).unwrap().clone();
        let idx = graph.add_node(module);
        if name == "broadcaster" {
            button_idx = Some(graph.add_node(Module::Button));
            graph.add_edge(button_idx.unwrap(), idx, ());
        }
        node_indices.insert(name, idx);
        node_outputs.insert(idx, outputs);
    }
    for (node, outputs) in node_outputs {
        for output in outputs {
            if let Some(output_node) = node_indices.get(&output) {
                graph.add_edge(node, *output_node, ());
            } else {
                let output_idx = graph.add_node(Module::Output(output.clone()));
                graph.add_edge(node, output_idx, ());
            }
        }
    }
    (graph, button_idx.unwrap())
}

impl Day for Day20 {
    type Input = (HashMap<String, Module>, HashMap<String, Vec<String>>);

    fn parse(input: &str) -> IResult<&str, Self::Input> {
        let (rest, items) = separated_list0(
            line_ending,
            map(
                separated_pair(parse_module, tag(" -> "), parse_outputs),
                |(module, outputs)| ((module.name(), module.clone()), (module.name(), outputs)),
            ),
        )(input)?;
        Ok((rest, items.into_iter().unzip()))
    }

    type Output1 = usize;

    fn part_1(input: &Self::Input) -> Self::Output1 {
        let (modules, outputs) = input;
        let (mut graph, button_idx) = create_graph(modules, outputs.clone());
        let mut low_pulses = 0;
        let mut high_pulses = 0;
        for _ in 0..1000 {
            let mut instr = VecDeque::<Instruction>::new();
            instr.push_back(Instruction {
                from: button_idx,
                dest: graph.neighbors(button_idx).next().unwrap(),
                pulse: Pulse::Low,
            });
            while let Some(i) = instr.pop_front() {
                match i.pulse {
                    Pulse::Low => low_pulses += 1,
                    Pulse::High => high_pulses += 1,
                }
                let parents = graph
                    .neighbors_directed(i.dest, Direction::Incoming)
                    .collect_vec();
                let dest = &mut graph[i.dest];
                match dest {
                    Module::Button | Module::Output(_) => {}
                    Module::Broadcaster => {
                        for n in graph.neighbors_directed(i.dest, Direction::Outgoing) {
                            instr.push_back(Instruction {
                                from: i.dest,
                                dest: n,
                                pulse: Pulse::Low,
                            });
                        }
                    }
                    Module::FlipFlop(m) => match i.pulse {
                        Pulse::High => {}
                        Pulse::Low => {
                            let pulse = m.flip();
                            for n in graph.neighbors_directed(i.dest, Direction::Outgoing) {
                                instr.push_back(Instruction {
                                    from: i.dest,
                                    dest: n,
                                    pulse,
                                });
                            }
                        }
                    },
                    Module::Conjunction(m) => {
                        m.record_pulse(i.from, i.pulse);
                        let pulse = m.get_output(&parents);
                        for n in graph.neighbors_directed(i.dest, Direction::Outgoing) {
                            instr.push_back(Instruction {
                                from: i.dest,
                                dest: n,
                                pulse,
                            });
                        }
                    }
                }
            }
        }
        low_pulses * high_pulses
    }

    type Output2 = usize;

    fn part_2(_input: &Self::Input) -> Self::Output2 {
        unimplemented!("part_2")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const INPUT: &str = "broadcaster -> a
%a -> inv, con
&inv -> b
%b -> con
&con -> output";

    #[test]
    fn test_part1() {
        let parsed = Day20::parse(INPUT).unwrap().1;
        assert_eq!(Day20::part_1(&parsed), 11_687_500);
    }

    // #[test]
    // fn test_part2() {
    //     let parsed = Day20::parse(INPUT).unwrap().1;
    //     assert_eq!(Day20::part_2(&parsed), 167_409_079_868_000);
    // }
}
