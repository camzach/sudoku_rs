use log::{info, trace};

use crate::grid::{Cell, Grid};

type Strategy = fn(&mut Grid) -> bool;

pub struct Solver {
    strategies: Vec<Strategy>,
}
impl Solver {
    pub fn new() -> Solver {
        Solver { strategies: vec![] }
    }
    pub fn add_strategy(&mut self, strategy: Strategy) {
        self.strategies.push(strategy);
    }

    pub fn step(&self, grid: &mut Grid) -> bool {
        self.strategies.iter().find(|strat| strat(grid)).is_some()
    }
    pub fn backtrack(&self, grid: &mut Grid) -> bool {
        let target = grid.iter().flatten().enumerate().fold(None, |p, (i, c)| {
            let Cell::Unsolved(ccands) = c else { return p };
            if let Some(pi) = p {
                let prow: [Cell; 9] = grid[pi / 9];
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

        let Some(i) = target else { return false };
        let Cell::Unsolved(cands) = grid[i / 9][i % 9] else {
            return false;
        };

        let mut copy = Grid([[Cell::default(); 9]; 9]);
        for cand in cands
            .iter()
            .enumerate()
            .filter_map(|(i, t)| if *t { Some(i) } else { None })
        {
            copy.copy_from_slice(&(*grid).0);
            copy[i / 9][i % 9] = Cell::Solved(cand);
            trace!("Trying a {} in R{}C{}...", cand + 1, i / 9, i % 9);
            trace!("{}", copy);
            while !copy.solved() {
                if self.step(&mut copy) {
                    trace!("{}", copy);
                } else if copy.broken() {
                    trace!("Backtracking failed, backing up");
                    break;
                } else if !copy.solved() {
                    trace!("Backtracking further...");
                    if self.backtrack(&mut copy) {
                        return true;
                    }
                }
            }
            if copy.solved() {
                info!("Solution found!\n{}", copy);
                return true;
            }
        }

        false
    }
}
