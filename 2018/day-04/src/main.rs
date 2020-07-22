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

#[derive(Eq, PartialEq, Ord, PartialOrd)]
struct DateTime {
    year: u32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
}

type GuardID = u32;

enum EventKind {
    StartShift { guard_id: GuardID },
    Asleep,
    Wakeup,
}

struct GuardEvent {
    datetime: DateTime,
    kind: EventKind,
}

impl FromStr for GuardEvent {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?x)
                \[
                    (?P<year>[0-9]{4})-(?P<month>[0-9]{2})-(?P<day>[0-9]{2})
                    \s+
                    (?P<hour>[0-9]{2}):(?P<minute>[0-9]{2})
                \]
                \s+
                (:?Guard\ \#(?P<id>[0-9]+)\ begins\ shift|(?P<action>.+))"
            )
            .unwrap();
        }

        if let Some(capture) = RE.captures(s) {
            let datetime = DateTime {
                year: capture["year"].parse()?,
                month: capture["month"].parse()?,
                day: capture["day"].parse()?,
                hour: capture["hour"].parse()?,
                minute: capture["minute"].parse()?,
            };
            let kind = {
                if let Some(id) = capture.name("id") {
                    EventKind::StartShift {
                        guard_id: id.as_str().parse()?,
                    }
                } else if &capture["action"] == "falls asleep" {
                    EventKind::Asleep
                } else if &capture["action"] == "wakes up" {
                    EventKind::Wakeup
                } else {
                    return Err(anyhow!("could not determine event kind"));
                }
            };

            Ok(GuardEvent { datetime, kind })
        } else {
            Err(anyhow!("unrecognized event"))
        }
    }
}

fn aggregate_minutes_sleep_per_guard(
    guard_events: &[GuardEvent],
) -> Result<HashMap<GuardID, [u32; 60]>> {
    let mut aggregates = HashMap::new();

    let mut current_guard = None;
    let mut current_asleep = None;
    for event in guard_events {
        match event.kind {
            EventKind::StartShift { guard_id } => current_guard = Some(guard_id),
            EventKind::Asleep => {
                if current_guard.is_none() {
                    return Err(anyhow!("unordered event"));
                }
                current_asleep = Some(event.datetime.minute);
            }
            EventKind::Wakeup => {
                if current_guard.is_none() || current_asleep.is_none() {
                    return Err(anyhow!("unordered event"));
                }
                let guard_id = current_guard.unwrap();
                let asleep = current_asleep.unwrap();
                let wakeup = event.datetime.minute;
                let freq_sleep_minutes = aggregates.entry(guard_id).or_insert([0; 60]);

                if wakeup < asleep {
                    for minute in asleep..=59 {
                        freq_sleep_minutes[minute as usize] += 1;
                    }

                    for minute in 0..wakeup {
                        freq_sleep_minutes[minute as usize] += 1;
                    }
                } else {
                    for minute in asleep..wakeup {
                        freq_sleep_minutes[minute as usize] += 1;
                    }
                }
            }
        }
    }

    Ok(aggregates)
}

fn find_most_sleep_guard(aggregates: &HashMap<GuardID, [u32; 60]>) -> Option<&GuardID> {
    aggregates
        .iter()
        .map(|(guard_id, freq_sleep_minutes)| {
            (guard_id, freq_sleep_minutes.iter().cloned().sum::<u32>())
        })
        .max_by(|(_, tot_minutes1), (_, tot_minutes2)| tot_minutes1.cmp(tot_minutes2))
        .map(|(guard_id, _)| guard_id)
}

fn find_most_minute_sleep(freqs: &[u32; 60]) -> Option<usize> {
    freqs
        .iter()
        .enumerate()
        .max_by(|(_, freq_sleep1), (_, freq_sleep2)| freq_sleep1.cmp(freq_sleep2))
        .map(|(minute, _)| minute)
}

fn find_most_sleep_minute_for_most_sleep_guard(
    aggregates: &HashMap<GuardID, [u32; 60]>,
) -> Result<(GuardID, usize)> {
    if let Some(guard_id) = find_most_sleep_guard(aggregates) {
        if let Some(freqs) = aggregates.get(guard_id) {
            if let Some(minute) = find_most_minute_sleep(freqs) {
                Ok((*guard_id, minute))
            } else {
                Err(anyhow!("can't find most minute sleep"))
            }
        } else {
            Err(anyhow!("can't find sleep freqs"))
        }
    } else {
        Err(anyhow!("can't find most sleepy guard"))
    }
}

fn find_most_sleep_minute_guard(
    aggregates: &HashMap<GuardID, [u32; 60]>,
) -> Result<(GuardID, usize)> {
    if let Some(((guard_id, minute), _)) = aggregates
        .iter()
        .flat_map(|(guard_id, freq_sleep_minutes)| {
            freq_sleep_minutes
                .iter()
                .enumerate()
                .max_by(|(_, freq_sleep1), (_, freq_sleep2)| freq_sleep1.cmp(freq_sleep2))
                .map(|(minute, freq_sleep)| ((guard_id, minute), *freq_sleep))
        })
        .max_by(|(_, max_freq_sleep1), (_, max_freq_sleep2)| max_freq_sleep1.cmp(max_freq_sleep2))
    {
        Ok((*guard_id, minute))
    } else {
        Err(anyhow!("can't find max freq sleep by minutes",))
    }
}

fn main() -> Result<()> {
    let file = File::open("2018/day-04/input/input.txt").context("failed to read input file")?;
    let reader = BufReader::new(file);

    let mut guard_events = reader
        .lines()
        .filter_map(|line| line.ok().and_then(|s| s.parse::<GuardEvent>().ok()))
        .collect::<Vec<_>>();
    guard_events.sort_by(|ev1, ev2| ev1.datetime.cmp(&ev2.datetime));
    let aggregates = aggregate_minutes_sleep_per_guard(&guard_events)?;

    let (guard_id, minute) = find_most_sleep_minute_for_most_sleep_guard(&aggregates)?;
    writeln!(
        io::stdout(),
        "most minute guard for most sleepy guard: guard_id x minute => {} x {} = {}",
        guard_id,
        minute,
        guard_id * (minute as u32)
    )?;
    let (guard_id, minute) = find_most_sleep_minute_guard(&aggregates)?;
    writeln!(
        io::stdout(),
        "most sleep minute guard: guard_id x minute => {} x {} = {}",
        guard_id,
        minute,
        guard_id * (minute as u32)
    )?;

    Ok(())
}
