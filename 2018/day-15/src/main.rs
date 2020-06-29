use anyhow::{anyhow, Context, Result};
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet},
    fs,
    hash::Hash,
    io::{self, Write},
    rc::Rc,
};

#[derive(PartialEq)]
enum Terrain {
    Wall,
    Space,
}

#[derive(PartialEq, Eq, Hash)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Race {
    Elf,
    Goblin,
}

#[derive(Clone)]
struct Unit {
    race: Race,
    attack: u32,
    hp: u32,
}

type RefUnit = Rc<RefCell<Unit>>;

#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Clone, Copy)]
struct Location {
    y: usize,
    x: usize,
}

type Grid = Vec<Vec<Terrain>>;

struct Trace {
    next_step: Location,
    comes_from: HashSet<Direction>,
}

fn scan_grids(lines: String) -> Result<(Grid, BTreeMap<Location, Unit>)> {
    let mut grids = vec![];
    let mut units = BTreeMap::new();
    for (y, line) in lines.lines().enumerate() {
        let mut y_grids = vec![];
        for (x, spot) in line.as_bytes().iter().enumerate() {
            match spot {
                b'#' => y_grids.push(Terrain::Wall),
                b'.' => y_grids.push(Terrain::Space),
                b'G' => {
                    y_grids.push(Terrain::Space);
                    units.insert(
                        Location { y, x },
                        Unit {
                            race: Race::Goblin,
                            attack: 3,
                            hp: 200,
                        },
                    );
                }
                b'E' => {
                    y_grids.push(Terrain::Space);
                    units.insert(
                        Location { y, x },
                        Unit {
                            race: Race::Elf,
                            attack: 3,
                            hp: 200,
                        },
                    );
                }
                _ => return Err(anyhow!("invalid data on {}, {}", x, y)),
            }
        }
        grids.push(y_grids)
    }
    Ok((grids, units))
}

fn in_range_weakest_enemy(
    loc: &Location,
    units: &BTreeMap<Location, RefUnit>,
    grid: &[Vec<Terrain>],
    enemy_race: Race,
) -> Option<(Location, RefUnit)> {
    let mut in_range_enemies = vec![];
    if loc.y > 0 {
        let enemy_loc = Location {
            y: loc.y - 1,
            x: loc.x,
        };
        if let Some(unit) = units.get(&enemy_loc) {
            if unit.borrow().race == enemy_race && unit.borrow().hp > 0 {
                in_range_enemies.push((enemy_loc, unit.clone()))
            };
        }
    }
    if loc.y < grid.len() {
        let enemy_loc = Location {
            y: loc.y + 1,
            x: loc.x,
        };
        if let Some(unit) = units.get(&enemy_loc) {
            if unit.borrow().race == enemy_race && unit.borrow().hp > 0 {
                in_range_enemies.push((enemy_loc, unit.clone()))
            };
        }
    }
    if loc.x > 0 {
        let enemy_loc = Location {
            y: loc.y,
            x: loc.x - 1,
        };
        if let Some(unit) = units.get(&enemy_loc) {
            if unit.borrow().race == enemy_race && unit.borrow().hp > 0 {
                in_range_enemies.push((enemy_loc, unit.clone()))
            };
        }
    }
    if loc.x < grid[0].len() {
        let enemy_loc = Location {
            y: loc.y,
            x: loc.x + 1,
        };
        if let Some(unit) = units.get(&enemy_loc) {
            if unit.borrow().race == enemy_race && unit.borrow().hp > 0 {
                in_range_enemies.push((enemy_loc, unit.clone()))
            };
        }
    }

    in_range_enemies.sort_by(|(eloc1, e1), (eloc2, e2)| {
        let e1 = e1.borrow();
        let e2 = e2.borrow();
        (e1.hp, eloc1.y, eloc2.x).cmp(&(e2.hp, eloc2.y, eloc2.x))
    });
    in_range_enemies.get(0).map(|(l, e)| (*l, e.clone()))
}

fn move_to_nearest_enemy(
    old_loc: &Location,
    units: &BTreeMap<Location, RefUnit>,
    grid: &[Vec<Terrain>],
    enemy_race: Race,
) -> Option<Location> {
    let target_spots = units
        .iter()
        .filter(|(_, u)| u.borrow().race == enemy_race)
        .flat_map(|(l, _)| {
            let mut spots = vec![];
            if l.y > 0 {
                let spot = Location { y: l.y - 1, x: l.x };
                if units.get(&spot).is_none() && grid[spot.y][spot.x] == Terrain::Space {
                    spots.push(spot);
                }
            }
            if l.y < grid.len() {
                let spot = Location { y: l.y + 1, x: l.x };
                if units.get(&spot).is_none() && grid[spot.y][spot.x] == Terrain::Space {
                    spots.push(spot);
                }
            }
            if l.x > 0 {
                let spot = Location { y: l.y, x: l.x - 1 };
                if units.get(&spot).is_none() && grid[spot.y][spot.x] == Terrain::Space {
                    spots.push(spot);
                }
            }
            if l.x < grid[0].len() {
                let spot = Location { y: l.y, x: l.x + 1 };
                if units.get(&spot).is_none() && grid[spot.y][spot.x] == Terrain::Space {
                    spots.push(spot);
                }
            }
            spots
        })
        .collect::<HashSet<Location>>();

    if !target_spots.is_empty() {
        let mut traces = BTreeMap::new();
        if old_loc.y > 0 {
            let l = Location {
                y: old_loc.y - 1,
                x: old_loc.x,
            };
            if target_spots.contains(&l) {
                return Some(l);
            }
            if units.get(&l).is_none() && grid[l.y][l.x] == Terrain::Space {
                traces
                    .entry(l)
                    .or_insert(Trace {
                        next_step: l,
                        comes_from: HashSet::new(),
                    })
                    .comes_from
                    .insert(Direction::Down);
            }
        }
        if old_loc.x > 0 {
            let l = Location {
                y: old_loc.y,
                x: old_loc.x - 1,
            };
            if target_spots.contains(&l) {
                return Some(l);
            }
            if units.get(&l).is_none() && grid[l.y][l.x] == Terrain::Space {
                traces
                    .entry(l)
                    .or_insert(Trace {
                        next_step: l,
                        comes_from: HashSet::new(),
                    })
                    .comes_from
                    .insert(Direction::Right);
            }
        }
        if old_loc.x < grid[0].len() {
            let l = Location {
                y: old_loc.y,
                x: old_loc.x + 1,
            };
            if target_spots.contains(&l) {
                return Some(l);
            }
            if units.get(&l).is_none() && grid[l.y][l.x] == Terrain::Space {
                traces
                    .entry(l)
                    .or_insert(Trace {
                        next_step: l,
                        comes_from: HashSet::new(),
                    })
                    .comes_from
                    .insert(Direction::Left);
            }
        }
        if old_loc.y < grid.len() {
            let l = Location {
                y: old_loc.y + 1,
                x: old_loc.x,
            };
            if target_spots.contains(&l) {
                return Some(l);
            }
            if units.get(&l).is_none() && grid[l.y][l.x] == Terrain::Space {
                traces
                    .entry(l)
                    .or_insert(Trace {
                        next_step: l,
                        comes_from: HashSet::new(),
                    })
                    .comes_from
                    .insert(Direction::Up);
            }
        }

        while !traces.is_empty() {
            let mut new_traces = BTreeMap::new();
            for (loc, trace) in traces.iter() {
                if loc.y > 0 && !trace.comes_from.contains(&Direction::Up) {
                    let l = Location {
                        y: loc.y - 1,
                        x: loc.x,
                    };
                    if target_spots.contains(&l) {
                        return Some(trace.next_step);
                    }
                    if units.get(&l).is_none() && grid[l.y][l.x] == Terrain::Space {
                        new_traces
                            .entry(l)
                            .or_insert(Trace {
                                next_step: trace.next_step,
                                comes_from: HashSet::new(),
                            })
                            .comes_from
                            .insert(Direction::Down);
                    }
                }
                if loc.x > 0 && !trace.comes_from.contains(&Direction::Left) {
                    let l = Location {
                        y: loc.y,
                        x: loc.x - 1,
                    };
                    if target_spots.contains(&l) {
                        return Some(trace.next_step);
                    }
                    if units.get(&l).is_none() && grid[l.y][l.x] == Terrain::Space {
                        new_traces
                            .entry(l)
                            .or_insert(Trace {
                                next_step: trace.next_step,
                                comes_from: HashSet::new(),
                            })
                            .comes_from
                            .insert(Direction::Right);
                    }
                }
                if loc.x < grid[0].len() && !trace.comes_from.contains(&Direction::Right) {
                    let l = Location {
                        y: loc.y,
                        x: loc.x + 1,
                    };
                    if target_spots.contains(&l) {
                        return Some(trace.next_step);
                    }
                    if units.get(&l).is_none() && grid[l.y][l.x] == Terrain::Space {
                        new_traces
                            .entry(l)
                            .or_insert(Trace {
                                next_step: trace.next_step,
                                comes_from: HashSet::new(),
                            })
                            .comes_from
                            .insert(Direction::Left);
                    }
                }
                if loc.y < grid.len() && !trace.comes_from.contains(&Direction::Down) {
                    let l = Location {
                        y: loc.y + 1,
                        x: loc.x,
                    };
                    if target_spots.contains(&l) {
                        return Some(trace.next_step);
                    }
                    if units.get(&l).is_none() && grid[l.y][l.x] == Terrain::Space {
                        new_traces
                            .entry(l)
                            .or_insert(Trace {
                                next_step: trace.next_step,
                                comes_from: HashSet::new(),
                            })
                            .comes_from
                            .insert(Direction::Up);
                    }
                }
            }

            traces = new_traces;
        }
    }

    None
}

fn rounds(grid: &[Vec<Terrain>], units: &mut BTreeMap<Location, RefUnit>) -> bool {
    let any_elf = units.values().any(|u| u.borrow().race == Race::Elf);
    let any_goblin = units.values().any(|u| u.borrow().race == Race::Goblin);

    if any_elf && any_goblin {
        units
            .clone()
            .iter()
            .filter(|(_, u)| u.borrow().hp > 0)
            .for_each(|(loc, unit)| {
                if let Some(enemy) = {
                    let enemy_race = match unit.borrow().race {
                        Race::Elf => Race::Goblin,
                        Race::Goblin => Race::Elf,
                    };
                    let enemy = in_range_weakest_enemy(loc, units, grid, enemy_race);
                    if enemy.is_none() {
                        if let Some(new_loc) = move_to_nearest_enemy(loc, units, grid, enemy_race) {
                            units.remove(loc);
                            units.insert(new_loc, unit.clone());
                            in_range_weakest_enemy(&new_loc, units, grid, enemy_race)
                        } else {
                            None
                        }
                    } else {
                        enemy
                    }
                } {
                    let (enemy_loc, enemy_unit) = enemy;
                    let mut enemy_unit = enemy_unit.borrow_mut();
                    enemy_unit.hp = enemy_unit.hp.saturating_sub(unit.borrow().attack);

                    if enemy_unit.hp == 0 {
                        units.remove(&enemy_loc);
                    }
                }
            });
        true
    } else {
        false
    }
}

fn main() -> Result<()> {
    let content = fs::read_to_string("input/input.txt").context("failed to read input file")?;
    let (grid, units) = scan_grids(content)?;
    let mut units_part1 = units
        .iter()
        .map(|(l, u)| (*l, Rc::new(RefCell::new(u.clone()))))
        .collect::<BTreeMap<_, _>>();
    let mut round = 0usize;
    while rounds(&grid, &mut units_part1) {
        round += 1;
    }
    let total_hp = units_part1.values().map(|u| u.borrow().hp).sum::<u32>();
    writeln!(
        io::stdout(),
        "Combat ends after {} full rounds, with total hp left {}, answer: {}",
        round - 1,
        total_hp,
        (round - 1) * total_hp as usize
    )?;

    let elves_number = units.values().filter(|u| u.race == Race::Elf).count();
    let mut elves_attack = 4;
    loop {
        let mut units_part2 = units
            .iter()
            .map(|(l, u)| {
                if u.race == Race::Elf {
                    (
                        *l,
                        Rc::new(RefCell::new(Unit {
                            race: Race::Elf,
                            attack: elves_attack,
                            hp: 200,
                        })),
                    )
                } else {
                    (*l, Rc::new(RefCell::new(u.clone())))
                }
            })
            .collect::<BTreeMap<_, _>>();
        let mut round = 0usize;
        while rounds(&grid, &mut units_part2) {
            round += 1;
        }
        let winner_race = units_part2.values().next().unwrap().borrow().race;
        let total_hp = units_part2.values().map(|u| u.borrow().hp).sum::<u32>();
        writeln!(
            io::stdout(),
            "with attack power {}, combat wins by {:?} after {} full rounds, with total hp left {}, answer: {}",
            elves_attack,
            winner_race,
            round - 1,
            total_hp,
            (round - 1) * total_hp as usize
        )?;

        if units_part2
            .values()
            .filter(|u| u.borrow().race == Race::Elf)
            .count()
            == elves_number
        {
            break;
        } else {
            elves_attack += 1;
        }
    }

    Ok(())
}
