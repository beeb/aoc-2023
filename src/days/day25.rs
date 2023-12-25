use std::collections::HashMap;

use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, line_ending, space1},
    multi::{separated_list0, separated_list1},
    sequence::separated_pair,
    IResult,
};
use petgraph::prelude::*;
use rustworkx_core::connectivity::stoer_wagner_min_cut;

use crate::days::Day;

pub struct Day25;

impl Day for Day25 {
    type Input = UnGraph<(), ()>;

    /// Get a graph of the connected components
    fn parse(input: &str) -> IResult<&str, Self::Input> {
        let (rest, components) = separated_list0(
            line_ending,
            separated_pair(alpha1, tag(": "), separated_list1(space1, alpha1)),
        )(input)?;
        let mut graph = UnGraph::<(), ()>::new_undirected();
        let mut node_indices = HashMap::<String, NodeIndex>::new();
        for (name, _) in &components {
            let idx = graph.add_node(());
            node_indices.insert((*name).to_string(), idx);
        }
        for (name, conn) in &components {
            let this = *(node_indices.get(*name).unwrap());
            for c in conn {
                let other = if let Some(other) = node_indices.get(*c) {
                    *other
                } else {
                    graph.add_node(())
                };
                node_indices.insert((*c).to_string(), other);
                graph.add_edge(this, other, ());
            }
        }
        Ok((rest, graph))
    }

    type Output1 = usize;

    /// Part 1 took 234.191498ms
    fn part_1(input: &Self::Input) -> Self::Output1 {
        // Use a minimum cut algorithm to partition the graph into two
        let (min_cut, partition): (usize, Vec<_>) =
            stoer_wagner_min_cut::<_, _, _, anyhow::Error>(input, |_| Ok(1))
                .unwrap()
                .unwrap();
        // double-check that the number of cut edges is 3
        assert_eq!(min_cut, 3);
        partition.len() * (input.node_count() - partition.len())
    }

    type Output2 = usize;

    fn part_2(_input: &Self::Input) -> Self::Output2 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = "jqt: rhn xhk nvd
rsh: frs pzl lsr
xhk: hfx
cmg: qnr nvd lhk bvb
rhn: xhk bvb hfx
bvb: xhk hfx
pzl: lsr hfx nvd
qnr: nvd
ntq: jqt hfx bvb xhk
nvd: lhk
lsr: lhk
rzs: qnr cmg lsr rsh
frs: qnr lhk lsr";

    #[test]
    fn test_part1() {
        let parsed = Day25::parse(INPUT).unwrap().1;
        assert_eq!(Day25::part_1(&parsed), 54);
    }
}
