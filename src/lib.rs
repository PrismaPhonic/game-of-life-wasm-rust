extern crate cfg_if;
extern crate wasm_bindgen;

mod utils;

use std::fmt;
use wasm_bindgen::prelude::*;

extern crate web_sys;

macro_rules! log {
    ( $( $t:tt )*  ) => {
        web_sys::console::log_1(&format!( $( $t )*  ).into());
    }
}

impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn get_row(&self, index: usize) -> u32 {
        index as u32 / self.width
    }

    fn get_col(&self, index: usize) -> u32 {
        index as u32 % self.width
    }

    /// Generate spaceship based on clicked index
    fn gen_spaceship(&mut self, row: u32, col: u32) {
        self.set_cells(&[
            (row + 1, col + 2),
            (row + 2, col + 3),
            (row + 3, col + 1),
            (row + 3, col + 2),
            (row + 3, col + 3),
        ]);
    }

    /// Get the dead and alive values of the entire universe.
    pub fn get_cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells[idx] = Cell::Alive;
        }
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }

    // Generates a pulsar by providing an index we are at along with
    // a starting and ending col for our choosen pulsar - we will
    // return whether the given index in relation to the starting
    // coordinate is a dead or alive cell for initial pulsar gen
    fn gen_pulsar(&mut self, index: usize) {
        let start_row = self.get_row(index) - 6;
        let start_col = self.get_col(index) - 6;
        let index = self.get_index(start_row, start_col);
        for row in 0..13 {
            for col in 0..13 {
                let idx = (index as u32 + row * self.width + col) as usize;
                if row == 1 || row == 6 || row == 11 {
                    self.cells[idx].kill();
                } else if row == 0 || row == 5 || row == 7 || row == 12 {
                    if col >= 2 && col <= 4 || col >= 8 && col <= 10 {
                        self.cells[idx].birth();
                    } else {
                        self.cells[idx].kill();
                    }
                } else {
                    if col == 0 || col == 5 || col == 7 || col == 12 {
                        self.cells[idx].birth();
                    } else {
                        self.cells[idx].kill();
                    }
                }
            }
        }
    }
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

impl Cell {
    fn toggle(&mut self) {
        *self = match *self {
            Cell::Dead => Cell::Alive,
            Cell::Alive => Cell::Dead,
        };
    }

    fn birth(&mut self) {
        *self = Cell::Alive;
    }

    fn kill(&mut self) {
        *self = Cell::Dead;
    }
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

#[wasm_bindgen]
impl Universe {
    pub fn new() -> Universe {
        utils::set_panic_hook();

        let width = 64;
        let height = 64;

        let cells = (0..width * height)
            .map(|i| {
                if i % 2 == 0 || i % 7 == 0 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        self.cells = (0..width * self.height).map(|_| Cell::Dead).collect();
    }

    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.cells = (0..self.width * height).map(|_| Cell::Dead).collect();
    }

    /// Clear the universe (all cells dead)
    pub fn kill_all(&mut self) {
        self.cells = (0..self.width * self.height).map(|_| Cell::Dead).collect();
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells[idx].toggle();
    }

    pub fn add_pulsar(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.gen_pulsar(idx);
    }

    pub fn add_spaceship(&mut self, row: u32, column: u32) {
        self.gen_spaceship(row, column);
    }

    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                // log!(
                //     "cell[{}, {}] is initially {:?} and has {} live neighbors",
                //     row,
                //     col,
                //     cell,
                //     live_neighbors
                // );

                let next_cell = match (cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live
                    // neighbors dies, as if caused by underpopulation
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    // Rule 2: Any live cell with two or three live
                    // neighbors lives on to the next generation
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    // Rule 3: Any live cell with more than three live
                    // neighbors dies, as if by overpopulation
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    // Rule 4: Any dead cell with exactly three live
                    // neighbors becomes a live cell, as if by repreoduction
                    (Cell::Dead, 3) => Cell::Alive,
                    // All other cells remain in same state
                    (otherwise, _) => otherwise,
                };

                // log!("    it becomes {:?}", next_cell);

                next[idx] = next_cell;
            }
        }

        self.cells = next;
    }

    pub fn render(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}
