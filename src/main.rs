use std::fs::read_to_string;

use clap::Parser;
use log::{info, trace};
use simple_logger::{set_up_color_terminal, SimpleLogger};
// use std::io::Read;

#[derive(Clone, Copy, Debug)]
enum Cell {
    Solved(usize),
    Unsolved([bool; 9]),
}
impl Cell {
    fn new(n: Option<usize>) -> Cell {
        n.map(|n| Cell::Solved(n - 1))
            .unwrap_or(Cell::Unsolved([true; 9]))
    }
    fn row(ns: [Option<usize>; 9]) -> [Cell; 9] {
        ns.map(Cell::new)
    }
    fn remove_candidate(&mut self, n: usize) -> bool {
        if let Cell::Unsolved(cands) = self {
            if cands[n] {
                cands[n] = false;
                return true;
            }
        }
        false
    }
}
impl core::fmt::Display for Cell {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Cell::Solved(n) = self {
            f.write_str(&(n + 1).to_string())
        } else {
            f.write_str(&"_")
        }
    }
}
impl std::default::Default for Cell {
    fn default() -> Self {
        Cell::Unsolved([true; 9])
    }
}

type Grid = [[Cell; 9]; 9];
struct PrintableGrid(Grid);
trait GridTrait {
    fn new(grid: [[Option<usize>; 9]; 9]) -> Grid;
    fn solved(&self) -> bool;
    fn broken(&self) -> bool;

    fn iter_cols(&mut self) -> Vec<Vec<&mut Cell>>;
    fn iter_boxes(&mut self) -> Vec<Vec<&mut Cell>>;

    fn naked_singles(&mut self) -> Option<()>;
    fn basic_elimination(&mut self) -> Option<()>;
    fn hidden_singles(&mut self) -> Option<()>;

    fn step(&mut self) -> Option<()>;

    fn backtrack(&mut self) -> Option<()>;
}
impl std::fmt::Display for PrintableGrid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for (i, line) in self.0.into_iter().enumerate() {
            if i % 3 == 0 && i > 0 {
                f.write_str("---------+---------+---------\n")?;
            }
            for (j, chunk) in line.chunks(3).enumerate() {
                if j % 3 != 0 {
                    f.write_str("|")?;
                }
                for c in chunk {
                    f.write_fmt(format_args!(" {} ", c))?;
                }
            }
            f.write_str("\n")?;
        }
        Ok(())
    }
}
impl GridTrait for Grid {
    fn new(grid: [[Option<usize>; 9]; 9]) -> Grid {
        let mut result = [[Cell::new(None); 9]; 9];
        for (i, row) in grid.into_iter().enumerate() {
            result[i] = Cell::row(row);
        }
        return result;
    }
    fn solved(&self) -> bool {
        self.iter()
            .flatten()
            .all(|c| if let Cell::Solved(_) = c { true } else { false })
    }
    fn broken(&self) -> bool {
        self.iter().flatten().any(|cell| {
            if let Cell::Unsolved(cands) = cell {
                cands.iter().all(|t| !t)
            } else {
                false
            }
        })
    }

    fn iter_cols(&mut self) -> Vec<Vec<&mut Cell>> {
        self.iter_mut().flatten().enumerate().fold(
            (0..9).map(|_| Vec::new()).collect(),
            |mut p, (i, c)| {
                p.get_mut(i % 9).unwrap().push(c);
                p
            },
        )
    }
    fn iter_boxes(&mut self) -> Vec<Vec<&mut Cell>> {
        self.iter_mut().flatten().enumerate().fold(
            (0..9).map(|_| Vec::new()).collect(),
            |mut p, (i, c)| {
                let row = i / 27;
                let col = (i % 9) / 3;
                p.get_mut(col + row * 3).unwrap().push(c);
                p
            },
        )
    }

    fn naked_singles(&mut self) -> Option<()> {
        trace!("Searching for naked singles");
        let mut result = None;

        for ref mut cell in self.iter_mut().flatten() {
            if let Cell::Unsolved(candidates) = cell {
                if candidates.iter().filter(|t| **t).count() == 1 {
                    let n = candidates.iter().position(|c| *c == true).unwrap();
                    **cell = Cell::Solved(n);
                    result = Some(());
                }
            }
        }
        result
    }
    fn basic_elimination(&mut self) -> Option<()> {
        trace!("Attempting basic elimination");
        let mut result = None;
        // rows
        for row in self.iter_mut() {
            let ns_present: Vec<_> = row
                .iter_mut()
                .filter_map(|c| {
                    if let Cell::Solved(n) = c {
                        Some(n.clone())
                    } else {
                        None
                    }
                })
                .collect();
            for cell in row {
                for n in ns_present.iter() {
                    if cell.remove_candidate(*n) {
                        result = Some(())
                    }
                }
            }
        }

        // cols
        for mut col in self.iter_cols() {
            let ns_present: Vec<_> = col
                .iter_mut()
                .filter_map(|c| {
                    if let Cell::Solved(n) = c {
                        Some(n.clone())
                    } else {
                        None
                    }
                })
                .collect();
            for cell in col {
                for n in ns_present.iter() {
                    if cell.remove_candidate(*n) {
                        result = Some(())
                    }
                }
            }
        }

        // boxes
        for mut bx in self.iter_boxes() {
            let ns_present: Vec<_> = bx
                .iter_mut()
                .filter_map(|c| {
                    if let Cell::Solved(n) = c {
                        Some(n.clone())
                    } else {
                        None
                    }
                })
                .collect();
            for cell in bx {
                for n in ns_present.iter() {
                    if cell.remove_candidate(*n) {
                        result = Some(())
                    }
                }
            }
        }

        result
    }
    fn hidden_singles(&mut self) -> Option<()> {
        trace!("Searching for hidden singles");
        let mut result = None;
        for row in self.iter_mut() {
            for i in 0..9 {
                let cells: Vec<_> = row
                    .iter_mut()
                    .filter(|c| {
                        if let Cell::Unsolved(cands) = c {
                            return cands[i];
                        }
                        false
                    })
                    .collect();
                if cells.len() == 1 {
                    result = Some(());
                    for cell in cells {
                        let mut newcands = [false; 9];
                        newcands[i] = true;
                        *cell = Cell::Unsolved(newcands);
                    }
                }
            }
        }

        for mut col in self.iter_cols() {
            for i in 0..9 {
                let cells: Vec<_> = col
                    .iter_mut()
                    .filter(|c| {
                        if let Cell::Unsolved(cands) = c {
                            return cands[i];
                        }
                        false
                    })
                    .collect();
                if cells.len() == 1 {
                    result = Some(());
                    for cell in cells {
                        let mut newcands = [false; 9];
                        newcands[i] = true;
                        **cell = Cell::Unsolved(newcands);
                    }
                }
            }
        }

        for mut bx in self.iter_boxes() {
            for i in 0..9 {
                let cells: Vec<_> = bx
                    .iter_mut()
                    .filter(|c| {
                        if let Cell::Unsolved(cands) = c {
                            return cands[i];
                        }
                        false
                    })
                    .collect();
                if cells.len() == 1 {
                    result = Some(());
                    for cell in cells {
                        let mut newcands = [false; 9];
                        newcands[i] = true;
                        **cell = Cell::Unsolved(newcands);
                    }
                }
            }
        }

        result
    }

    fn step(&mut self) -> Option<()> {
        if let Some(_) = self.naked_singles() {
            trace!("Naked singles");
        } else if let Some(_) = self.basic_elimination() {
            trace!("Basic elimination");
        } else if let Some(_) = self.hidden_singles() {
            trace!("Hidden singles");
        } else {
            return None;
        }
        Some(())
    }

    fn backtrack(&mut self) -> Option<()> {
        let target = self.iter().flatten().enumerate().fold(None, |p, (i, c)| {
            let Cell::Unsolved(ccands) = c else { return p };
            if let Some(pi) = p {
                let prow: [Cell; 9] = self[pi / 9];
                let pcell = prow[pi % 9];
                if let Cell::Unsolved(pcands) = pcell {
                    if ccands.iter().filter(|t| **t).count() < pcands.iter().filter(|t| **t).count()
                    {
                        return Some(i);
                    }
                }
                return p;
            } else {
                return Some(i);
            }
        });

        let Some(i) = target else { return None };
        let Cell::Unsolved(cands) = self[i / 9][i % 9] else {
            return None;
        };

        let mut copy = [[Cell::default(); 9]; 9];
        for cand in cands
            .iter()
            .enumerate()
            .filter_map(|(i, t)| if *t { Some(i) } else { None })
        {
            copy.copy_from_slice(self);
            copy[i / 9][i % 9] = Cell::Solved(cand);
            trace!("Trying a {} in R{}C{}...", cand + 1, i / 9, i % 9);
            trace!("{}", PrintableGrid(copy));
            while !copy.solved() {
                if let Some(_) = copy.step() {
                    trace!("{}", PrintableGrid(copy));
                } else if copy.broken() {
                    trace!("Backtracking failed, backing up");
                    break;
                } else if !copy.solved() {
                    trace!("Backtracking further...");
                    if let Some(()) = copy.backtrack() {
                        return Some(());
                    }
                }
            }
            if copy.solved() {
                info!("Solution found!\n{}", PrintableGrid(copy));
                return Some(());
            }
        }

        None
    }
}

#[derive(Parser, Debug)]
#[command()]
struct Args {
    /// File to read from.
    /// If omitted, the sudoku will be read from stdin
    input: Option<String>,
    /// Enables backtracking when no logical steps remain
    #[arg(short, long)]
    backtracking: bool,
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

    let mut grid = [[Cell::default(); 9]; 9];
    for (i, char) in input.replace([' ', '\n', '\t'], "").chars().enumerate() {
        if i >= 81 {
            break;
        }
        grid[i / 9][i % 9] = Cell::new(char.to_digit(10).map(|d| d as usize));
    }

    trace!("initial grid: \n{}", PrintableGrid(grid));

    let mut failed = false;
    while !grid.solved() && !failed {
        if let Some(_) = grid.step() {
            trace!("{}", PrintableGrid(grid));
        } else {
            failed = true;
        }
    }
    if !failed {
        info!("Puzzle solved!");
        return Ok(());
    }

    info!("Failed to find a solution.");
    if args.backtracking {
        trace!("Starting backtracking");
        if let Some(_) = grid.backtrack() {
            info!("Solved!")
        } else {
            info!("Failed to solve puzzle")
        }
    } else {
        info!("Run with --backtracking to try again with backtracking enabled");
    }
    Ok(())
}
