extern crate rand;
use rand::distributions::{IndependentSample, Range};
use std::ops::{Index, IndexMut};
use std::{thread, time};

#[derive(Debug)]
struct LifeBoard {
    board_size: usize,
    cells: std::vec::Vec<bool>,
}
struct CellMut<'a> {
    row: usize,
    col: usize,
    state: &'a mut bool,
}

struct Cell<'a> {
    row: usize,
    col: usize,
    state: &'a bool,
}

struct CellMutIterator<'a> {
    board: &'a mut LifeBoard,
    index: usize,
}

struct CellIterator<'a> {
    board: &'a LifeBoard,
    index: usize,
}

impl LifeBoard {
    fn new(board_size: usize) -> LifeBoard {
        let mut cells = Vec::new();
        cells.resize(board_size * board_size, false);

        LifeBoard { board_size, cells }
    }

    fn print(&self) {
        let header_footer = || {
            print!("+");
                for _ in 0..self.board_size {
                print!("--");
            }
            println!("--+");
        };

        header_footer( );
        for r in 0..self.board_size {
            print!("| ");
            for c in 0..self.board_size {
                let board = &["  ", "# "];
                let cell = self[r][c];
                print!("{}", board[cell as usize]);
            }
            println!(" |");
        }
        header_footer( );
    }

    fn iter_mut(&mut self) -> CellMutIterator {
        // THIS IS A HACK TO ENABLE MUTABLE ITERATORS
        // https://stackoverflow.com/questions/25730586/how-can-i-create-my-own-data-structure-with-an-iterator-that-returns-mutable-ref
        unsafe {
            CellMutIterator {
                board: &mut *(self as *mut _),
                index: 0,
            }
        }
    }

    fn iter(&self) -> CellIterator {
        CellIterator {
            board: &self,
            index: 0,
        }
    }

    fn is_neighbour_alive(&self, cell: &Cell, delta_row: isize, delta_col: isize) -> u8 {
        self[(cell.row.wrapping_add(delta_row as usize)) % self.board_size]
            [(cell.col.wrapping_add(delta_col as usize)) % self.board_size] as u8
    }

    fn step(&self, next: &mut LifeBoard) {
        for (source, target) in self.iter().zip(next.iter_mut()) {
            let neighours = &[
                (-1, -1),
                (-1, 0),
                (-1, 1),
                (0, -1),
                (0, 1),
                (1, -1),
                (1, 0),
                (1, 1),
            ];
            let mut count = 0_u8;

            for cood in neighours {
                count += self.is_neighbour_alive(&source, cood.0, cood.1);
            }
            count |= *source.state as u8;
            
            *target.state = count == 3;
        }
    }
}

impl Index<usize> for LifeBoard {
    type Output = [bool];
    fn index(&self, row: usize) -> &[bool] {
        let row_index = row * self.board_size;
        &self.cells[row_index..row_index + self.board_size]
    }
}

impl IndexMut<usize> for LifeBoard {
    fn index_mut(&mut self, row: usize) -> &mut [bool] {
        let row_index = row * self.board_size;
        &mut self.cells[row_index..row_index + self.board_size]
    }
}

impl<'a> std::iter::Iterator for CellMutIterator<'a> {
    type Item = CellMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.board.cells.len() {
            let board_size = self.board.board_size;
            let row = self.index / board_size;
            let col = self.index % board_size;
            let state = &mut self.board.cells[self.index];
            self.index += 1;

            // THIS IS A HACK TO ENABLE MUTABLE ITERATORS
            // https://stackoverflow.com/questions/25730586/how-can-i-create-my-own-data-structure-with-an-iterator-that-returns-mutable-ref
            unsafe {
                Some(CellMut {
                    row,
                    col,
                    state: &mut *(state as *mut _),
                })
            }
        } else {
            None
        }
    }
}

impl<'a> std::iter::Iterator for CellIterator<'a> {
    type Item = Cell<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.board.cells.len() {
            let board_size = self.board.board_size;
            let row = self.index / board_size;
            let col = self.index % board_size;
            let state = &self.board.cells[self.index];
            self.index += 1;

            Some(Cell { row, col, state })
        } else {
            None
        }
    }
}

fn main() {
    let mut x = LifeBoard::new(32);
    let mut y = LifeBoard::new(32);

    let step = Range::new(0, 2);
    let mut rng = rand::thread_rng();

    for cell in x.iter_mut() {
        let state = cell.state;
        *state = step.ind_sample(&mut rng) == 0;
    }

    let ten_millis = time::Duration::from_millis(100);
    for _ in 0..1000 {
        x.step(&mut y);
        x.print();
        thread::sleep(ten_millis);

        y.step(&mut x);
        y.print();
        thread::sleep(ten_millis);
    }
}
