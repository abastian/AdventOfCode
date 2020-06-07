use anyhow::{anyhow, Result};
use std::io::{self, Write};

fn calculate_power_grids(serial_number: u32, n: usize) -> Vec<Vec<i32>> {
    (0..n)
        .map(|y| {
            (0..n)
                .map(|x| {
                    let rack_id = x + 10;
                    let mut power_level = rack_id * y;
                    power_level += serial_number as usize;
                    power_level *= rack_id;
                    let hundred = (power_level % 1000) / 100;
                    (hundred as i32) - 5
                })
                .collect::<Vec<i32>>()
        })
        .collect::<Vec<_>>()
}

fn calculate_cluster_power_grids(grids: &[Vec<i32>], cluster_size: usize) -> Result<Vec<Vec<i32>>> {
    if grids.len() < cluster_size {
        Err(anyhow!(
            "cluster size {} larger than grids length {}",
            cluster_size,
            grids.len()
        ))
    } else {
        Ok(grids
            .iter()
            .map(|row| {
                row.windows(cluster_size)
                    .map(|x_cluster| x_cluster.iter().sum())
                    .collect::<Vec<i32>>()
            })
            .collect::<Vec<_>>()
            .windows(cluster_size)
            .map(|y_cluster| {
                let row0 = y_cluster[0].clone();
                y_cluster[1..].iter().fold(row0, |acc, row| {
                    acc.iter()
                        .zip(row.iter())
                        .map(|(v1, v2)| v1 + v2)
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>())
    }
}

struct GridPower {
    n: usize,
    posx: usize,
    posy: usize,
    power: i32,
}

fn main() -> Result<()> {
    let power_grids = calculate_power_grids(8561, 300);
    if let Ok(power_cluster_grids) = calculate_cluster_power_grids(&power_grids, 3) {
        if let Some(((x, y), power)) = power_cluster_grids
            .iter()
            .enumerate()
            .flat_map(|(y, row)| {
                row.iter()
                    .enumerate()
                    .map(move |(x, power)| ((x, y), power))
            })
            .max_by(|(_, power1), (_, power2)| power1.cmp(power2))
        {
            writeln!(
                io::stdout(),
                "3x3 cluster grid with max power: {}, located at ({}, {})",
                power,
                x,
                y
            )?;
        }
    }

    let mut highest_grid_power: Option<GridPower> = None;
    for n in 1..=300 {
        if let Ok(cluster_grids) = calculate_cluster_power_grids(&power_grids, n) {
            if let Some(grid) = &highest_grid_power {
                let next_highest_grid_power = cluster_grids
                    .iter()
                    .enumerate()
                    .flat_map(|(y, row)| {
                        row.iter().enumerate().filter_map(move |(x, power)| {
                            if *power > grid.power {
                                Some(GridPower {
                                    n,
                                    posx: x,
                                    posy: y,
                                    power: *power,
                                })
                            } else {
                                None
                            }
                        })
                    })
                    .max_by(|g1, g2| g1.power.cmp(&g2.power));
                if next_highest_grid_power.is_some() {
                    highest_grid_power = next_highest_grid_power;
                }
            } else {
                highest_grid_power = cluster_grids
                    .iter()
                    .enumerate()
                    .flat_map(|(y, row)| {
                        row.iter().enumerate().map(move |(x, power)| GridPower {
                            n,
                            posx: x,
                            posy: y,
                            power: *power,
                        })
                    })
                    .max_by(|g1, g2| g1.power.cmp(&g2.power))
            }
        }
    }
    if let Some(grid) = highest_grid_power {
        writeln!(
            io::stdout(),
            "best power using dimension {}, location ({}, {}) with power {}",
            grid.n,
            grid.posx,
            grid.posy,
            grid.power
        )?;
    }
    Ok(())
}
