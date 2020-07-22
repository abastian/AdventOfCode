use anyhow::{anyhow, Context, Error, Result};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    hash::Hash,
    io::{self, BufRead, BufReader, Write},
    str::FromStr,
};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct Coordinate {
    x: i32,
    y: i32,
}

impl FromStr for Coordinate {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let xy = s.split(',').collect::<Vec<_>>();
        if xy.len() != 2 {
            Err(anyhow!("unparsable coordinate"))
        } else {
            Ok(Coordinate {
                x: xy[0].trim().parse()?,
                y: xy[1].trim().parse()?,
            })
        }
    }
}

impl Coordinate {
    fn find_center(coordinates: &[Coordinate]) -> Coordinate {
        let (x, y, cnt) = coordinates.iter().fold((0, 0, 0), |acc, coordinate| {
            let (mut x, mut y, mut cnt) = acc;
            x += coordinate.x;
            y += coordinate.y;
            cnt += 1;
            (x, y, cnt)
        });
        Coordinate {
            x: x / cnt,
            y: y / cnt,
        }
    }

    fn calculate_manhattan_length(&self, other: &Coordinate) -> u32 {
        ((self.x - other.x).abs() as u32) + ((self.y - other.y).abs() as u32)
    }

    fn calculate_longest_manhattant_distant(&self, coordinates: &[Coordinate]) -> Option<u32> {
        coordinates
            .iter()
            .map(|coordinate| self.calculate_manhattan_length(coordinate))
            .max()
    }

    fn calculate_coordinates_within_manhattan_radius(&self, radius: u32) -> Vec<Coordinate> {
        if radius == 0 {
            vec![self.clone()]
        } else {
            let mut list = Vec::with_capacity((4 * radius) as usize);
            list.push(Coordinate {
                x: self.x - radius as i32,
                y: self.y,
            });
            list.push(Coordinate {
                x: self.x + radius as i32,
                y: self.y,
            });
            list.push(Coordinate {
                x: self.x,
                y: self.y - radius as i32,
            });
            list.push(Coordinate {
                x: self.x,
                y: self.y + radius as i32,
            });
            for i in 1..radius {
                list.push(Coordinate {
                    x: self.x - i as i32,
                    y: self.y + (radius - i) as i32,
                });
                list.push(Coordinate {
                    x: self.x - i as i32,
                    y: self.y - (radius - i) as i32,
                });
                list.push(Coordinate {
                    x: self.x + i as i32,
                    y: self.y + (radius - i) as i32,
                });
                list.push(Coordinate {
                    x: self.x + i as i32,
                    y: self.y - (radius - i) as i32,
                });
            }

            list
        }
    }
}

fn calculate_largest_areas_nearest_to_one_coordinate_only(
    coordinates: &[Coordinate],
    center: &Coordinate,
) -> Option<u32> {
    let farthest_distant = center
        .calculate_longest_manhattant_distant(&coordinates)
        .unwrap();
    let mut coordinates_counter = HashMap::new();

    for i in 0..=farthest_distant {
        center
            .calculate_coordinates_within_manhattan_radius(i)
            .iter()
            .for_each(|point| {
                let mut coordinates_distant = coordinates
                    .iter()
                    .map(|coordinate| {
                        (
                            coordinate.clone(),
                            point.calculate_manhattan_length(coordinate),
                        )
                    })
                    .collect::<Vec<_>>();
                coordinates_distant.sort_by(|(_, dist1), (_, dist2)| dist1.cmp(dist2));
                if coordinates_distant[0].1 < coordinates_distant[1].1 {
                    *coordinates_counter
                        .entry(coordinates_distant[0].0.clone())
                        .or_insert(0u32) += 1;
                }
            });
    }

    let mut infinite_points = HashSet::new();
    center
        .calculate_coordinates_within_manhattan_radius(farthest_distant)
        .iter()
        .for_each(|point| {
            let mut coordinates_distant = coordinates
                .iter()
                .map(|coordinate| {
                    (
                        coordinate.clone(),
                        point.calculate_manhattan_length(coordinate),
                    )
                })
                .collect::<Vec<_>>();
            coordinates_distant.sort_by(|(_, dist1), (_, dist2)| dist1.cmp(dist2));
            if coordinates_distant[0].1 < coordinates_distant[1].1 {
                infinite_points.insert(coordinates_distant[0].0.clone());
            }
        });

    coordinates_counter
        .iter()
        .filter(|(coordinate, _)| !infinite_points.contains(coordinate))
        .map(|(_, size)| *size)
        .max()
}

fn calculate_largest_areas_nearest_to_all_coordinates(
    coordinates: &[Coordinate],
    center: &Coordinate,
    max_total_acceptable_distance: u32,
) -> u32 {
    let mut areas = 0;

    let mut radius = 0;
    loop {
        let mut any_within_acceptable_distance = false;
        center
            .calculate_coordinates_within_manhattan_radius(radius)
            .iter()
            .for_each(|point| {
                let total_distance: u32 = coordinates
                    .iter()
                    .map(|coordinate| point.calculate_manhattan_length(coordinate))
                    .sum();
                if total_distance < max_total_acceptable_distance {
                    any_within_acceptable_distance = true;
                    areas += 1;
                }
            });
        radius += 1;
        if !any_within_acceptable_distance {
            break;
        }
    }

    areas
}

fn main() -> Result<()> {
    let file = File::open("2018/day-06/input/input.txt").context("failed to read input file")?;
    let reader = BufReader::new(file);

    let coordinates = reader
        .lines()
        .filter_map(|line| line.ok().and_then(|s| s.parse::<Coordinate>().ok()))
        .collect::<Vec<_>>();
    let center = Coordinate::find_center(&coordinates);

    if let Some(max_size) =
        calculate_largest_areas_nearest_to_one_coordinate_only(&coordinates, &center)
    {
        writeln!(io::stdout(), "max coverage: {}", max_size)?;
    }
    let areas = calculate_largest_areas_nearest_to_all_coordinates(&coordinates, &center, 10_000);
    writeln!(io::stdout(), "areas within acceptable ranges: {}", areas)?;

    Ok(())
}
