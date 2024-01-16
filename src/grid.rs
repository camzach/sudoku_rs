use std::{
    hash::Hash,
    ops::{Deref, DerefMut},
};

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum Cell {
    Solved(usize),
    Unsolved([bool; 9]),
}
impl Cell {
    pub fn remove_candidate(&mut self, n: usize) -> bool {
        if let Cell::Unsolved(cands) = self {
            if cands[n] {
                cands[n] = false;
                return true;
            }
        }
        false
    }
    pub fn candidates(&self) -> Vec<usize> {
        if let Self::Unsolved(candidates) = self {
            candidates
                .iter()
                .enumerate()
                .filter_map(|(i, n)| if *n { Some(i) } else { None })
                .collect()
        } else {
            Vec::new()
        }
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
        None.into()
    }
}
impl From<Option<usize>> for Cell {
    fn from(value: Option<usize>) -> Self {
        value
            .map(|n| Cell::Solved(n - 1))
            .unwrap_or(Cell::Unsolved([true; 9]))
    }
}

pub struct Grid(pub [[Cell; 9]; 9]);
impl std::fmt::Display for Grid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for (i, line) in self.into_iter().enumerate() {
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
    pub fn solved(&self) -> bool {
        self.iter()
            .flatten()
            .all(|c| if let Cell::Solved(_) = c { true } else { false })
    }
    pub fn broken(&self) -> bool {
        self.iter().flatten().any(|cell| {
            if let Cell::Unsolved(cands) = cell {
                cands.iter().all(|t| !t)
            } else {
                false
            }
        })
    }

    pub fn cols(&mut self) -> Vec<Vec<&mut Cell>> {
        self.iter_mut().flatten().enumerate().fold(
            (0..9).map(|_| Vec::new()).collect(),
            |mut p, (i, c)| {
                p.get_mut(i % 9).unwrap().push(c);
                p
            },
        )
    }
    pub fn boxes(&mut self) -> Vec<Vec<&mut Cell>> {
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
}
