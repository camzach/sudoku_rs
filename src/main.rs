use std::{
    fs::read_to_string,
    ops::{Deref, DerefMut},
};

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

struct Grid([[Cell; 9]; 9]);
impl std::fmt::Display for Grid {
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
impl Deref for Grid {
    type Target = [[Cell; 9]; 9];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Grid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Grid {
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

    fn cols(&mut self) -> Vec<Vec<&mut Cell>> {
        self.iter_mut().flatten().enumerate().fold(
            (0..9).map(|_| Vec::new()).collect(),
            |mut p, (i, c)| {
                p.get_mut(i % 9).unwrap().push(c);
                p
            },
        )
    }
    fn boxes(&mut self) -> Vec<Vec<&mut Cell>> {
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
        for mut col in self.cols() {
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
        for mut bx in self.boxes() {
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

        for mut col in self.cols() {
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

        for mut bx in self.boxes() {
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

        let mut copy = Grid([[Cell::default(); 9]; 9]);
        for cand in cands
            .iter()
            .enumerate()
            .filter_map(|(i, t)| if *t { Some(i) } else { None })
        {
            copy.copy_from_slice(&(*self).0);
            copy[i / 9][i % 9] = Cell::Solved(cand);
            trace!("Trying a {} in R{}C{}...", cand + 1, i / 9, i % 9);
            trace!("{}", copy);
            while !copy.solved() {
                if let Some(_) = copy.step() {
                    trace!("{}", copy);
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
                info!("Solution found!\n{}", copy);
                return Some(());
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use crate::{Cell, Grid};

    fn exact_candidates(cell: &Cell, candidates: &HashSet<usize>) -> bool {
        if let Cell::Unsolved(c) = cell {
            for (i, cand) in c.iter().enumerate() {
                if *cand != candidates.contains(&i) {
                    return false;
                }
            }
            return true;
        }
        false
    }

    #[test]
    fn naked_singles() {
        let mut grid = Grid([[Cell::default(); 9]; 9]);
        let mut opts = [false; 9];
        opts[0] = true;
        grid[0][0] = Cell::Unsolved(opts);
        assert!(matches!(grid.naked_singles(), Some(())));
        assert!(matches!(grid[0][0], Cell::Solved(0)));
    }

    #[test]
    fn basic_elimination() {
        let mut grid = Grid([[Cell::default(); 9]; 9]);
        grid[0][0] = Cell::Solved(0);
        let reduced = HashSet::from([1, 2, 3, 4, 5, 6, 7, 8]);
        let unreduced = HashSet::from([0, 1, 2, 3, 4, 5, 6, 7, 8]);

        assert!(matches!(grid.basic_elimination(), Some(())));

        assert!(grid[0]
            .iter()
            .skip(1)
            .all(|c| exact_candidates(c, &reduced)));

        let mut iter_rows = grid.iter();
        assert!(iter_rows.by_ref().skip(1).take(2).all(|row| {
            let mut row_iter = row.iter();
            row_iter
                .by_ref()
                .take(3)
                .all(|c| exact_candidates(c, &reduced))
                && row_iter.all(|c| exact_candidates(c, &unreduced))
        }));
        assert!(iter_rows.all(|row| {
            let mut row_iter = row.iter();
            row_iter
                .by_ref()
                .take(1)
                .all(|c| exact_candidates(c, &reduced))
                && row_iter.all(|c| exact_candidates(c, &unreduced))
        }));
    }

    #[test]
    fn hidden_singles() {
        let mut grid = Grid([[Cell::default(); 9]; 9]);

        for cell in grid[8].iter_mut().skip(1) {
            cell.remove_candidate(0);
        }

        for cell in grid.cols().get_mut(8).unwrap().iter_mut().skip(1) {
            cell.remove_candidate(1);
        }

        for cell in grid.boxes().get_mut(4).unwrap().iter_mut().skip(1) {
            cell.remove_candidate(2);
        }

        assert!(matches!(grid.hidden_singles(), Some(())));

        assert!(exact_candidates(&grid[8][0], &HashSet::from([0])));
        assert!(exact_candidates(&grid[0][8], &HashSet::from([1])));
        assert!(exact_candidates(&grid[3][3], &HashSet::from([2])));
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

    let mut grid = Grid([[Cell::default(); 9]; 9]);
    for (i, char) in input.replace([' ', '\n', '\t'], "").chars().enumerate() {
        if i >= 81 {
            break;
        }
        grid[i / 9][i % 9] = Cell::new(char.to_digit(10).map(|d| d as usize));
    }

    trace!("initial grid: \n{}", grid);

    let mut failed = false;
    while !grid.solved() && !failed {
        if let Some(_) = grid.step() {
            trace!("{}", grid);
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
