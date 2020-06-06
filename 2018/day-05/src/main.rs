use anyhow::{Context, Result};
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

fn reacting(mut polymer: impl Iterator<Item = u8>) -> String {
    fn test_react(unit1: u8, unit2: u8) -> bool {
        if unit1 < unit2 {
            unit2 - unit1 == 32
        } else {
            unit1 - unit2 == 32
        }
    }

    let mut new_polymer = Vec::new();
    let mut reactant = polymer.next();

    for unit2 in polymer {
        if let Some(unit1) = reactant {
            if test_react(unit1, unit2) {
                reactant = new_polymer.pop();
            } else {
                new_polymer.push(unit1);
                reactant = Some(unit2);
            }
        } else {
            reactant = Some(unit2);
        }
    }
    if let Some(last) = reactant {
        new_polymer.push(last);
    }

    String::from_utf8_lossy(&new_polymer).to_string()
}

fn main() -> Result<()> {
    let file = File::open("input/input.txt").context("failed to read input file")?;
    let reader = BufReader::new(file);

    if let Some(Ok(base_polymer)) = reader.lines().take(1).next() {
        let base_polymer = base_polymer.as_bytes();
        let shrink_polymer = reacting(base_polymer.iter().cloned());
        writeln!(
            io::stdout(),
            "shrink polymer has {} unit",
            shrink_polymer.len()
        )?;

        if let Some((b, len)) = (b'A'..=b'Z')
            .map(|b| {
                let reduce_polymer = base_polymer
                    .iter()
                    .cloned()
                    .filter(|&v| v != b && v != (b + 32));
                (b, reacting(reduce_polymer).len())
            })
            .min_by(|(_, len1), (_, len2)| len1.cmp(len2))
        {
            writeln!(
                io::stdout(),
                "by reducing unit {}, polymer can further be shrink to {} unit",
                b as char,
                len
            )?;
        } else {
            writeln!(io::stderr(), "can't reduce further")?;
        }
    } else {
        writeln!(io::stderr(), "no polymer to be processed")?;
    }

    Ok(())
}
