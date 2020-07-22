use anyhow::{anyhow, Context, Result};
use std::{
    collections::{btree_map::Entry, BTreeMap},
    fs,
    io::{self, Write},
};

enum Track {
    Horizontal,
    Vertical,
    Curve1,
    Curve2,
    Intersection,
}

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy)]
struct Cart {
    direction: Direction,
    intersection: u8,
}

impl Cart {
    fn tick(&mut self, location: Location, grid: &[Vec<Option<Track>>]) -> Result<Location> {
        let (old_x, old_y) = (location.x, location.y);
        let (x, y) = match self.direction {
            Direction::Left => (old_x - 1, old_y),
            Direction::Right => (old_x + 1, old_y),
            Direction::Up => (old_x, old_y - 1),
            Direction::Down => (old_x, old_y + 1),
        };

        let (direction, intersection) = match grid[y][x] {
            None => Err(anyhow!("no track to step")),
            Some(Track::Horizontal) => {
                if self.direction == Direction::Left || self.direction == Direction::Right {
                    Ok((self.direction, self.intersection))
                } else {
                    Err(anyhow!("funny combination on horizontal ({}, {})", x, y))
                }
            }
            Some(Track::Vertical) => {
                if self.direction == Direction::Up || self.direction == Direction::Down {
                    Ok((self.direction, self.intersection))
                } else {
                    Err(anyhow!("funny combination on vertical ({}, {})", x, y))
                }
            }
            Some(Track::Curve1) => match self.direction {
                Direction::Left => Ok((Direction::Down, self.intersection)),
                Direction::Right => Ok((Direction::Up, self.intersection)),
                Direction::Up => Ok((Direction::Right, self.intersection)),
                Direction::Down => Ok((Direction::Left, self.intersection)),
            },
            Some(Track::Curve2) => match self.direction {
                Direction::Left => Ok((Direction::Up, self.intersection)),
                Direction::Right => Ok((Direction::Down, self.intersection)),
                Direction::Up => Ok((Direction::Left, self.intersection)),
                Direction::Down => Ok((Direction::Right, self.intersection)),
            },
            Some(Track::Intersection) => match (self.direction, self.intersection) {
                (Direction::Left, 0) => Ok((Direction::Down, 1)),
                (Direction::Right, 0) => Ok((Direction::Up, 1)),
                (Direction::Up, 0) => Ok((Direction::Left, 1)),
                (Direction::Down, 0) => Ok((Direction::Right, 1)),
                (Direction::Left, 1) => Ok((Direction::Left, 2)),
                (Direction::Right, 1) => Ok((Direction::Right, 2)),
                (Direction::Up, 1) => Ok((Direction::Up, 2)),
                (Direction::Down, 1) => Ok((Direction::Down, 2)),
                (Direction::Left, 2) => Ok((Direction::Up, 0)),
                (Direction::Right, 2) => Ok((Direction::Down, 0)),
                (Direction::Up, 2) => Ok((Direction::Right, 0)),
                (Direction::Down, 2) => Ok((Direction::Left, 0)),
                _ => Err(anyhow!("funny combination on intersection ({}, {})", x, y)),
            },
        }?;

        self.direction = direction;
        self.intersection = intersection;
        Ok(Location { x, y })
    }
}

const HORIZON_PASSABLE: &str = "-+/\\";
const VERTICAL_PASSABLE: &str = "|+/\\";

type Grid = Vec<Vec<Option<Track>>>;

#[derive(Eq, PartialEq, Hash, PartialOrd, Ord, Clone, Copy)]
struct Location {
    x: usize,
    y: usize,
}

impl From<(usize, usize)> for Location {
    fn from(value: (usize, usize)) -> Self {
        Location {
            x: value.0,
            y: value.1,
        }
    }
}

fn scan_grids(lines: String) -> Result<(Grid, BTreeMap<Location, Cart>)> {
    let grid_proto = lines
        .lines()
        .map(|line| line.chars().collect::<Vec<char>>())
        .collect::<Vec<_>>();

    let mut grid: Grid = Vec::with_capacity(grid_proto.len());
    let mut grid_carts: BTreeMap<Location, Cart> = BTreeMap::new();

    for (y, line) in (&grid_proto).iter().enumerate() {
        let mut y_grid: Vec<Option<Track>> = Vec::with_capacity(line.len());
        for (x, point) in line.iter().enumerate() {
            let track = match *point {
                ' ' => Ok(None),
                '-' => Ok(Some(Track::Horizontal)),
                '|' => Ok(Some(Track::Vertical)),
                '+' => Ok(Some(Track::Intersection)),
                '/' => Ok(Some(Track::Curve1)),
                '\\' => Ok(Some(Track::Curve2)),
                '<' => {
                    let left_passable = if x == 0 {
                        false
                    } else {
                        HORIZON_PASSABLE.contains(line[x - 1])
                    };
                    let right_passable = if x == line.len() - 1 {
                        false
                    } else {
                        HORIZON_PASSABLE.contains(line[x + 1])
                    };
                    let up_passable = if y == 0 {
                        false
                    } else {
                        VERTICAL_PASSABLE.contains(grid_proto[y - 1][x])
                    };
                    let down_passable = if y == grid_proto.len() - 1 {
                        false
                    } else {
                        VERTICAL_PASSABLE.contains(grid_proto[y + 1][x])
                    };

                    let track = if left_passable && right_passable {
                        match (up_passable, down_passable) {
                            (true, true) => Ok(Some(Track::Intersection)),
                            (false, false) => Ok(Some(Track::Horizontal)),
                            _ => Err(anyhow!("invalid neighbor track combination")),
                        }
                    } else if left_passable {
                        match (up_passable, down_passable) {
                            (true, false) => Ok(Some(Track::Curve1)),
                            (false, true) => Ok(Some(Track::Curve2)),
                            _ => Err(anyhow!("invalid neighbor track combination")),
                        }
                    } else {
                        Err(anyhow!("invalid neighbor track combination"))
                    };

                    if track.is_ok() {
                        grid_carts.insert(
                            Location { x, y },
                            Cart {
                                direction: Direction::Left,
                                intersection: 0,
                            },
                        );
                    }
                    track
                }
                '>' => {
                    let left_passable = if x == 0 {
                        false
                    } else {
                        HORIZON_PASSABLE.contains(line[x - 1])
                    };
                    let right_passable = if x == line.len() - 1 {
                        false
                    } else {
                        HORIZON_PASSABLE.contains(line[x + 1])
                    };
                    let up_passable = if y == 0 {
                        false
                    } else {
                        VERTICAL_PASSABLE.contains(grid_proto[y - 1][x])
                    };
                    let down_passable = if y == grid_proto.len() - 1 {
                        false
                    } else {
                        VERTICAL_PASSABLE.contains(grid_proto[y + 1][x])
                    };

                    let track = if left_passable && right_passable {
                        match (up_passable, down_passable) {
                            (true, true) => Ok(Some(Track::Intersection)),
                            (false, false) => Ok(Some(Track::Horizontal)),
                            _ => Err(anyhow!("invalid neighbor track combination")),
                        }
                    } else if right_passable {
                        match (up_passable, down_passable) {
                            (true, false) => Ok(Some(Track::Curve2)),
                            (false, true) => Ok(Some(Track::Curve1)),
                            _ => Err(anyhow!("invalid neighbor track combination")),
                        }
                    } else {
                        Err(anyhow!("invalid neighbor track combination"))
                    };

                    if track.is_ok() {
                        grid_carts.insert(
                            Location { x, y },
                            Cart {
                                direction: Direction::Right,
                                intersection: 0,
                            },
                        );
                    }
                    track
                }
                '^' => {
                    let left_passable = if x == 0 {
                        false
                    } else {
                        HORIZON_PASSABLE.contains(line[x - 1])
                    };
                    let right_passable = if x == line.len() - 1 {
                        false
                    } else {
                        HORIZON_PASSABLE.contains(line[x + 1])
                    };
                    let up_passable = if y == 0 {
                        false
                    } else {
                        VERTICAL_PASSABLE.contains(grid_proto[y - 1][x])
                    };
                    let down_passable = if y == grid_proto.len() - 1 {
                        false
                    } else {
                        VERTICAL_PASSABLE.contains(grid_proto[y + 1][x])
                    };

                    let track = if up_passable && down_passable {
                        match (left_passable, right_passable) {
                            (true, true) => Ok(Some(Track::Intersection)),
                            (false, false) => Ok(Some(Track::Vertical)),
                            _ => Err(anyhow!("invalid neighbor track combination")),
                        }
                    } else if up_passable {
                        match (left_passable, right_passable) {
                            (true, false) => Ok(Some(Track::Curve1)),
                            (false, true) => Ok(Some(Track::Curve2)),
                            _ => Err(anyhow!("invalid neighbor track combination")),
                        }
                    } else {
                        Err(anyhow!("invalid neighbor track combination"))
                    };

                    if track.is_ok() {
                        grid_carts.insert(
                            Location { x, y },
                            Cart {
                                direction: Direction::Up,
                                intersection: 0,
                            },
                        );
                    }
                    track
                }
                'v' => {
                    let left_passable = if x == 0 {
                        false
                    } else {
                        HORIZON_PASSABLE.contains(line[x - 1])
                    };
                    let right_passable = if x == line.len() - 1 {
                        false
                    } else {
                        HORIZON_PASSABLE.contains(line[x + 1])
                    };
                    let up_passable = if y == 0 {
                        false
                    } else {
                        VERTICAL_PASSABLE.contains(grid_proto[y - 1][x])
                    };
                    let down_passable = if y == grid_proto.len() - 1 {
                        false
                    } else {
                        VERTICAL_PASSABLE.contains(grid_proto[y + 1][x])
                    };

                    let track = if up_passable && down_passable {
                        match (left_passable, right_passable) {
                            (true, true) => Ok(Some(Track::Intersection)),
                            (false, false) => Ok(Some(Track::Vertical)),
                            _ => Err(anyhow!("invalid neighbor track combination")),
                        }
                    } else if down_passable {
                        match (left_passable, right_passable) {
                            (true, false) => Ok(Some(Track::Curve2)),
                            (false, true) => Ok(Some(Track::Curve1)),
                            _ => Err(anyhow!("invalid neighbor track combination")),
                        }
                    } else {
                        Err(anyhow!("invalid neighbor track combination"))
                    };

                    if track.is_ok() {
                        grid_carts.insert(
                            Location { x, y },
                            Cart {
                                direction: Direction::Down,
                                intersection: 0,
                            },
                        );
                    }
                    track
                }
                _ => Err(anyhow!("unrecognized track")),
            }?;
            y_grid.insert(x, track);
        }
        grid.insert(y, y_grid);
    }
    Ok((grid, grid_carts))
}

fn tick(
    grid: &[Vec<Option<Track>>],
    grid_carts: &mut BTreeMap<Location, Cart>,
) -> Result<Vec<Location>> {
    let mut crash_locations = Vec::new();
    let cart_locations = grid_carts.keys().cloned().collect::<Vec<_>>();

    for location in cart_locations {
        if let Entry::Occupied(entry) = grid_carts.entry(location) {
            let (_, mut cart) = entry.remove_entry();
            let new_location = cart.tick(location, grid)?;

            match grid_carts.entry(new_location) {
                Entry::Occupied(entry) => {
                    entry.remove_entry();
                    crash_locations.push(new_location);
                }
                Entry::Vacant(entry) => {
                    entry.insert(cart);
                }
            }
        };
    }
    Ok(crash_locations)
}

fn main() -> Result<()> {
    let content =
        fs::read_to_string("2018/day-13/input/input.txt").context("failed to read input file")?;
    let (grid, mut grid_carts) = scan_grids(content)?;

    loop {
        let crashes = tick(&grid, &mut grid_carts)?;
        for crash in crashes {
            writeln!(io::stdout(), "crash happened at ({}, {})", crash.x, crash.y)?;
        }

        match grid_carts.len() {
            0 => {
                writeln!(io::stdout(), "no survivor")?;
                break;
            }
            1 => {
                let last_location = grid_carts.keys().next().unwrap();
                writeln!(
                    io::stdout(),
                    "sole survivor, last position at ({}, {})",
                    last_location.x,
                    last_location.y
                )?;
                break;
            }
            _ => (),
        }
    }

    Ok(())
}
