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

    fn iter_cols_mut(&mut self) -> Vec<Vec<&mut Cell>>;
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

    fn iter_cols_mut(&mut self) -> Vec<Vec<&mut Cell>> {
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
        // println!("Searching for naked singles");
        let mut result = None;

        for ref mut cell in self.into_iter().flatten() {
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
        // println!("Attempting basic elimination");
        let mut result = None;

        // rows
        for row in self.iter_mut() {
            for i in 0..9 {
                if let Cell::Solved(n) = row[i] {
                    for j in 0..9 {
                        if let Cell::Unsolved(ref mut candidates) = row[j] {
                            if candidates[n] {
                                candidates[n] = false;
                                result = Some(());
                            }
                        }
                    }
                }
            }
        }

        // cols
        for col in self.iter_cols_mut() {
            let found: Vec<usize> = col
                .iter()
                .filter_map(|cell| {
                    if let Cell::Solved(n) = cell {
                        Some(*n)
                    } else {
                        None
                    }
                })
                .collect();
            for cell in col {
                for n in found.iter() {
                    if let Cell::Unsolved(ref mut candidates) = cell {
                        if candidates[*n] {
                            candidates[*n] = false;
                            result = Some(());
                        }
                    }
                }
            }
        }

        // boxes
        for bx in self.iter_boxes() {
            let found: Vec<usize> = bx
                .iter()
                .filter_map(|cell| {
                    if let Cell::Solved(n) = cell {
                        Some(*n)
                    } else {
                        None
                    }
                })
                .collect();
            for cell in bx {
                for n in found.iter() {
                    if let Cell::Unsolved(ref mut candidates) = cell {
                        if candidates[*n] {
                            candidates[*n] = false;
                            result = Some(());
                        }
                    }
                }
            }
        }

        result
    }
    fn hidden_singles(&mut self) -> Option<()> {
        let mut result = None;
        for row in self {
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
                        *cell = Cell::Solved(i);
                    }
                }
            }
        }
        result
    }

    fn step(&mut self) -> Option<()> {
        if let Some(_) = self.naked_singles() {
            println!("Naked singles");
        } else if let Some(_) = self.basic_elimination() {
            println!("Basic elimination");
        } else if let Some(_) = self.hidden_singles() {
            println!("Hidden singles");
        } else {
            return None;
        }
        Some(())
    }

    fn backtrack(&mut self) -> Option<()> {
        let mut copy = [[Cell::default(); 9]; 9];
        copy.copy_from_slice(self);

        // (cell, (row, col), cands, count)
        let mut target: Option<(&mut Cell, (usize, usize), Vec<usize>, usize)> = None;

        for (i, cell) in copy.iter_mut().flatten().enumerate() {
            if let Cell::Unsolved(clues) = cell {
                let cands: Vec<_> = clues
                    .clone()
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, b)| if b { Some(i + 1) } else { None })
                    .collect();
                let numcands = cands.clone().len();
                let coords = ((i / 9) + 1, (i % 9) + 1);
                match target {
                    Some((_, _, _, other)) if other > numcands => {
                        target = Some((&mut (*cell), coords, cands, numcands))
                    }
                    None => target = Some((&mut (*cell), coords, cands, numcands)),
                    _ => {}
                }
            }
        }

        if let Some((cell, (row, col), cands, _)) = target {
            if let Some(guess) = cands.first() {
                println!("Trying {} at R{}C{}...", guess + 1, row, col);
                *cell = Cell::Solved(*guess);
            }
        }

        let mut failed = false;
        while !copy.solved() && !failed {
            if let None = copy.step() {
                failed = true;
            }
        }
        // copy.display();

        Some(())
    }
}

const PUZZLE: &str = "
_________
1________
________1
_________
_________
___1_____
_________
_________
_____1___
";

fn main() -> Result<(), ()> {
    let mut grid = [[Cell::new(None); 9]; 9];
    for (row, line) in PUZZLE.trim().lines().enumerate() {
        for (col, c) in line.chars().enumerate() {
            if row >= 9 || col >= 9 {
                continue;
            }
            grid[row][col] = Cell::new(c.to_digit(10).map(|d| d as usize))
        }
    }

    println!("initial grid: \n{}", PrintableGrid(grid));

    let mut failed = false;
    while !grid.solved() && !failed {
        if let Some(_) = grid.step() {
            println!("{}", PrintableGrid(grid));
        } else {
            failed = true;
        }
    }
    if !failed {
        println!("Puzzle solved!");
        return Ok(());
    }

    // println!("Failed to find a solution. Try backtracking? (Y/N)");
    // let mut buffer = String::new();
    // std::io::stdin().read_line(&mut buffer).map_err(|_| ())?;
    // if buffer.trim().to_lowercase() == "y" {
    //     grid.backtrack();
    //     println!("done");
    // }
    Ok(())
}
