use anyhow::{Context, Result};
use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

fn acc_freq_changes(freq_changes: &[i64]) -> i64 {
    freq_changes.iter().sum()
}

fn first_repeat_acc_freq_changes(freq_changes: &[i64]) -> i64 {
    let mut accumulates: HashSet<i64> = HashSet::new();
    let mut run_acc = 0i64;
    'result: loop {
        for freq_change in freq_changes {
            run_acc += freq_change;
            if accumulates.contains(&run_acc) {
                break 'result run_acc;
            } else {
                accumulates.insert(run_acc);
            }
        }
    }
}

fn main() -> Result<()> {
    let file = File::open("2018/day-01/input/input.txt").context("failed to read input file")?;
    let reader = BufReader::new(file);
    let freq_changes = reader
        .lines()
        .filter_map(|line| line.ok().and_then(|s| s.parse::<i64>().ok()))
        .collect::<Vec<_>>();

    writeln!(
        io::stdout(),
        "accumulate freq: {}",
        acc_freq_changes(&freq_changes)
    )?;
    writeln!(
        io::stdout(),
        "first repeat accumulate freq: {}",
        first_repeat_acc_freq_changes(&freq_changes)
    )?;
    Ok(())
}
