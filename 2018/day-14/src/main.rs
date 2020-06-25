use anyhow::Result;
use std::io::{self, Write};

struct RecipeBoard {
    board: Vec<u8>,
    elf1_index: usize,
    elf2_index: usize,
}

impl RecipeBoard {
    fn new() -> Self {
        let board = vec![3, 7];

        RecipeBoard {
            board,
            elf1_index: 0,
            elf2_index: 1,
        }
    }

    fn step(&mut self) {
        let elf1_recipe = self.board[self.elf1_index];
        let elf2_recipe = self.board[self.elf2_index];
        let sum_recipe = elf1_recipe + elf2_recipe;
        if sum_recipe >= 10 {
            self.board.push(1);
        }
        self.board.push(sum_recipe % 10);

        let board_len = self.board.len();
        self.elf1_index = (self.elf1_index + elf1_recipe as usize + 1) % board_len;
        self.elf2_index = (self.elf2_index + elf2_recipe as usize + 1) % board_len;
    }
}

fn trace_first(board: &[u8], pattern: &[u8]) -> Vec<usize> {
    let pat_len = pattern.len();
    let board_len = board.len();
    if board_len < pat_len {
        vec![]
    } else {
        let mut result = vec![];
        for i in 0..board_len - pat_len {
            if board[i..i + pat_len]
                .iter()
                .zip(pattern.iter())
                .all(|(v1, v2)| v1 == v2)
            {
                result.push(i)
            }
        }

        result
    }
}

struct TraceRecipeBoard {
    recipe_board: RecipeBoard,
    tracer_index: Option<usize>,
    found_indices: Vec<usize>,
    pattern: Vec<u8>,
}

impl TraceRecipeBoard {
    fn new(rb: RecipeBoard, pattern: Vec<u8>) -> Self {
        let found_indices = trace_first(&rb.board, &pattern);
        let tracer_index = if rb.board.len() < pattern.len() {
            None
        } else {
            Some(rb.board.len() - pattern.len())
        };
        TraceRecipeBoard {
            recipe_board: rb,
            tracer_index,
            found_indices,
            pattern,
        }
    }

    fn step(&mut self) {
        self.recipe_board.step();
        if let Some(index) = self.tracer_index {
            for i in index + 1..self.recipe_board.board.len() - self.pattern.len() {
                if self.recipe_board.board[i..i + self.pattern.len()]
                    .iter()
                    .zip(self.pattern.iter())
                    .all(|(v1, v2)| v1 == v2)
                {
                    self.found_indices.push(i)
                }
            }
            self.tracer_index = Some(self.recipe_board.board.len() - self.pattern.len());
        }
    }
}

fn render_recipe_board(n: usize) -> RecipeBoard {
    let mut recipe_board = RecipeBoard::new();

    while recipe_board.board.len() < (n + 10) {
        recipe_board.step();
    }

    recipe_board
}

fn main() -> Result<()> {
    let input1 = 110201;
    let rb = render_recipe_board(input1 + 10);
    let scores = rb
        .board
        .iter()
        .skip(input1)
        .take(10)
        .copied()
        .collect::<Vec<_>>();
    writeln!(io::stdout(), "last ten recipe score {:?}", scores)?;

    // reuse previous calculation
    let mut trace_recipe_board = TraceRecipeBoard::new(rb, vec![1, 1, 0, 2, 0, 1]);
    while trace_recipe_board.found_indices.is_empty() {
        trace_recipe_board.step();
    }
    writeln!(
        io::stdout(),
        "first recipes score as pattern input {:?} found at {}",
        trace_recipe_board.pattern,
        trace_recipe_board.found_indices[0]
    )?;
    Ok(())
}
