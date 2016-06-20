use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use board::Cell;
use board::Board;

pub struct CellStateCallback {
	id: usize,
    callback: Box<Fn(&[&Cell])>,
}

impl CellStateCallback {
	pub fn new(id: usize, callback: Box<Fn(&[&Cell])>) -> CellStateCallback {
		CellStateCallback {
			id: id,
			callback: callback
		}
	}
	
	pub fn run(&self, cells: &[&Cell]) {
		let c = &self.callback;
		
		c(cells);
	}
}

impl Hash for CellStateCallback {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for CellStateCallback {
    fn eq(&self, other: &CellStateCallback) -> bool {
        self.id == other.id
    }

    fn ne(&self, other: &CellStateCallback) -> bool {
        self.id != other.id
    }
}

impl Eq for CellStateCallback {}

#[derive(PartialEq, Eq, Hash)]
pub enum BoardSectionSide {
    Top,
    Bottom,
    Left,
    Right,
}

pub trait BoardSection {
    fn subscribe(&mut self, side: BoardSectionSide, callback: CellStateCallback);
    fn unsubscribe(&mut self, side: BoardSectionSide, callback: CellStateCallback);

    fn update(&mut self, side: BoardSectionSide, cells: &[&Cell]);
    fn try_iteration(&mut self, upto_iteration: usize);
    fn get_board(&self) -> &Board;
}

pub struct LocalBoardSection {
    board: Board,

    subscribes: HashMap<BoardSectionSide, HashSet<CellStateCallback>>,
}

impl LocalBoardSection {
    pub fn new(board: Board) -> LocalBoardSection {
        LocalBoardSection {
            board: board,
            subscribes: HashMap::new(),
        }
    }
}

impl BoardSection for LocalBoardSection {
    fn get_board(&self) -> &Board {
        &self.board
    }

    fn subscribe(&mut self, side: BoardSectionSide, callback: CellStateCallback) {
        let callbacks = self.subscribes.entry(side).or_insert_with(|| HashSet::new());
        callbacks.insert(callback);
    }

    fn unsubscribe(&mut self, side: BoardSectionSide, callback: CellStateCallback) {
        match self.subscribes.get_mut(&side) {
            Some(callbacks) => callbacks.remove(&callback),
            None => false,
        };
    }

    fn update(&mut self, side: BoardSectionSide, cells: &[&Cell]) {
        // TODO: actual implementation to update state
        // TODO: check the cells array has right length?
        match side {
            BoardSectionSide::Top => {
                for (x, &update_cell) in cells.iter().enumerate() {
                    let stable_coord = 0;
                    let &copy = update_cell;

                    self.board.set_cell(x as u32, stable_coord, copy);
                }
            }
            BoardSectionSide::Bottom => {
                for (x, &update_cell) in cells.iter().enumerate() {
                    let stable_coord = self.board.get_height() - 1;
                    let &copy = update_cell;

                    self.board.set_cell(x as u32, stable_coord, copy);
                }
            }
            BoardSectionSide::Left => {
                for (y, &update_cell) in cells.iter().enumerate() {
                    let stable_coord = 0;
                    let &copy = update_cell;

                    self.board.set_cell(stable_coord, y as u32, copy);
                }
            }
            BoardSectionSide::Right => {
                for (y, &update_cell) in cells.iter().enumerate() {
                    let stable_coord = self.board.get_width() - 1;
                    let &copy = update_cell;

                    self.board.set_cell(stable_coord, y as u32, copy);
                }
            }
        }
    }

    fn try_iteration(&mut self, upto_iteration: usize) {
        // update each cell if possible, ordering is important?
        for x in 1..self.board.get_width() - 1 {
            for y in 1..self.board.get_height() - 1 {
                let &current = self.board.get_cell(x, y);

                if current.get_iteration() < upto_iteration {
                    match self.board.next_cell(x, y, &current) {
                        Some(next) => self.board.set_cell(x, y, next),
                        None => {
                            debug!("Unable to update a cell due to old neighbours. Cell at [{}] \
                                    x and [{}] y is [{:?}]",
                                   x,
                                   y,
                                   current);
                        }
                    }
                } else {
                    trace!("Not updating cell due to upto iteration limit. Cell at [{}] x and \
                            [{}] y is [{:?}]",
                           x,
                           y,
                           current);
                }
            }
        }

        // callback subscribers
        for callbacks in self.subscribes.get(&BoardSectionSide::Top) {
            let mut cells = Vec::with_capacity(self.board.get_width() as usize);
            for x in 0..self.board.get_width() {
                cells.push(self.board.get_cell(x, 1))
            }
            let cells = &cells;

            for callback in callbacks {
                callback.run(cells);
            }
        }
        for callbacks in self.subscribes.get(&BoardSectionSide::Bottom) {
            let mut cells = Vec::with_capacity(self.board.get_width() as usize);
            for x in 0..self.board.get_width() {
                cells.push(self.board.get_cell(x, self.board.get_height() - 2))
            }
            let cells = &cells;

            for callback in callbacks {
                callback.run(cells);
            }
        }
        for callbacks in self.subscribes.get(&BoardSectionSide::Left) {
            let mut cells = Vec::with_capacity(self.board.get_height() as usize);
            for y in 0..self.board.get_height() {
                cells.push(self.board.get_cell(1, y))
            }
            let cells = &cells;

            for callback in callbacks {
                callback.run(cells);
            }
        }
        for callbacks in self.subscribes.get(&BoardSectionSide::Right) {
            let mut cells = Vec::with_capacity(self.board.get_height() as usize);
            for y in 0..self.board.get_height() {
                cells.push(self.board.get_cell(self.board.get_width() - 2, y))
            }
            let cells = &cells;

            for callback in callbacks {
                callback.run(cells);
            }
        }
    }
}
