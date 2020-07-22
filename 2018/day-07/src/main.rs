#[macro_use]
extern crate lazy_static;

use anyhow::{anyhow, Context, Error, Result};
use regex::Regex;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    str::FromStr,
};

type Step = char;

struct Dependency {
    predecessor: Step,
    successor: Step,
}

struct DAG {
    ready_to_run: Vec<Step>,
    prerequisites: HashMap<Step, Vec<Step>>,
    next_steps: HashMap<Step, Vec<Step>>,
}

impl DAG {
    fn new(dependencies: &[Dependency]) -> Self {
        let mut prerequisites = HashMap::with_capacity(25);
        let mut next_steps = HashMap::with_capacity(25);
        dependencies.iter().for_each(|dep| {
            prerequisites
                .entry(dep.successor)
                .or_insert_with(Vec::new)
                .push(dep.predecessor);
            next_steps
                .entry(dep.predecessor)
                .or_insert_with(Vec::new)
                .push(dep.successor);
        });

        let ready_to_run = next_steps
            .keys()
            .filter(|step| !prerequisites.contains_key(step))
            .cloned()
            .collect::<Vec<_>>();

        DAG {
            prerequisites,
            next_steps,
            ready_to_run,
        }
    }

    fn run_sequences(&self, worker: usize) -> (String, u64) {
        let mut sequences = String::with_capacity(25);
        let mut timer = 0u64;
        let mut ready_to_run = self.ready_to_run.clone();
        let next_steps = &self.next_steps;
        let mut prerequisites = self.prerequisites.clone();
        let mut available_workers = (0..worker).collect::<Vec<_>>();
        let mut occupied_workers = HashMap::new();

        loop {
            ready_to_run.sort_by(|t1, t2| t2.cmp(t1));

            while !ready_to_run.is_empty() && !available_workers.is_empty() {
                let ready_step = ready_to_run.pop().unwrap();
                let completion_time = 60 + (ready_step as u8 - b'A' + 1) as u64;
                let worker = available_workers.pop().unwrap();

                occupied_workers.insert(worker, (completion_time, ready_step));
            }

            if let Some(wait_time) = occupied_workers
                .values()
                .filter(|(t, _)| *t > 0)
                .map(|(t, _)| *t)
                .min()
            {
                let mut finished_steps = vec![];
                occupied_workers.iter_mut().for_each(|(w, (t, s))| {
                    if *t == wait_time {
                        available_workers.push(*w);
                        finished_steps.push(*s);
                    } else {
                        *t -= wait_time;
                    }
                });
                for worker in &available_workers {
                    occupied_workers.remove(&worker);
                }
                finished_steps.sort();
                for finish_step in finished_steps {
                    if let Some(steps) = next_steps.get(&finish_step) {
                        steps.iter().for_each(|next_step| {
                            let complete =
                                if let Some(prerequisite) = prerequisites.get_mut(next_step) {
                                    let pos = prerequisite
                                        .iter()
                                        .enumerate()
                                        .filter(|(_, item)| **item == finish_step)
                                        .map(|(pos, _)| pos)
                                        .next()
                                        .unwrap();
                                    prerequisite.remove(pos);
                                    prerequisite.is_empty()
                                } else {
                                    false
                                };
                            if complete {
                                ready_to_run.push(*next_step);
                            }
                        });
                    }
                    sequences.push(finish_step);
                }
                timer += wait_time;
            }

            if ready_to_run.is_empty() && occupied_workers.is_empty() {
                break;
            }
        }

        (sequences, timer)
    }
}

impl FromStr for Dependency {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"Step (?P<predecessor>[A-Z]) must be finished before step (?P<successor>[A-Z]) can begin."
            )
            .unwrap();
        }

        if let Some(capture) = RE.captures(s) {
            Ok(Dependency {
                predecessor: capture["predecessor"].parse()?,
                successor: capture["successor"].parse()?,
            })
        } else {
            Err(anyhow!("unrecognized step requirement"))
        }
    }
}

fn main() -> Result<()> {
    let file = File::open("2018/day-07/input/input.txt").context("failed to read input file")?;
    let reader = BufReader::new(file);

    let dependencies = reader
        .lines()
        .filter_map(|line| line.ok().and_then(|s| s.parse::<Dependency>().ok()))
        .collect::<Vec<_>>();
    let dag = DAG::new(&dependencies);

    let run_sequences = dag.run_sequences(1);
    writeln!(
        io::stdout(),
        "sequences of task with 1 worker: {}, takes {} time unit",
        run_sequences.0,
        run_sequences.1
    )?;

    let run_sequences_5worker = dag.run_sequences(5);
    writeln!(
        io::stdout(),
        "sequences of task with 5 worker: {}, takes {} time unit",
        run_sequences_5worker.0,
        run_sequences_5worker.1
    )?;

    Ok(())
}
