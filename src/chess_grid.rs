use crate::grid::{Cell, Grid};

trait ChessGrid {
    fn kings(&mut self) -> Option<()>;
}

impl ChessGrid for Grid {
    fn kings(&mut self) -> Option<()> {
        let mut result = None;
        for r in 0..9 {
            for c in 0..9 {
                if let Cell::Solved(n) = self[r][c] {
                    for rr in [
                        r.checked_sub(1),
                        Some(r),
                        if r + 1 < 9 { Some(r + 1) } else { None },
                    ] {
                        for cc in [
                            c.checked_sub(1),
                            Some(c),
                            if c + 1 < 9 { Some(c + 1) } else { None },
                        ] {
                            let (Some(rr), Some(cc)) = (rr, cc) else {
                                continue;
                            };
                            if rr == r && cc == c {
                                continue;
                            }
                            if self[rr][cc].remove_candidate(n) {
                                result = Some(());
                            }
                        }
                    }
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod test {
    use crate::{
        chess_grid::ChessGrid,
        grid::{Cell, Grid},
    };

    #[test]
    fn kings() {
        let mut grid = Grid([[Cell::default(); 9]; 9]);

        grid[0][0] = Cell::Solved(0);
        grid[4][4] = Cell::Solved(0);
        grid[8][8] = Cell::Solved(0);

        assert!(matches!(grid.kings(), Some(())));

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
}
