#[macro_use]
extern crate lazy_static;

use anyhow::{anyhow, Context, Error, Result};
use regex::Regex;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
    str::FromStr,
};

struct PointChange {
    initial_x: i32,
    initial_y: i32,
    velocity_x: i32,
    velocity_y: i32,
}

impl FromStr for PointChange {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?x)
                position=<\s*
                    (?P<initial_x>-?[0-9]+)
                    \s*,\s*
                    (?P<initial_y>-?[0-9]+)
                >\s*velocity=<\s*
                    (?P<velocity_x>-?[0-9]+)
                    \s*,\s*
                    (?P<velocity_y>-?[0-9]+)
                >"
            )
            .unwrap();
        }

        if let Some(capture) = RE.captures(s) {
            Ok(PointChange {
                initial_x: capture["initial_x"].parse()?,
                initial_y: capture["initial_y"].parse()?,
                velocity_x: capture["velocity_x"].parse()?,
                velocity_y: capture["velocity_y"].parse()?,
            })
        } else {
            Err(anyhow!("unrecognized point changes"))
        }
    }
}

struct Point {
    x: i32,
    y: i32,
}

fn calculate_points(point_changes: &[PointChange], sec: u32) -> Vec<Point> {
    point_changes
        .iter()
        .map(|pc| Point {
            x: pc.initial_x + (pc.velocity_x * sec as i32),
            y: pc.initial_y + (pc.velocity_y * sec as i32),
        })
        .collect()
}

fn render_points(points: &[Point]) -> Option<String> {
    let min_x = points.iter().map(|p| p.x).min().unwrap();
    let max_x = points.iter().map(|p| p.x).max().unwrap();
    let min_y = points.iter().map(|p| p.y).min().unwrap();
    let max_y = points.iter().map(|p| p.y).max().unwrap();

    if (max_x - min_x) < 120 && (max_y - min_y) < 20 {
        let mut grid = vec![vec![b'.'; 120]; 20];
        points
            .iter()
            .map(|p| ((p.x - min_x) as usize, (p.y - min_y) as usize))
            .for_each(|(x, y)| grid[y][x] = b'#');
        let mut grid_str = String::with_capacity(121 * 20);
        grid.iter()
            .map(|v| String::from_utf8_lossy(&v))
            .for_each(|s| {
                grid_str.push_str(&s);
                grid_str.push('\n');
            });
        Some(grid_str)
    } else {
        None
    }
}

fn main() -> Result<()> {
    let file = File::open("2018/day-10/input/input.txt").context("failed to read input file")?;
    let reader = BufReader::new(file);

    let point_changes = reader
        .lines()
        .filter_map(|line| line.ok().and_then(|s| s.parse::<PointChange>().ok()))
        .collect::<Vec<_>>();

    let mut sec = 0u32;
    let mut in_range = false;
    while !in_range {
        let points = calculate_points(&point_changes, sec);
        if let Some(grid) = render_points(&points) {
            in_range = true;
            writeln!(io::stdout(), "grid at {} second(s)", sec)?;
            writeln!(io::stdout(), "{}", grid)?;
            writeln!(io::stdout(), "{}", "~".repeat(120))?;
        }
        sec += 1;
    }
    loop {
        let points = calculate_points(&point_changes, sec);
        if let Some(grid) = render_points(&points) {
            writeln!(io::stdout(), "grid at {} second(s)", sec)?;
            writeln!(io::stdout(), "{}", grid)?;
            writeln!(io::stdout(), "{}", "~".repeat(120))?;
        } else {
            break;
        }
        sec += 1;
    }
    Ok(())
}
