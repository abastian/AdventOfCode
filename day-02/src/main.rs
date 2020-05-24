use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader, Error, Write},
};

fn calculate_checksum(box_sequences: &[String]) -> u64 {
    let mut box_2_letters_count = 0;
    let mut box_3_letters_count = 0;

    for box_sequence in box_sequences {
        let mut occurences = HashMap::new();

        for ch in box_sequence.chars() {
            *occurences.entry(ch).or_insert(0) += 1;
        }
        box_2_letters_count += if occurences.values().any(|&v| v == 2) {
            1
        } else {
            0
        };
        box_3_letters_count += if occurences.values().any(|&v| v == 3) {
            1
        } else {
            0
        };
    }

    box_2_letters_count * box_3_letters_count
}

fn find_first_near_identical_box(box_sequences: &[String]) -> Option<(&String, &String)> {
    for (idx, box_sequence) in box_sequences.iter().enumerate() {
        if let Some((_, near_identical)) = box_sequences
            .iter()
            .enumerate()
            .filter(|&(inner_idx, _)| inner_idx != idx)
            .find(|&(_, inner_box_sequence)| {
                if box_sequence.len() == inner_box_sequence.len() {
                    let mut any_diff = false;
                    for (ch1, ch2) in box_sequence.chars().zip(inner_box_sequence.chars()) {
                        if ch1 != ch2 {
                            if any_diff {
                                return false;
                            } else {
                                any_diff = true;
                            }
                        }
                    }
                    any_diff
                } else {
                    false
                }
            })
        {
            return Some((box_sequence, near_identical));
        }
    }
    None
}

fn main() -> Result<(), Error> {
    let file = File::open("input/input.txt")?;
    let reader = BufReader::new(file);
    let box_sequences = reader
        .lines()
        .filter_map(|line| line.ok())
        .collect::<Vec<_>>();

    writeln!(
        io::stdout(),
        "checksum: {}",
        calculate_checksum(&box_sequences)
    )?;

    if let Some((str1, str2)) = find_first_near_identical_box(&box_sequences) {
        let ident_letters = str1
            .chars()
            .zip(str2.chars())
            .filter(|&(ch1, ch2)| ch1 == ch2)
            .map(|(ch, _)| ch)
            .collect::<String>();
        writeln!(
            io::stdout(),
            "common letters of near identical box: \"{}\"",
            ident_letters
        )?;
    } else {
        writeln!(io::stderr(), "no near identical boxes found")?;
    }
    Ok(())
}
