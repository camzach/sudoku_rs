use std::fs::read_to_string;

use clap::Parser;
use log::{info, trace};
use simple_logger::{set_up_color_terminal, SimpleLogger};

mod grid;
use grid::{Cell, Grid};

use crate::{
    basic_sudoku::{basic_elimination, hidden_singles, naked_singles, naked_tuples},
    chess_strategies::kings,
    solver::Solver,
};

mod basic_sudoku;
mod chess_strategies;
mod solver;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    /// File to read from.
    /// If omitted, the sudoku will be read from stdin
    input: Option<String>,
    /// Enables backtracking when no logical steps remain
    #[arg(short, long)]
    backtracking: bool,
    /// Enables antiKing constraint
    #[arg(short = 'k', long)]
    antiking: bool,
    #[command(flatten)]
    log_level: clap_verbosity_flag::Verbosity,
}

fn main() -> Result<(), ()> {
    let args = Args::parse();

    set_up_color_terminal();
    let logger = SimpleLogger::new();

    if let Err(_) = log::set_boxed_logger(Box::new(logger)) {
        println!("Failed to initialize logging");
        return Err(());
    }
    log::set_max_level(args.log_level.log_level_filter());

    let Ok(input) = (match args.input {
        Some(infile) => read_to_string(infile),
        _ => {
            let mut out = String::new();
            println!("Enter your puzzle in one line, using any non-digit, non-whitespace character to represent an unknown cell.");
            std::io::stdin().read_line(&mut out).map(|_| out)
        }
    }) else {
        return Err(());
    };

    let mut grid = Grid([[Cell::default(); 9]; 9]);
    for (i, char) in input.replace([' ', '\n', '\t'], "").chars().enumerate() {
        if i >= 81 {
            break;
        }
        grid[i / 9][i % 9] = char.to_digit(10).map(|d| d as usize).into();
    }

    trace!("initial grid: \n{}", grid);

    let mut solver = Solver::new();
    solver.add_strategy(naked_singles);
    solver.add_strategy(basic_elimination);
    solver.add_strategy(hidden_singles);
    solver.add_strategy(naked_tuples);
    if args.antiking {
        solver.add_strategy(kings);
    }

    let mut failed = false;
    while !grid.solved() && !failed {
        if solver.step(&mut grid) {
            trace!("{}", grid);
        } else {
            failed = true;
        }
    }
    if !failed {
        info!("Puzzle solved!");
        return Ok(());
    }

    info!("Failed to find a solution logically.");
    if args.backtracking {
        trace!("Starting backtracking");
        if solver.backtrack(&mut grid) {
            info!("Solved!")
        } else {
            info!("Puzzle has no solutions")
        }
    } else {
        info!("Run with --backtracking to try again with backtracking enabled");
    }
    Ok(())
}
