use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use board::Cell;
use board::Board;

#[derive(Eq)]
struct CellStateCallback {
    id: usize,
    callback: fn(&[Cell]),
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

#[derive(PartialEq, Eq, Hash)]
enum BoardSectionSide {
    Top,
    Bottom,
    Left,
    Right,
}

trait BoardSection {
    fn subscribe(&mut self, side: BoardSectionSide, callback: CellStateCallback);
    fn unsubscribe(&mut self, side: BoardSectionSide, callback: CellStateCallback);

    fn update(&mut self, side: BoardSectionSide, cells: &[Cell]);
    fn try_iteration(&mut self);
}

struct LocalBoardSection {
    board: Board,

    subscribes: HashMap<BoardSectionSide, HashSet<CellStateCallback>>,
}

impl BoardSection for LocalBoardSection {
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

    fn update(&mut self, side: BoardSectionSide, cells: &[Cell]) {
        // TODO: actual implementation to update state
        //TODO: check the cells array has right length?
        match side {
            BoardSectionSide::Top => {
                for (x, &update_cell) in cells.iter().enumerate() {
                	let stable_coord = 0;
                	
                    self.board.set_cell(x as u32, stable_coord, update_cell);
                }
            }
            BoardSectionSide::Bottom => {
                for (x, &update_cell) in cells.iter().enumerate() {
                	let stable_coord = self.board.get_height() - 1;
                	
                    self.board.set_cell(x as u32, stable_coord, update_cell);
                }
            }
            BoardSectionSide::Left => {
                for (y, &update_cell) in cells.iter().enumerate() {
                	let stable_coord = 0;
                	
                    self.board.set_cell(stable_coord, y as u32, update_cell);
                }
            }
            BoardSectionSide::Right => {
                for (y, &update_cell) in cells.iter().enumerate() {
                	let stable_coord = self.board.get_width() - 1;
                	
                    self.board.set_cell(stable_coord, y as u32, update_cell);
                }
            }
        }
    }

    fn try_iteration(&mut self) {
        // update each cell if possible, ordering is important?
        // callback subscribers
    }
}
