#[macro_use]
extern crate lazy_static;

use anyhow::{anyhow, Context, Error, Result};
use regex::Regex;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::Path,
    str::FromStr,
};

type Grid = Vec<Vec<u8>>;

#[derive(Eq, PartialEq, PartialOrd, Ord)]
struct Coordinate {
    x: usize,
    y: usize,
}

struct RangeCoordinateIter {
    x_start: usize,
    x_end: usize,
    y_end: usize,
    curr_x: usize,
    curr_y: usize,
}

impl Iterator for RangeCoordinateIter {
    type Item = Coordinate;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_x > self.x_end {
            if self.curr_y >= self.y_end {
                None
            } else {
                self.curr_y += 1;
                self.curr_x = self.x_start + 1;
                Some(Coordinate {
                    x: self.x_start,
                    y: self.curr_y,
                })
            }
        } else {
            let x = self.curr_x;
            self.curr_x += 1;
            Some(Coordinate { x, y: self.curr_y })
        }
    }
}

#[derive(Clone)]
struct RangeCoordinates {
    x_start: usize,
    x_end: usize,
    y_start: usize,
    y_end: usize,
}

struct EdgeState {
    x: usize,
    open: bool,
}

fn parse_range<S: Into<String>>(input: S) -> Result<(usize, usize)> {
    let input = input.into();
    if input.contains("..") {
        let range: Vec<&str> = input.split("..").collect();
        if range.len() != 2 {
            return Err(anyhow!("unrecognized range pattern: {}", input));
        }
        Ok((range[0].parse::<usize>()?, range[1].parse::<usize>()?))
    } else {
        let single = input.parse::<usize>()?;
        Ok((single, single))
    }
}

impl FromStr for RangeCoordinates {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref COORDINATE_REGEX: Regex = Regex::new(
                r"(?x)
                (?:
                    (?:
                        x=(?P<x>[0-9]+(?:\.{2}[0-9]+)?)|
                        y=(?P<y>[0-9]+(?:\.{2}[0-9]+)?)
                    )
                    (?:,\s*)?
                ){2}"
            )
            .unwrap();
            static ref RANGE_REGEX: Regex =
                Regex::new(r"(?P<start>[0..9]+)\.{2}(?P<end>[0..9]+)").unwrap();
        }

        if let Some(capture) = COORDINATE_REGEX.captures(&s) {
            let (x_start, x_end) = parse_range(capture["x"].parse::<String>()?)?;
            let (y_start, y_end) = parse_range(capture["y"].parse::<String>()?)?;
            Ok(RangeCoordinates {
                x_start,
                x_end,
                y_start,
                y_end,
            })
        } else {
            Err(anyhow!("unrecognized coordinate pattern"))
        }
    }
}

impl RangeCoordinates {
    fn iter(&self) -> RangeCoordinateIter {
        RangeCoordinateIter {
            x_start: self.x_start,
            x_end: self.x_end,
            y_end: self.y_end,
            curr_x: self.x_start,
            curr_y: self.y_start,
        }
    }
}

const CLAY_WALL: u8 = b'#';
const SAND_SPACE: u8 = b'.';
const WATER_FALL: u8 = b'|';
const WATER_FILL: u8 = b'~';

fn render_input<P: AsRef<Path>>(path: P) -> Result<Grid> {
    let file = File::open(path).context("failed to read input file")?;
    let reader = BufReader::new(file);

    let clay_coordinates = reader
        .lines()
        .filter_map(|line| line.ok().and_then(|s| s.parse::<RangeCoordinates>().ok()))
        .flat_map(|range_coordinates| range_coordinates.iter())
        .collect::<Vec<_>>();
    let farthest_coordinate = clay_coordinates.iter().max().unwrap();

    let mut result = vec![vec![SAND_SPACE; farthest_coordinate.x + 2]; farthest_coordinate.y + 1];
    clay_coordinates.iter().for_each(|c| {
        result[c.y][c.x] = CLAY_WALL;
    });
    result[0][500] = b'+';
    Ok(result)
}

fn water_fill(grid: &mut Grid, source: Coordinate) -> Result<bool> {
    let x = source.x;
    let y = source.y;
    let y_upper = grid.len() - 1;
    let x_upper = grid[0].len() - 1;

    if y >= y_upper {
        Ok(false)
    } else {
        let left = if x == 0 {
            EdgeState { x, open: true }
        } else {
            let mut x_left = x - 1;
            loop {
                while x_left > 0
                    && grid[y][x_left] == SAND_SPACE
                    && (grid[y + 1][x_left] == CLAY_WALL || grid[y + 1][x_left] == WATER_FILL)
                {
                    x_left -= 1;
                }
                if x_left == 0 {
                    if grid[y + 1][x_left] == SAND_SPACE {
                        water_fall(grid, Coordinate { x: x_left, y })?;
                    }
                    break EdgeState {
                        x: x_left,
                        open: true,
                    };
                } else {
                    match grid[y][x_left] {
                        SAND_SPACE => {
                            water_fall(grid, Coordinate { x: x_left, y })?;
                            if grid[y + 1][x_left] == WATER_FALL {
                                break EdgeState {
                                    x: x_left,
                                    open: true,
                                };
                            }
                        }
                        CLAY_WALL => {
                            break EdgeState {
                                x: x_left + 1,
                                open: false,
                            };
                        }
                        WATER_FALL => {
                            break EdgeState {
                                x: x_left + 1,
                                open: true,
                            };
                        }
                        border => {
                            return Err(anyhow!(
                                "unexpected left border: {} @({}, {})",
                                border as char,
                                x_left,
                                y
                            ))
                        }
                    }
                }
            }
        };
        let right = if x == x_upper {
            EdgeState { x, open: true }
        } else {
            let mut x_right = x + 1;
            loop {
                while x_right < x_upper
                    && grid[y][x_right] == SAND_SPACE
                    && (grid[y + 1][x_right] == CLAY_WALL || grid[y + 1][x_right] == WATER_FILL)
                {
                    x_right += 1;
                }
                if x_right == x_upper {
                    if grid[y + 1][x_right] == SAND_SPACE {
                        water_fall(grid, Coordinate { x: x_right, y })?;
                    }
                    break EdgeState {
                        x: x_right,
                        open: true,
                    };
                } else {
                    match grid[y][x_right] {
                        SAND_SPACE => {
                            water_fall(grid, Coordinate { x: x_right, y })?;
                            if grid[y + 1][x_right] == WATER_FALL {
                                break EdgeState {
                                    x: x_right,
                                    open: true,
                                };
                            }
                        }
                        CLAY_WALL => {
                            break EdgeState {
                                x: x_right - 1,
                                open: false,
                            };
                        }
                        WATER_FALL => {
                            break EdgeState {
                                x: x_right - 1,
                                open: true,
                            };
                        }
                        border => {
                            return Err(anyhow!(
                                "unexpected right border: {} @({}, {})",
                                border as char,
                                x_right,
                                y
                            ))
                        }
                    }
                }
            }
        };

        if !left.open && !right.open {
            (left.x..=right.x).for_each(|x| grid[y][x] = WATER_FILL);
            Ok(true)
        } else {
            (left.x..=right.x).for_each(|x| grid[y][x] = WATER_FALL);
            Ok(false)
        }
    }
}

fn water_fall(grid: &mut Grid, source: Coordinate) -> Result<()> {
    let x = source.x;
    let y_lower = source.y + 1;
    let y_upper = {
        if grid.is_empty() {
            Err(anyhow!("empty grids"))
        } else {
            Ok(grid.len() - 1)
        }
    }?;

    // water fall
    let mut y = y_lower;
    loop {
        match grid[y][x] {
            SAND_SPACE => grid[y][x] = WATER_FALL,
            WATER_FILL | CLAY_WALL => break Ok(()),
            WATER_FALL => return Ok(()),
            terrain => {
                break Err(anyhow!(
                    "unexpected terrain: {} @({}, {})",
                    terrain as char,
                    x,
                    y
                ))
            }
        }
        if y < y_upper {
            y += 1;
        } else {
            return Ok(());
        }
    }?;

    // water fill
    y -= 1;
    loop {
        if y < y_lower {
            break;
        }
        if water_fill(grid, Coordinate { x, y })? {
            y -= 1;
        } else {
            break;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let mut grid = render_input("2018/day-17/input/input.txt")?;
    let y_lower = grid
        .iter()
        .enumerate()
        .filter_map(|(idx, line)| {
            if line.contains(&CLAY_WALL) {
                Some(idx)
            } else {
                None
            }
        })
        .min()
        .unwrap();
    water_fall(&mut grid, Coordinate { x: 500, y: 0 })?;

    let water_coverage = grid
        .iter()
        .skip(y_lower)
        .map(|line| {
            line.iter()
                .filter(|v| **v == WATER_FALL || **v == WATER_FILL)
                .count()
        })
        .sum::<usize>();
    writeln!(io::stdout(), "water coverage is {}", water_coverage)?;

    let water_left_coverage = grid
        .iter()
        .map(|line| line.iter().filter(|v| **v == WATER_FILL).count())
        .sum::<usize>();
    writeln!(
        io::stdout(),
        "water left coverage is {}",
        water_left_coverage
    )?;

    Ok(())
}
