#[macro_use]
extern crate lazy_static;

use regex::Regex;
use std::{
    collections::HashMap,
    fmt,
    fs::File,
    io::{self, BufRead, BufReader, Error, Write},
    num::ParseIntError,
    str::FromStr,
};

#[derive(Debug)]
enum ClaimError {
    ParseInt(ParseIntError),
    Custom(&'static str),
}

impl std::error::Error for ClaimError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            ClaimError::ParseInt(ref err) => Some(err),
            ClaimError::Custom(_) => None,
        }
    }
}

impl fmt::Display for ClaimError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ClaimError::ParseInt(ref err) => write!(f, "Failed to parse, caused by {:?}", err),
            ClaimError::Custom(msg) => write!(f, "error: {}", msg),
        }
    }
}

impl From<ParseIntError> for ClaimError {
    fn from(error: ParseIntError) -> Self {
        ClaimError::ParseInt(error)
    }
}

struct Claim {
    id: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

struct IterPoints<'claim> {
    claim: &'claim Claim,
    curr_x: u32,
    curr_y: u32,
}

impl<'claim> Iterator for IterPoints<'claim> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_x >= self.claim.x + self.claim.width {
            self.curr_x = self.claim.x;
            self.curr_y += 1;
        }
        if self.curr_y >= self.claim.y + self.claim.height {
            None
        } else {
            let result = Some((self.curr_x, self.curr_y));
            self.curr_x += 1;
            result
        }
    }
}

impl Claim {
    fn points(&self) -> IterPoints {
        IterPoints {
            claim: self,
            curr_x: self.x,
            curr_y: self.y,
        }
    }
}

impl FromStr for Claim {
    type Err = ClaimError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?x)
                \#
                (?P<id>[0-9]+)
                \s+@\s+
                (?P<x>[0-9]+),(?P<y>[0-9]+)
                :\s+
                (?P<width>[0-9]+)x(?P<height>[0-9]+)"
            )
            .unwrap();
        }

        if let Some(capture) = RE.captures(s) {
            Ok(Claim {
                id: capture["id"].parse()?,
                x: capture["x"].parse()?,
                y: capture["y"].parse()?,
                width: capture["width"].parse()?,
                height: capture["height"].parse()?,
            })
        } else {
            Err(ClaimError::Custom("unrecognized claim"))
        }
    }
}

type Grid = HashMap<(u32, u32), u32>;

fn calculate_grid(claims: &[Claim]) -> Grid {
    let mut grid = HashMap::new();
    for claim in claims {
        for point in claim.points() {
            *grid.entry(point).or_default() += 1;
        }
    }

    grid
}

fn calculte_overlap_area(grid: &Grid) -> usize {
    grid.values().filter(|&&v| v > 1).count()
}

fn find_first_non_overlap_claim<'claim, 'grid>(
    claims: &'claim [Claim],
    grid: &'grid Grid,
) -> Option<&'claim Claim> {
    for claim in claims {
        if claim.points().all(|point| grid[&point] == 1) {
            return Some(claim);
        }
    }
    None
}

fn main() -> Result<(), Error> {
    let file = File::open("input/input.txt")?;
    let reader = BufReader::new(file);
    let claims = reader
        .lines()
        .filter_map(|line| line.ok().and_then(|s| s.parse::<Claim>().ok()))
        .collect::<Vec<_>>();
    let grid = calculate_grid(&claims);

    let overlap_area = calculte_overlap_area(&grid);
    writeln!(io::stdout(), "overlap area: {}", overlap_area)?;
    if let Some(claim) = find_first_non_overlap_claim(&claims, &grid) {
        writeln!(io::stdout(), "first non overlap claim found: {}", claim.id)?;
    } else {
        writeln!(io::stdout(), "no overlap claim found")?;
    }

    Ok(())
}
