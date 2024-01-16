use log::trace;

use crate::grid::{Cell, Grid};

trait CheckedAdd {
    fn sudoku_add(&self, other: usize) -> Option<usize>;
}
impl CheckedAdd for usize {
    fn sudoku_add(&self, other: usize) -> Option<usize> {
        if self + other < 9 {
            Some(self + other)
        } else {
            None
        }
    }
}

pub fn kings(grid: &mut Grid) -> bool {
    trace!("Searching for kings");
    let mut result = false;
    for r in 0..9 {
        for c in 0..9 {
            if let Cell::Solved(n) = grid[r][c] {
                for rr in [r.checked_sub(1), Some(r), r.sudoku_add(1)] {
                    for cc in [c.checked_sub(1), Some(c), c.sudoku_add(1)] {
                        let (Some(rr), Some(cc)) = (rr, cc) else {
                            continue;
                        };
                        if rr == r && cc == c {
                            continue;
                        }
                        if grid[rr][cc].remove_candidate(n) {
                            result = true;
                        }
                    }
                }
            }
        }
    }
    result
}

pub fn knights(grid: &mut Grid) -> bool {
    trace!("Searching for knights");
    let mut result = false;
    for r in 0..9 {
        for c in 0..9 {
            if let Cell::Solved(n) = grid[r][c] {
                for rr in [r.checked_sub(2), r.sudoku_add(2)] {
                    for cc in [c.checked_sub(1), r.sudoku_add(1)] {
                        let (Some(rr), Some(cc)) = (rr, cc) else {
                            continue;
                        };
                        if rr == r && cc == c {
                            continue;
                        }
                        if grid[rr][cc].remove_candidate(n) {
                            result = true;
                        }
                    }
                }
                for rr in [r.checked_sub(1), r.sudoku_add(1)] {
                    for cc in [c.checked_sub(2), r.sudoku_add(2)] {
                        let (Some(rr), Some(cc)) = (rr, cc) else {
                            continue;
                        };
                        if rr == r && cc == c {
                            continue;
                        }
                        if grid[rr][cc].remove_candidate(n) {
                            result = true;
                        }
                    }
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod test {
    use crate::{
        chess_strategies::{kings, knights},
        grid::{Cell, Grid},
    };

    #[test]
    fn test_kings() {
        let mut grid = Grid([[Cell::default(); 9]; 9]);

        grid[0][0] = Cell::Solved(0);
        grid[4][4] = Cell::Solved(0);
        grid[8][8] = Cell::Solved(0);

        assert!(kings(&mut grid));

        assert!([grid[1][0], grid[1][1], grid[0][1]]
            .iter()
            .all(|c| matches!(c, Cell::Unsolved([false, _, _, _, _, _, _, _, _]))));

        assert!([
            grid[3][3], grid[3][4], grid[3][5], grid[4][3], grid[4][5], grid[5][3], grid[5][4],
            grid[5][5],
        ]
        .iter()
        .all(|c| matches!(c, Cell::Unsolved([false, _, _, _, _, _, _, _, _]))));

        assert!([grid[7][7], grid[7][8], grid[8][7]]
            .iter()
            .all(|c| matches!(c, Cell::Unsolved([false, _, _, _, _, _, _, _, _]))));
    }
    #[test]
    fn test_knights() {
        let mut grid = Grid([[Cell::default(); 9]; 9]);

        grid[0][0] = Cell::Solved(0);
        grid[4][4] = Cell::Solved(0);
        grid[8][8] = Cell::Solved(0);

        assert!(knights(&mut grid));

        assert!([grid[2][1], grid[1][2]]
            .iter()
            .all(|c| matches!(c, Cell::Unsolved([false, _, _, _, _, _, _, _, _]))));

        assert!([
            grid[3][2], grid[2][3], grid[5][3], grid[3][5], grid[2][5], grid[5][2], grid[5][6],
            grid[6][5]
        ]
        .iter()
        .all(|c| matches!(c, Cell::Unsolved([false, _, _, _, _, _, _, _, _]))));

        assert!([grid[6][5], grid[5][6]]
            .iter()
            .all(|c| matches!(c, Cell::Unsolved([false, _, _, _, _, _, _, _, _]))));
    }
}
