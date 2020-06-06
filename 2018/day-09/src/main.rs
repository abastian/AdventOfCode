use anyhow::{Context, Result};
use regex::Regex;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

fn calculate_highest_point(num_player: usize, last_point: u32) -> u64 {
    let mut points: Vec<u64> = vec![0; num_player];
    let mut game_arena = Vec::with_capacity(last_point as usize);
    game_arena.push(0);
    let mut current = 0usize;
    for marble in 1..=last_point {
        if (marble % 23) > 0 {
            current = ((current + 1) % game_arena.len()) + 1;
            game_arena.insert(current, marble);
        } else {
            let player_idx = marble as usize % num_player;
            current = (current + game_arena.len() - 7) % game_arena.len();
            points[player_idx] += (marble + game_arena.remove(current)) as u64;
        }
    }
    *points.iter().max().unwrap()
}

fn main() -> Result<()> {
    let file = File::open("input/input.txt").context("failed to read input file")?;
    let reader = BufReader::new(file);

    if let Some(s) = reader.lines().filter_map(|line| line.ok()).next() {
        let re = Regex::new(
            "(?P<players>[0-9]+) players; last marble is worth (?P<points>[0-9]+) points",
        )
        .unwrap();

        if let Some(capture) = re.captures(&s) {
            let num_player: usize = capture["players"].parse()?;
            let last_point: u32 = capture["points"].parse()?;

            let highest_point = calculate_highest_point(num_player, last_point);
            writeln!(
                io::stdout(),
                "highest point with {} players and last marble worth {} points is {}",
                num_player,
                last_point,
                highest_point
            )?;

            let highest_point = calculate_highest_point(num_player, 100 * last_point);
            writeln!(
                io::stdout(),
                "highest point with {} players and last marble worth 100 * {} points is {}",
                num_player,
                last_point,
                highest_point
            )?;
        }
    }
    Ok(())
}
