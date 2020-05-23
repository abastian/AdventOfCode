use std::{
    collections::HashSet,
    fmt,
    fs::File,
    io::{self, BufRead, BufReader, Error as IOError, Write},
};

#[derive(Debug)]
enum Error {
    IO(IOError),
}

impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        Error::IO(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::IO(ref err) => err.fmt(f),
        }
    }
}

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

fn main() -> Result<(), Error> {
    let file = File::open("input/input.txt")?;
    let reader = BufReader::new(file);
    let freq_changes = reader
        .lines()
        .filter_map(|line| line.ok().and_then(|s| s.parse::<i64>().ok()))
        .collect::<Vec<_>>();

    let acc_freq = acc_freq_changes(&freq_changes);
    let first_repeat_freq = first_repeat_acc_freq_changes(&freq_changes);

    writeln!(io::stdout(), "accumulate freq: {}", acc_freq)?;
    writeln!(
        io::stdout(),
        "first repeat accumulate freq: {}",
        first_repeat_freq
    )?;
    Ok(())
}
