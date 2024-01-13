# Sudoku Solver

This project exists to practice using Rust to solve a real(ish) problem. This sudoku solver solves puzzles via the following iterative process:

1. Attempt a series of logical deductions, in order of increasing complexity
2. If no logical deduction could be made, find the cell with the fewest possible values and for each possibility, do the following steps:
    1. Repeat step 1 until no deductions remain
    2. If the puzzle is now unsolvable, move to the next candidate from step 2
    3. If the puzzle is now solved, we are done!
    4. If the puzzle is still unsolved, repeat step 2 from the current position

The backtracking alone is enough to solve every sudoku puzzle, but the addition of the logical deductions greatly reduces the number of backtracking guesses necessary

In the future, I'd like to extend the solver to include sudoku variants (killer cages, chess sudoku, etc)
