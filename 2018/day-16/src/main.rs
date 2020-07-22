#[macro_use]
extern crate lazy_static;

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, BufRead, BufReader, Write},
    rc::Rc,
};

#[derive(Debug)]
struct Operation {
    opcode_id: u8,
    in_a: u8,
    in_b: u8,
    out: u8,
}

#[derive(Debug)]
struct Sample {
    input_registers: [u64; 4],
    operation: Operation,
    output_registers: [u64; 4],
}

type Opcode = Rc<fn(&[u64; 4], u8, u8) -> u64>;

fn extract_input(input_path: &str) -> Result<(Vec<Sample>, Vec<Operation>)> {
    lazy_static! {
        static ref BEFORE: Regex = Regex::new(
            "Before: \\[(?P<reg0>[0-9]+), (?P<reg1>[0-9]+), (?P<reg2>[0-9]+), (?P<reg3>[0-9]+)\\]"
        )
        .unwrap();
        static ref OPERATION: Regex =
            Regex::new("(?P<opcode>[0-9]+) (?P<inA>[0-9]+) (?P<inB>[0-9]+) (?P<out>[0-9]+)")
                .unwrap();
        static ref AFTER: Regex = Regex::new(
            "After:  \\[(?P<reg0>[0-9]+), (?P<reg1>[0-9]+), (?P<reg2>[0-9]+), (?P<reg3>[0-9]+)\\]"
        )
        .unwrap();
    }

    let file = File::open(input_path).context("failed to read input file")?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines().filter_map(|line| {
        line.ok()
            .and_then(|s| if s.is_empty() { None } else { Some(s) })
    });

    let mut samples = vec![];
    let mut operations = vec![];
    loop {
        if let Some(line) = lines.next() {
            if let Some(before_capture) = BEFORE.captures(&line) {
                let input_registers: [u64; 4] = [
                    before_capture["reg0"].parse()?,
                    before_capture["reg1"].parse()?,
                    before_capture["reg2"].parse()?,
                    before_capture["reg3"].parse()?,
                ];
                let operation_line = lines.next().ok_or(anyhow!("operation not found!"))?;
                let operation = if let Some(operation_capture) = OPERATION.captures(&operation_line)
                {
                    Operation {
                        opcode_id: operation_capture["opcode"].parse::<u8>()?,
                        in_a: operation_capture["inA"].parse::<u8>()?,
                        in_b: operation_capture["inB"].parse::<u8>()?,
                        out: operation_capture["out"].parse::<u8>()?,
                    }
                } else {
                    return Err(anyhow!("unrecognized operation"));
                };
                let after_line = lines.next().ok_or(anyhow!("after not found"))?;
                let output_registers: [u64; 4] =
                    if let Some(after_capture) = AFTER.captures(&after_line) {
                        [
                            after_capture["reg0"].parse()?,
                            after_capture["reg1"].parse()?,
                            after_capture["reg2"].parse()?,
                            after_capture["reg3"].parse()?,
                        ]
                    } else {
                        return Err(anyhow!("unrecognized after"));
                    };

                samples.push(Sample {
                    input_registers,
                    operation,
                    output_registers,
                });
            } else if let Some(operation_capture) = OPERATION.captures(&line) {
                operations.push(Operation {
                    opcode_id: operation_capture["opcode"].parse::<u8>()?,
                    in_a: operation_capture["inA"].parse::<u8>()?,
                    in_b: operation_capture["inB"].parse::<u8>()?,
                    out: operation_capture["out"].parse::<u8>()?,
                })
            } else {
                break;
            }
        } else {
            break;
        }
    }

    Ok((samples, operations))
}

fn group_samples(
    instructions: &HashMap<String, Opcode>,
    samples: Vec<Sample>,
) -> Vec<(u8, Vec<String>)> {
    samples
        .iter()
        .map(|sample| {
            (
                sample.operation.opcode_id,
                instructions
                    .iter()
                    .filter_map(move |(opcode, function)| {
                        let operation_result = function(
                            &sample.input_registers,
                            sample.operation.in_a,
                            sample.operation.in_b,
                        );
                        if operation_result
                            == sample.output_registers[sample.operation.out as usize]
                        {
                            Some(opcode)
                        } else {
                            None
                        }
                    })
                    .cloned()
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>()
}

fn mapping_opcode_function(
    instructions: &HashMap<String, Opcode>,
    pair_opcode_candidates: Vec<(u8, Vec<String>)>,
) -> Result<HashMap<u8, Opcode>> {
    let mut reduce_opcode_candidates =
        pair_opcode_candidates
            .iter()
            .fold(HashMap::new(), |mut acc, (opcode, new_candidates)| {
                let candidates = acc.entry(*opcode).or_insert_with(HashSet::new);
                if candidates.is_empty() {
                    new_candidates.iter().for_each(|candidate| {
                        candidates.insert(candidate.clone());
                    })
                } else {
                    let intersection = candidates
                        .drain()
                        .filter(|candidate| new_candidates.contains(candidate))
                        .collect::<HashSet<_>>();
                    candidates.clone_from(&intersection);
                }
                acc
            });

    let mut result = HashMap::new();
    loop {
        let unique_opcode = reduce_opcode_candidates
            .iter()
            .filter_map(|(opcode_id, candidates)| {
                if candidates.len() == 1 {
                    Some((*opcode_id, candidates.iter().cloned().next().unwrap()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if unique_opcode.is_empty() {
            break;
        }

        for (opcode_id, function_name) in unique_opcode {
            reduce_opcode_candidates.remove(&opcode_id);
            reduce_opcode_candidates
                .values_mut()
                .for_each(|candidates| {
                    candidates.remove(&function_name);
                });
            result.insert(opcode_id, instructions[&function_name].clone());
        }
    }

    if !reduce_opcode_candidates.is_empty() {
        writeln!(
            io::stderr(),
            "there's ambigu opcode detected, {:?}",
            reduce_opcode_candidates
        )?;
        Err(anyhow!("ambigu detected"))
    } else {
        Ok(result)
    }
}

fn main() -> Result<()> {
    let instructions: HashMap<String, Opcode> = {
        let inner: [(String, Opcode); 16] = [
            (
                "addr".to_string(),
                Rc::new(|registers, in_a, in_b| {
                    registers[in_a as usize].saturating_add(registers[in_b as usize])
                }),
            ),
            (
                "addi".to_string(),
                Rc::new(|registers, in_a, in_b| registers[in_a as usize].saturating_add(in_b as _)),
            ),
            (
                "mulr".to_string(),
                Rc::new(|registers, in_a, in_b| {
                    registers[in_a as usize].saturating_mul(registers[in_b as usize])
                }),
            ),
            (
                "muli".to_string(),
                Rc::new(|registers, in_a, in_b| registers[in_a as usize].saturating_mul(in_b as _)),
            ),
            (
                "banr".to_string(),
                Rc::new(|registers, in_a, in_b| {
                    registers[in_a as usize] & registers[in_b as usize]
                }),
            ),
            (
                "bani".to_string(),
                Rc::new(|registers, in_a, in_b| registers[in_a as usize] & in_b as u64),
            ),
            (
                "borr".to_string(),
                Rc::new(|registers, in_a, in_b| {
                    registers[in_a as usize] | registers[in_b as usize]
                }),
            ),
            (
                "bori".to_string(),
                Rc::new(|registers, in_a, in_b| registers[in_a as usize] | in_b as u64),
            ),
            (
                "setr".to_string(),
                Rc::new(|registers, in_a, _in_b| registers[in_a as usize]),
            ),
            (
                "seti".to_string(),
                Rc::new(|_registers, in_a, _in_b| in_a as _),
            ),
            (
                "gtir".to_string(),
                Rc::new(|registers, in_a, in_b| (in_a as u64 > registers[in_b as usize]) as _),
            ),
            (
                "gtri".to_string(),
                Rc::new(|registers, in_a, in_b| (registers[in_a as usize] > in_b as _) as _),
            ),
            (
                "gtrr".to_string(),
                Rc::new(|registers, in_a, in_b| {
                    (registers[in_a as usize] > registers[in_b as usize]) as _
                }),
            ),
            (
                "eqir".to_string(),
                Rc::new(|registers, in_a, in_b| (in_a as u64 == registers[in_b as usize]) as _),
            ),
            (
                "eqri".to_string(),
                Rc::new(|registers, in_a, in_b| (registers[in_a as usize] == in_b as _) as _),
            ),
            (
                "eqrr".to_string(),
                Rc::new(|registers, in_a, in_b| {
                    (registers[in_a as usize] == registers[in_b as usize]) as _
                }),
            ),
        ];

        inner.iter().cloned().collect()
    };

    let (samples, operations) = extract_input("2018/day-16/input/input.txt")?;
    let pair_opcode_candidates = group_samples(&instructions, samples);

    let count_three_above = pair_opcode_candidates
        .iter()
        .filter(|(_, candidates)| candidates.len() >= 3)
        .count();
    writeln!(
        io::stdout(),
        "sample which behave like three or more opcodes: {}",
        count_three_above
    )?;

    let map_opcodes = mapping_opcode_function(&instructions, pair_opcode_candidates)?;
    let mut registers = [0u64; 4];
    for operation in operations {
        let function = map_opcodes[&operation.opcode_id].clone();
        registers[operation.out as usize] = function(&registers, operation.in_a, operation.in_b);
    }

    writeln!(
        io::stdout(),
        "final result at registry 0 is {}",
        registers[0]
    )?;

    Ok(())
}
