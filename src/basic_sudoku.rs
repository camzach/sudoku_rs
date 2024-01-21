use std::collections::{HashMap, HashSet};

use itertools::Itertools;

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

    fn process_group(group: Vec<&mut Cell>) -> bool {
        let mut result = false;

        let ns_present = group
            .iter()
            .filter_map(|c| {
                if let Cell::Solved(n) = c {
                    Some(n.clone())
                } else {
                    None
                }
            })
            .collect_vec();
        for cell in group {
            for n in ns_present.iter() {
                if cell.remove_candidate(*n) {
                    result = true
                }
            }
        }
        result
    }
    for row in grid.iter_mut() {
        result |= process_group(row.iter_mut().collect_vec());
    }
    for col in grid.cols() {
        result |= process_group(col);
    }
    for bx in grid.boxes() {
        result |= process_group(bx);
    }
    result
}
pub fn hidden_singles(grid: &mut Grid) -> bool {
    trace!("Searching for hidden singles");
    let mut result = false;

    fn process_group(group: &mut Vec<&mut Cell>) -> bool {
        let mut result = false;
        for i in 0..9 {
            let cells = group
                .iter_mut()
                .filter(|c| {
                    if let Cell::Unsolved(cands) = c {
                        return cands[i];
                    }
                    false
                })
                .collect_vec();
            if cells.len() == 1 {
                result = true;
                for cell in cells {
                    let mut newcands = [false; 9];
                    newcands[i] = true;
                    **cell = Cell::Unsolved(newcands);
                }
            }
        }
        result
    }
    for row in grid.iter_mut() {
        result |= process_group(&mut row.iter_mut().collect_vec());
    }
    for mut col in grid.cols() {
        result |= process_group(&mut col);
    }
    for mut bx in grid.boxes() {
        result |= process_group(&mut bx);
    }

    result
}
pub fn naked_tuples(grid: &mut Grid) -> bool {
    trace!("Searching for naked tuples");
    let mut result = false;

    fn process_group(group: &mut Vec<&mut Cell>) -> bool {
        let mut result = false;
        let mut map: HashMap<Cell, usize> = HashMap::new();
        for cell in group.iter() {
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
            for cell in group.iter_mut().filter(|c| **c != id) {
                for cand in candidates.iter() {
                    if cell.remove_candidate(*cand) {
                        result = true;
                    };
                }
            }
        }

        result
    }
    for row in grid.0.iter_mut() {
        result |= process_group(&mut row.iter_mut().collect_vec());
    }
    for col in grid.cols().iter_mut() {
        result |= process_group(col);
    }
    for bx in grid.boxes().iter_mut() {
        result |= process_group(bx);
    }

    result
}
pub fn hidden_tuples(grid: &mut Grid) -> bool {
    let mut result = false;
    trace!("Searching for hidden tuples");

    fn process_group(group: &mut Vec<&mut Cell>) -> bool {
        let mut result = false;
        let mut map: HashMap<usize, usize> = HashMap::new();
        for cell in group.iter() {
            for cand in cell.candidates() {
                if let Some(count) = map.get_mut(&cand) {
                    *count += 1;
                } else {
                    map.insert(cand, 1);
                }
            }
        }

        for len in 2..=4 {
            let cands = map
                .iter()
                .filter_map(|(k, v)| if v <= &len { Some(k) } else { None })
                .collect_vec();
            let mut cells = group
                .iter_mut()
                .filter(|c| c.candidates().iter().any(|c| cands.contains(&c)))
                .collect_vec();
            if cells.len() == len && cells.len() == cands.len() {
                for cell in cells.iter_mut() {
                    for cand in cell.candidates() {
                        if !cands.contains(&&cand) {
                            cell.remove_candidate(cand);
                        }
                    }
                }
                result = true;
            }
        }
        result
    }
    for row in grid.iter_mut() {
        result |= process_group(&mut row.iter_mut().collect_vec());
    }
    for col in grid.cols().iter_mut() {
        result |= process_group(col);
    }
    for bx in grid.boxes().iter_mut() {
        result |= process_group(bx);
    }

    result
}
pub fn pointing_tuples(grid: &mut Grid) -> bool {
    let mut result = false;

    // cand -> (row/col, box in row/col)
    let mut row_clears: HashMap<usize, HashSet<(usize, usize)>> = HashMap::new();
    let mut col_clears: HashMap<usize, HashSet<(usize, usize)>> = HashMap::new();

    for (i_bx, bx) in grid.boxes().iter().enumerate() {
        let box_row = i_bx / 3;
        let box_col = i_bx % 3;
        let mut row_map: HashMap<usize, HashSet<usize>> = HashMap::new();
        let mut col_map: HashMap<usize, HashSet<usize>> = HashMap::new();
        for (i_cell, cell) in bx.iter().enumerate() {
            for cand in cell.candidates() {
                if let Some(vec) = row_map.get_mut(&cand) {
                    vec.insert(i_cell / 3);
                } else {
                    row_map.insert(cand, HashSet::from([i_cell / 3]));
                }
                if let Some(vec) = col_map.get_mut(&cand) {
                    vec.insert(i_cell % 3);
                } else {
                    col_map.insert(cand, HashSet::from([i_cell % 3]));
                }
            }
        }
        for (k, row_in_box) in row_map.iter().filter_map(|(k, v)| {
            if v.len() == 1 {
                v.iter().last().map(|v| (k, v))
            } else {
                None
            }
        }) {
            let overall_row = row_in_box + (box_row * 3);
            let tuple = (overall_row, box_col);
            if let Some(set) = row_clears.get_mut(k) {
                set.insert(tuple);
            } else {
                row_clears.insert(*k, HashSet::from([tuple]));
            }
        }
        for (k, col_in_box) in col_map.iter().filter_map(|(k, v)| {
            if v.len() == 1 {
                v.iter().last().map(|v| (k, v))
            } else {
                None
            }
        }) {
            let overall_col = col_in_box + (box_col * 3);
            let tuple = (overall_col, box_row);
            if let Some(set) = col_clears.get_mut(k) {
                set.insert(tuple);
            } else {
                col_clears.insert(*k, HashSet::from([tuple]));
            }
        }
    }

    for (cand, set) in row_clears {
        for (row, box_col) in set {
            for (i, cell) in grid.0[row].iter_mut().enumerate() {
                if i / 3 != box_col {
                    cell.remove_candidate(cand);
                    result = true;
                }
            }
        }
    }
    for (cand, set) in col_clears {
        for (col, box_row) in set {
            for (i, cell) in grid.cols().get_mut(col).unwrap().iter_mut().enumerate() {
                if i / 3 != box_row {
                    cell.remove_candidate(cand);
                    result = true;
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use crate::{basic_sudoku::*, Cell, Grid};
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

    #[test]
    fn test_hidden_tuples() {
        let mut grid = Grid([[Cell::default(); 9]; 9]);

        let row_refs: [*const Cell; 2] = [&grid.0[0][4], &grid.0[0][7]];
        for cell in grid.0[0].iter_mut() {
            if row_refs.contains(&(&(*cell) as *const Cell)) {
                continue;
            }
            cell.remove_candidate(0);
            cell.remove_candidate(1);
        }

        let col_refs: [*mut Cell; 3] = [&mut grid.0[2][2], &mut grid.0[4][2], &mut grid.0[7][2]];
        for cell in grid.cols()[2].iter_mut() {
            if col_refs.contains(&(*cell as *mut Cell)) {
                continue;
            }
            cell.remove_candidate(2);
            cell.remove_candidate(3);
            cell.remove_candidate(4);
        }
        unsafe {
            (*col_refs[0]).remove_candidate(3);
        }

        let box_refs: [*const Cell; 2] = [&grid.0[4][4], &grid.0[5][5]];
        for cell in grid.boxes()[4].iter_mut() {
            if box_refs.contains(&(*cell as *const Cell)) {
                continue;
            }
            cell.remove_candidate(5);
            cell.remove_candidate(6);
        }

        assert!(hidden_tuples(&mut grid));

        assert!(row_refs
            .iter()
            .map(|c| unsafe { **c })
            .all(|c| c.exact_candidates(&HashSet::from([0, 1]))));
        assert!(col_refs
            .iter()
            .map(|c| unsafe { **c })
            .enumerate()
            .all(|(i, c)| c.exact_candidates(
                &(if i == 0 {
                    HashSet::from([2, 4])
                } else {
                    HashSet::from([2, 3, 4])
                })
            )));
        assert!(box_refs
            .iter()
            .map(|c| unsafe { **c })
            .all(|c| c.exact_candidates(&HashSet::from([5, 6]))));
    }

    #[test]
    fn test_pointing_tuples() {
        let mut grid = Grid([[Cell::default(); 9]; 9]);

        let row_refs: [*const Cell; 2] = [&grid.0[0][0], &grid.0[0][1]];
        let col_refs: [*const Cell; 3] = [&grid.0[0][2], &grid.0[1][2], &grid.0[2][2]];
        for cell in grid.boxes()[0].iter_mut() {
            let raw_pointer = *cell as *const Cell;
            if !row_refs.contains(&raw_pointer) {
                cell.remove_candidate(0);
            }
            if !col_refs.contains(&raw_pointer) {
                cell.remove_candidate(1);
            }
        }

        assert!(pointing_tuples(&mut grid));

        assert!(grid.0[0].iter().all(|cell| {
            row_refs.contains(&(cell as *const Cell)) == cell.candidates().contains(&0)
        }));

        assert!(grid.cols()[2].iter().all(
            |cell| col_refs.contains(&(*cell as *const Cell)) == cell.candidates().contains(&1)
        ))
    }
}
