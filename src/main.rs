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
    fn broken(&self) -> bool;

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
    fn broken(&self) -> bool {
        self.iter().flatten().any(|cell| {
            if let Cell::Unsolved(cands) = cell {
                cands.iter().all(|t| !t)
            } else {
                false
            }
        })
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

        for mut col in self.iter_cols_mut() {
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
            println!("Trying a {} in R{}C{}...", cand + 1, i / 9, i % 9);
            println!("{}", PrintableGrid(copy));
            while !copy.solved() {
                if let Some(_) = copy.step() {
                    println!("{}", PrintableGrid(copy));
                } else if copy.broken() {
                    println!("Backtracking failed, backing up");
                    break;
                } else if !copy.solved() {
                    println!("Backtracking further...");
                    if let Some(()) = copy.backtrack() {
                        return Some(());
                    }
                }
            }
            if copy.solved() {
                println!("Solution found!\n{}", PrintableGrid(copy));
                return Some(());
            }
        }

        None
    }
}

const PUZZLE: &str = "
    9...3....
    ...1..5..
    .32..6.8.
    6......9.
    .79.5.8..
    4....7...
    ....6...3
    .4.......
    .87..3.2.
    
    ";

fn main() -> Result<(), ()> {
    let mut grid = [[Cell::new(None); 9]; 9];
    for (i, c) in PUZZLE.replace([' ', '\n', '\t'], "").chars().enumerate() {
        let row = i / 9;
        let col = i % 9;
        if row >= 9 || col >= 9 {
            continue;
        }
        grid[row][col] = Cell::new(c.to_digit(10).map(|d| d as usize))
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

    println!("Failed to find a solution. Try backtracking? (Y/N)");
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).map_err(|_| ())?;
    if buffer.trim().to_lowercase() == "y" {
        if let Some(_) = grid.backtrack() {
            println!("Solved!")
        } else {
            println!("Failed to solve puzzle")
        }
    }
    Ok(())
}
