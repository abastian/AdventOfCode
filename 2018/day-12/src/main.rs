use anyhow::{anyhow, Context, Error, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
    str::FromStr,
};

lazy_static! {
    static ref INPUT_EXERCISE_RE: Regex =
        Regex::new(r"^(?P<from>[#.]{5}) => (?P<to>[#.])$").unwrap();
    static ref POTS_SIMPLIFY_RE: Regex =
        Regex::new(r"^(?P<ignore_start>\.*)(?P<simple>#[.#]*#)(?P<ignore_end>\.*)$").unwrap();
}

struct InputExercise {
    initial_state: String,
    transitions: HashMap<String, u8>,
}

impl FromStr for InputExercise {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        lazy_static! {}
        let mut lines = s.lines();

        let initial_state_line = match lines.next() {
            None => return Err(anyhow!("empty initial state")),
            Some(line) => line,
        };
        let prefix = "initial state: ";
        if !initial_state_line.starts_with(prefix) {
            return Err(anyhow!("unexpected prefix for initial state"));
        }
        let initial_state = initial_state_line.split_at(prefix.len()).1.to_string();

        match lines.next() {
            None => return Err(anyhow!("invalid end of data")),
            Some(line) if !line.is_empty() => {
                return Err(anyhow!("missing empty line separating transitions"))
            }
            _ => (),
        }

        let transitions = lines
            .map(|line| match INPUT_EXERCISE_RE.captures(&line) {
                None => Err(anyhow!("unrecognized transition pattern \"{}\"", &line)),
                Some(caps) => Ok((caps["from"].to_string(), caps["to"].as_bytes()[0])),
            })
            .collect::<Result<HashMap<String, u8>>>()?;

        Ok(InputExercise {
            initial_state,
            transitions,
        })
    }
}

struct PotsModel {
    presentation: String,
    pos_left: isize,
    pos_right: isize,
}

impl PotsModel {
    fn new(initial_state: &str) -> Result<Self> {
        if let Some(caps) = POTS_SIMPLIFY_RE.captures(initial_state) {
            Ok(PotsModel {
                presentation: format!("..{}..", caps["simple"].to_string()),
                pos_left: caps["ignore_start"].len() as isize - 2,
                pos_right: (initial_state.len() - caps["ignore_end"].len()) as isize + 2,
            })
        } else {
            Err(anyhow!("unrecognized initial state: {}", initial_state))
        }
    }

    fn simplify(&mut self) {
        if let Some(caps) = POTS_SIMPLIFY_RE.captures(&self.presentation) {
            self.pos_left += caps["ignore_start"].len() as isize - 2;
            self.pos_right -= caps["ignore_end"].len() as isize - 2;
            self.presentation = format!("..{}..", caps["simple"].to_string());
        }
    }

    fn render_next(&mut self, input_exercise: &InputExercise) -> Result<()> {
        let next_presentation = format!("..{}..", self.presentation)
            .as_bytes()
            .windows(5)
            .map(|pat| {
                let key = String::from_utf8_lossy(pat).to_string();
                input_exercise
                    .transitions
                    .get(&key)
                    .cloned()
                    .ok_or_else(|| anyhow!("unregistered pattern: {}", &key))
            })
            .collect::<Result<Vec<u8>>>()?;
        self.presentation = String::from_utf8_lossy(&next_presentation).to_string();
        self.simplify();

        Ok(())
    }
}

fn render_n_generation(input_exercise: &InputExercise, n: usize) -> Result<PotsModel> {
    let mut pots_model = PotsModel::new(&input_exercise.initial_state)?;
    for _ in 1..=n {
        pots_model.render_next(input_exercise)?;
    }

    Ok(pots_model)
}

fn main() -> Result<()> {
    let content = fs::read_to_string("input/input.txt").context("failed to read input file")?;
    let input_exercise = content.parse::<InputExercise>()?;

    let pots_model = render_n_generation(&input_exercise, 20)?;
    let sum_pot_number_with_plant = pots_model
        .presentation
        .as_bytes()
        .iter()
        .zip(pots_model.pos_left..=pots_model.pos_right)
        .filter_map(|(p, idx)| if *p == b'#' { Some(idx) } else { None })
        .sum::<isize>();
    writeln!(
        io::stdout(),
        "sum of pot number with plants: {}",
        sum_pot_number_with_plant
    )?;

    Ok(())
}
