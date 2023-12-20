use std::collections::{BTreeMap, HashMap, VecDeque};

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
use num::Integer;
use petgraph::prelude::*;

use crate::days::Day;

pub struct Day20;

#[derive(Debug, Clone, Copy, Hash)]
pub enum Pulse {
    High,
    Low,
}

#[derive(Debug, Clone, Copy, Hash)]
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

#[derive(Debug, Clone, Hash)]
pub struct FlipFlopModule {
    pub name: String,
    pub state: State,
}

#[derive(Debug, Clone, Hash)]
pub struct ConjunctionModule {
    pub name: String,
    pub input_states: BTreeMap<NodeIndex, Pulse>,
}

#[derive(Debug, Clone, Hash)]
pub enum Module {
    FlipFlop(FlipFlopModule),
    Conjunction(ConjunctionModule),
    Broadcaster,
    Button,
    Output,
}

impl FlipFlopModule {
    /// This module outputs a pulse when flipped from On to Off or reverse
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
    /// Record the last pulse received by one of our inputs
    fn record_pulse(&mut self, parent: NodeIndex, pulse: Pulse) {
        self.input_states.insert(parent, pulse);
    }

    /// This module only outputs a low pulse if all its inputs were last high
    fn get_output(&self, parents: &[NodeIndex]) -> Pulse {
        // defaults to low if we have never received a pulse from a parent
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
            Self::Output => "rx".to_string(),
        }
    }
}

/// Construct the graph and return the indices of the input (button) and output (rx) nodes
fn create_graph(
    modules: &HashMap<String, Module>,
    outputs_map: HashMap<String, Vec<String>>,
) -> (Graph<Module, ()>, NodeIndex, NodeIndex) {
    let mut graph = Graph::<Module, ()>::new();
    let mut node_indices = HashMap::new();
    let mut node_outputs = HashMap::new();
    let mut button_idx = None;
    let mut output_idx = None;
    // create all nodes
    for (name, outputs) in outputs_map {
        let module = modules.get(&name).unwrap().clone();
        let idx = graph.add_node(module);
        // we add the button manually since it's not part of the input
        if name == "broadcaster" {
            button_idx = Some(graph.add_node(Module::Button));
            graph.add_edge(button_idx.unwrap(), idx, ());
        }
        node_indices.insert(name, idx);
        node_outputs.insert(idx, outputs);
    }
    // create all edges
    for (node, outputs) in node_outputs {
        for output in outputs {
            if let Some(output_node) = node_indices.get(&output) {
                graph.add_edge(node, *output_node, ());
            } else {
                // if we have no definition for this module's name, it means it's "rx"
                output_idx = Some(graph.add_node(Module::Output));
                graph.add_edge(node, output_idx.unwrap(), ());
            }
        }
    }
    (graph, button_idx.unwrap(), output_idx.unwrap())
}

/// Reset the state of all nodes in the graph
fn reset_graph(graph: &mut Graph<Module, ()>) {
    for node in graph.node_weights_mut() {
        match node {
            Module::FlipFlop(m) => m.state = State::Off,
            Module::Conjunction(m) => m.input_states = BTreeMap::new(),
            _ => {}
        }
    }
}

/// Press the button once, returning the number of low and high pulses transmitted and optionally whether an output
/// node did output a high pulse (for part 2)
fn press_button(
    graph: &mut Graph<Module, ()>,
    button_idx: NodeIndex,
    check_output_high: Option<NodeIndex>,
) -> (usize, usize, bool) {
    let mut low = 0;
    let mut high = 0;
    let mut output_high = false;
    // queue for instructions, we push at the back
    let mut instr = VecDeque::<Instruction>::new();
    // press the button first
    instr.push_back(Instruction {
        from: button_idx,
        dest: graph.neighbors(button_idx).next().unwrap(), // we only have 1 child = the broadcaster
        pulse: Pulse::Low,
    });
    while let Some(i) = instr.pop_front() {
        // record that one pulse was sent
        match i.pulse {
            Pulse::Low => low += 1,
            Pulse::High => high += 1,
        }
        // get a list of all parents (useful for conjunction modules)
        let parents = graph
            .neighbors_directed(i.dest, Direction::Incoming)
            .collect_vec();
        // get the current node (receiving the pulse)
        let current = &mut graph[i.dest];
        match current {
            Module::Button | Module::Output => {}
            Module::Broadcaster => {
                // the broadcaster simply sends a low pulse to its children
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
                    // flip flops only forward a pulse if they receive low
                    // update internal state and get the type of pulse we need to output
                    let pulse = m.flip();
                    // send pulse to all children
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
                // let's update internal state of Conjecture module to record what the parent sent
                m.record_pulse(i.from, i.pulse);
                // check whether all parents were last sending a high pulse, and get the pulse we should send
                let pulse = m.get_output(&parents);
                // for part 2
                if let Some(output) = check_output_high {
                    if output == i.dest && matches!(pulse, Pulse::High) {
                        // if we are on the node of interest and we did output a high pulse, we can stop propagating
                        output_high = true;
                        break;
                    }
                }
                // send the pulse to all children
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
    (low, high, output_high)
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
                    input_states: BTreeMap::new(),
                }),
                _ => unimplemented!(),
            },
        ),
    ))(input)
}

fn parse_outputs(input: &str) -> IResult<&str, Vec<String>> {
    separated_list1(tag(", "), map(alpha1::<&str, _>, ToString::to_string))(input)
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

    /// Part 1 took 1.6558ms
    fn part_1(input: &Self::Input) -> Self::Output1 {
        let (modules, outputs) = input;
        let (mut graph, button_idx, _) = create_graph(modules, outputs.clone());
        let mut low_pulses = 0;
        let mut high_pulses = 0;
        // press the button 1000 times
        for _ in 0..1000 {
            let (low, high, _) = press_button(&mut graph, button_idx, None);
            low_pulses += low;
            high_pulses += high;
        }
        low_pulses * high_pulses
    }

    type Output2 = usize;

    /// Part 2 took 26.5538ms
    fn part_2(input: &Self::Input) -> Self::Output2 {
        // From checking a graph representation of the input, we know that the output node ("rx") is connected to a
        // single parent Conjunction module (the "collector"), which has 4 "source" inputs, each also a Conjunction
        // module.
        // Since we need the collector to output "low", each of the 4 sources needs to output "high".
        let (modules, outputs) = input;
        let (mut graph, button_idx, output_idx) = create_graph(modules, outputs.clone());
        // the parent of the output (rx node) -> the collector
        let collector_idx = graph
            .neighbors_directed(output_idx, Direction::Incoming)
            .next()
            .unwrap();
        // the four nodes that feed into the collector -> the sources
        let sources = graph
            .neighbors_directed(collector_idx, Direction::Incoming)
            .collect_vec();
        // for each source, we want to know how many button presses are needed until we see a "high" output
        let presses = sources
            .iter()
            .map(|source| {
                // important, we need to reset the graph, as we consider each of the 4 sources independently
                reset_graph(&mut graph);
                let mut presses = 1; // the while loop will stop before the first high output, so we record its press
                                     // by starting at 1

                // press until the source outputs high
                while !press_button(&mut graph, button_idx, Some(*source)).2 {
                    presses += 1;
                }
                presses
            })
            .collect_vec();

        // The least common multiplier of the presses for the 4 sources is how many presses are needed to turn on the
        // machine
        presses.into_iter().reduce(|acc, e| acc.lcm(&e)).unwrap()
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
}
