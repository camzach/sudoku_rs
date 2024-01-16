use std::collections::HashMap;

use log::trace;

use crate::grid::{Cell, Grid};

pub fn naked_singles(grid: &mut Grid) -> bool {
    trace!("Searching for naked singles");
    let mut result = false;

    for ref mut cell in grid.iter_mut().flatten() {
        if let Cell::Unsolved(candidates) = cell {
            if candidates.iter().filter(|t| **t).count() == 1 {
                let n = candidates.iter().position(|c| *c == true).unwrap();
                **cell = Cell::Solved(n);
                result = true;
            }
        }
    }
    result
}
pub fn basic_elimination(grid: &mut Grid) -> bool {
    trace!("Attempting basic elimination");
    let mut result = false;
    // rows
    for row in grid.iter_mut() {
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
                    result = true
                }
            }
        }
    }

    // cols
    for mut col in grid.cols() {
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
                    result = true
                }
            }
        }
    }

    // boxes
    for mut bx in grid.boxes() {
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
                    result = true
                }
            }
        }
    }

    result
}
pub fn hidden_singles(grid: &mut Grid) -> bool {
    trace!("Searching for hidden singles");
    let mut result = false;
    for row in grid.iter_mut() {
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
                result = true;
                for cell in cells {
                    let mut newcands = [false; 9];
                    newcands[i] = true;
                    *cell = Cell::Unsolved(newcands);
                }
            }
        }
    }

    for mut col in grid.cols() {
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
                result = true;
                for cell in cells {
                    let mut newcands = [false; 9];
                    newcands[i] = true;
                    **cell = Cell::Unsolved(newcands);
                }
            }
        }
    }

    for mut bx in grid.boxes() {
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
                result = true;
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
pub fn naked_tuples(grid: &mut Grid) -> bool {
    trace!("Searching for naked tuples");
    let mut result = false;

    for row in grid.0.iter_mut() {
        let mut map: HashMap<Cell, usize> = HashMap::new();
        for cell in row.iter() {
            if let Some(count) = map.get_mut(cell) {
                *count += 1;
            } else {
                map.insert(*cell, 1);
            }
        }

        for id in map.iter().filter_map(|(c, count)| {
            if c.candidates().len() == *count {
                Some(c)
            } else {
                None
            }
        }) {
            let candidates = id.candidates();
            for cell in row.iter_mut().filter(|c| *c != id) {
                for cand in candidates.iter() {
                    if cell.remove_candidate(*cand) {
                        result = true;
                    };
                }
            }
        }
    }

    for col in grid.cols().iter_mut() {
        let mut map: HashMap<Cell, usize> = HashMap::new();
        for cell in col.iter() {
            if let Some(count) = map.get_mut(cell) {
                *count += 1;
            } else {
                map.insert(**cell, 1);
            }
        }

        for id in map.iter().filter_map(|(c, count)| {
            if c.candidates().len() == *count {
                Some(c)
            } else {
                None
            }
        }) {
            let candidates = id.candidates();
            for cell in col.iter_mut().filter(|c| **c != id) {
                for cand in candidates.iter() {
                    if cell.remove_candidate(*cand) {
                        result = true;
                    };
                }
            }
        }
    }
    for bx in grid.boxes().iter_mut() {
        let mut map: HashMap<Cell, usize> = HashMap::new();
        for cell in bx.iter() {
            if let Some(count) = map.get_mut(cell) {
                *count += 1;
            } else {
                map.insert(**cell, 1);
            }
        }

        for id in map.iter().filter_map(|(c, count)| {
            if c.candidates().len() == *count {
                Some(c)
            } else {
                None
            }
        }) {
            let candidates = id.candidates();
            for cell in bx.iter_mut().filter(|c| **c != id) {
                for cand in candidates.iter() {
                    if cell.remove_candidate(*cand) {
                        result = true;
                    };
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use crate::{
        basic_sudoku::{basic_elimination, hidden_singles, naked_singles, naked_tuples},
        Cell, Grid,
    };
    impl Cell {
        fn exact_candidates(&self, candidates: &HashSet<usize>) -> bool {
            if let Cell::Unsolved(c) = self {
                for (i, cand) in c.iter().enumerate() {
                    if *cand != candidates.contains(&i) {
                        return false;
                    }
                }
                return true;
            }
            false
        }
        fn has_candidate(&self, n: usize) -> bool {
            if let Cell::Unsolved(cands) = self {
                cands[n]
            } else {
                false
            }
        }
    }

    #[test]
    fn test_naked_singles() {
        let mut grid = Grid([[Cell::default(); 9]; 9]);
        let mut opts = [false; 9];
        opts[0] = true;
        grid[0][0] = Cell::Unsolved(opts);
        assert!(naked_singles(&mut grid));
        assert!(matches!(grid[0][0], Cell::Solved(0)));
    }

    #[test]
    fn test_basic_elimination() {
        let mut grid = Grid([[Cell::default(); 9]; 9]);
        grid[0][0] = Cell::Solved(0);
        let reduced = HashSet::from([1, 2, 3, 4, 5, 6, 7, 8]);
        let unreduced = HashSet::from([0, 1, 2, 3, 4, 5, 6, 7, 8]);

        assert!(basic_elimination(&mut grid));

        assert!(grid[0].iter().skip(1).all(|c| c.exact_candidates(&reduced)));

        let mut iter_rows = grid.iter();
        assert!(iter_rows.by_ref().skip(1).take(2).all(|row| {
            let mut row_iter = row.iter();
            row_iter
                .by_ref()
                .take(3)
                .all(|c| c.exact_candidates(&reduced))
                && row_iter.all(|c| c.exact_candidates(&unreduced))
        }));
        assert!(iter_rows.all(|row| {
            let mut row_iter = row.iter();
            row_iter
                .by_ref()
                .take(1)
                .all(|c| c.exact_candidates(&reduced))
                && row_iter.all(|c| c.exact_candidates(&unreduced))
        }));
    }

    #[test]
    fn test_hidden_singles() {
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

        assert!(hidden_singles(&mut grid));

        assert!(grid[8][0].exact_candidates(&HashSet::from([0])));
        assert!(grid[0][8].exact_candidates(&HashSet::from([1])));
        assert!(grid[3][3].exact_candidates(&HashSet::from([2])));
    }

    #[test]
    fn test_naked_tuples() {
        let mut grid = Grid([[Cell::default(); 9]; 9]);

        let a = [true, true, false, false, false, false, false, false, false];
        grid[0][2] = Cell::Unsolved(a);
        grid[0][6] = Cell::Unsolved(a);

        let b = [false, false, true, true, false, false, false, false, false];
        grid[1][0] = Cell::Unsolved(b);
        grid[6][0] = Cell::Unsolved(b);

        let c = [false, false, false, false, true, true, false, false, false];
        grid[0][0] = Cell::Unsolved(c);
        grid[1][1] = Cell::Unsolved(c);

        assert!(naked_tuples(&mut grid));

        assert!(grid[0]
            .iter()
            .enumerate()
            .filter_map(|(i, c)| if i == 2 || i == 6 { None } else { Some(c) })
            .all(|c| !c.has_candidate(0) && !c.has_candidate(1)));
        assert!(grid
            .cols()
            .get(0)
            .unwrap()
            .iter()
            .enumerate()
            .filter_map(|(i, c)| if i == 1 || i == 6 { None } else { Some(c) })
            .all(|c| !c.has_candidate(2) && !c.has_candidate(3)));
        assert!(grid
            .boxes()
            .get(0)
            .unwrap()
            .iter()
            .enumerate()
            .filter_map(|(i, c)| if i == 0 || i == 4 { None } else { Some(c) })
            .all(|c| !c.has_candidate(4) && !c.has_candidate(5)));
    }
}
