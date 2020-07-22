use anyhow::{anyhow, Context, Error, Result};
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

struct Node {
    children: Vec<Node>,
    metadatas: Vec<u32>,
}

fn construct_nodes(s: String) -> Result<Vec<Node>, Error> {
    let mut stack_nodes: Vec<(u32, u32, Node)> = Vec::new();
    let mut nodes = Vec::new();
    let mut tokens = s.split(' ').filter_map(|token| token.parse::<u32>().ok());

    loop {
        match (tokens.next(), tokens.next()) {
            (Some(0), Some(metadata_qty)) => {
                let mut metadatas = vec![];
                for _ in 0..metadata_qty {
                    if let Some(metadata) = tokens.next() {
                        metadatas.push(metadata)
                    } else {
                        return Err(anyhow!("invalid end of data"));
                    }
                }
                let mut node = Node {
                    children: vec![],
                    metadatas,
                };

                loop {
                    match stack_nodes.pop() {
                        None => {
                            nodes.push(node);
                            break;
                        }
                        Some((1, metadata_qty, mut unfinished_node)) => {
                            for _ in 0..metadata_qty {
                                if let Some(metadata) = tokens.next() {
                                    unfinished_node.metadatas.push(metadata)
                                } else {
                                    return Err(anyhow!("invalid end of data"));
                                }
                            }
                            unfinished_node.children.push(node);
                            node = unfinished_node;
                        }
                        Some((unprocessed_child, metadata_qty, mut unfinished_node)) => {
                            unfinished_node.children.push(node);
                            stack_nodes.push((
                                unprocessed_child - 1,
                                metadata_qty,
                                unfinished_node,
                            ));
                            break;
                        }
                    }
                }
            }
            (Some(child_qty), Some(metadata_qty)) => {
                let node = Node {
                    children: Vec::with_capacity(child_qty as usize),
                    metadatas: Vec::with_capacity(metadata_qty as usize),
                };
                stack_nodes.push((child_qty, metadata_qty, node));
            }
            (None, None) => break,
            _ => return Err(anyhow!("invalid end of data")),
        }
    }
    Ok(nodes)
}

fn traverse_calculate_metadatas(nodes: &[Node]) -> u32 {
    let mut sum_metadata = 0;
    let mut nodes_stack = Vec::new();
    nodes_stack.push(nodes.iter());
    while let Some(node_iter) = nodes_stack.last_mut() {
        if let Some(node) = node_iter.next() {
            sum_metadata += node.metadatas.iter().sum::<u32>();
            if !node.children.is_empty() {
                nodes_stack.push(node.children.iter());
            }
        } else {
            nodes_stack.pop();
        }
    }
    sum_metadata
}

fn calculate_value_node(node: &Node) -> u32 {
    if node.children.is_empty() {
        node.metadatas.iter().sum::<u32>()
    } else {
        let mut children_values = vec![None; node.children.len()];
        let mut value = 0u32;
        for idx in &node.metadatas {
            let idx = *idx as usize;
            if idx <= node.children.len() && idx != 0 {
                if let Some(child_value) = children_values[idx - 1] {
                    value += child_value;
                } else {
                    let child_value = calculate_value_node(&node.children[idx - 1]);
                    children_values[idx - 1] = Some(child_value);
                    value += child_value;
                }
            }
        }
        value
    }
}

fn main() -> Result<()> {
    let file = File::open("2018/day-08/input/input.txt").context("failed to read input file")?;
    let reader = BufReader::new(file);

    if let Some(s) = reader.lines().filter_map(|line| line.ok()).next() {
        if let Ok(nodes) = construct_nodes(s) {
            let checksum_metadata = traverse_calculate_metadatas(&nodes);
            writeln!(io::stdout(), "checksum metadata: {}", checksum_metadata)?;

            let value_node = calculate_value_node(&nodes[0]);
            writeln!(io::stdout(), "root node value: {}", value_node)?;
        }
    }
    Ok(())
}
