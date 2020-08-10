use anyhow::{anyhow, Context, Result};
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::Path,
};

#[derive(Clone, Copy, PartialEq)]
enum Field {
    Open,
    Tree,
    Lumber,
}

impl Display for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Field::Open => write!(f, "."),
            Field::Tree => write!(f, "|"),
            Field::Lumber => write!(f, "#"),
        }
    }
}

struct Grid {
    x: usize,
    y: usize,
    field: Field,
}

type Grids = Vec<Vec<Field>>;

fn grids_string(grids: &Grids) -> String {
    grids
        .iter()
        .flatten()
        .map(|field| field.to_string())
        .collect::<String>()
}

fn render_input<P: AsRef<Path>>(path: P) -> Result<Grids> {
    let file = File::open(path).context("failed to read input file")?;
    let reader = BufReader::new(file);

    reader
        .lines()
        .filter_map(|line| {
            line.ok().map(|row| {
                row.as_bytes()
                    .iter()
                    .map(|cell| match *cell {
                        b'.' => Ok(Field::Open),
                        b'|' => Ok(Field::Tree),
                        b'#' => Ok(Field::Lumber),
                        field => Err(anyhow!("invalid field {}", field as char)),
                    })
                    .collect::<Result<Vec<_>>>()
            })
        })
        .collect::<Result<Vec<_>>>()
}

fn terraform(grids: &Grids) -> Grids {
    let row_number = grids.len();
    let col_number = grids[0].len();

    let mut result = vec![vec![Field::Open; col_number]; row_number];
    for i in 0..row_number {
        let neighbor_row = {
            let grid_enumerate = grids.iter().enumerate();
            if i > 0 {
                grid_enumerate.skip(i - 1).take(3)
            } else {
                grid_enumerate.skip(0).take(2)
            }
        };
        for j in 0..col_number {
            let neighbor_row_col = neighbor_row.clone().flat_map(|(y, row)| {
                let row_enumerate = row.iter().enumerate();
                if j > 0 {
                    row_enumerate.skip(j - 1).take(3)
                } else {
                    row_enumerate.skip(0).take(2)
                }
                .map(move |(x, field)| Grid {
                    x,
                    y,
                    field: *field,
                })
            });
            let neighbor = neighbor_row_col.filter(|grid| !(grid.x == j && grid.y == i));
            result[i][j] = match grids[i][j] {
                Field::Open => {
                    if neighbor.filter(|grid| grid.field == Field::Tree).count() >= 3 {
                        Field::Tree
                    } else {
                        Field::Open
                    }
                }
                Field::Tree => {
                    if neighbor.filter(|grid| grid.field == Field::Lumber).count() >= 3 {
                        Field::Lumber
                    } else {
                        Field::Tree
                    }
                }
                Field::Lumber => {
                    let neighbor = neighbor
                        .filter(|grid| grid.field != Field::Open)
                        .map(|grid| grid.field)
                        .collect::<Vec<_>>();
                    if neighbor.len() > 1
                        && neighbor.contains(&Field::Lumber)
                        && neighbor.contains(&Field::Tree)
                    {
                        Field::Lumber
                    } else {
                        Field::Open
                    }
                }
            }
        }
    }

    result
}

fn calculate_fields(grids: &Grids) -> (usize, usize, usize) {
    grids.iter().flatten().fold((0, 0, 0), |acc, grid| {
        let (mut lumber_num, mut tree_num, mut open_num) = acc;
        match *grid {
            Field::Tree => tree_num += 1,
            Field::Lumber => lumber_num += 1,
            Field::Open => open_num += 1,
        }
        (lumber_num, tree_num, open_num)
    })
}

fn main() -> Result<()> {
    let mut grids = render_input("2018/day-18/input/input.txt")?;
    let mut grids_set: HashMap<String, (usize, Vec<usize>)> = HashMap::new();

    for idx in 1..=10 {
        grids = terraform(&grids);
        let grids_string = grids_string(&grids);
        let (lumber_num, tree_num, _) = calculate_fields(&grids);
        let entry = grids_set.entry(grids_string).or_insert((0, vec![]));
        entry.0 = lumber_num * tree_num;
        entry.1.push(idx);
    }

    let (lumber_num, tree_num, _) = calculate_fields(&grids);
    writeln!(
        io::stdout(),
        "resource product on 10th iteration: lumber: {}, tree: {} = {}",
        lumber_num,
        tree_num,
        lumber_num * tree_num
    )?;

    let mut idx = 11;
    let (last_idx, first_idx) = loop {
        grids = terraform(&grids);
        let grids_string = grids_string(&grids);
        let (lumber_num, tree_num, _) = calculate_fields(&grids);
        let entry = grids_set.entry(grids_string.clone()).or_insert((0, vec![]));
        entry.0 = lumber_num * tree_num;
        entry.1.push(idx);
        if grids_set.len() != idx {
            break (idx, grids_set[&grids_string].1[0]);
        }
        idx += 1;
    };
    writeln!(
        io::stdout(),
        "first repeat when idx: {}, of idx: {}",
        last_idx,
        first_idx
    )?;
    grids_set.clear();

    let mut repeat_grids: Vec<Grids> = Vec::with_capacity(last_idx - first_idx);
    for _ in first_idx..last_idx {
        repeat_grids.push(grids.clone());
        grids = terraform(&grids);
    }
    let target_repetition = 1_000_000_000;
    let target_idx = (target_repetition - first_idx) % (last_idx - first_idx);
    let target_grids = &repeat_grids[target_idx];
    let (target_lumber_num, target_tree_num, _) = calculate_fields(target_grids);

    writeln!(
        io::stdout(),
        "resource product on {}th iteration: lumber: {}, tree: {} = {}",
        target_repetition,
        target_lumber_num,
        target_tree_num,
        target_lumber_num * target_tree_num
    )?;

    Ok(())
}
