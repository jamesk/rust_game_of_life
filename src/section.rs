use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::TrySendError;
use std::sync::Arc;

use board::Cell;
use board::Board;

pub struct CellStateCallback {
    id: (usize, usize),
    sender: SyncSender<Arc<Vec<Cell>>>,
}

impl CellStateCallback {
    pub fn new(id: (usize, usize), sender: SyncSender<Arc<Vec<Cell>>>) -> CellStateCallback {
        CellStateCallback {
            id: id,
            sender: sender,
        }
    }

    pub fn try_send(&self, cells: Arc<Vec<Cell>>) -> Result<(), TrySendError<Arc<Vec<Cell>>>> {
        self.sender.try_send(cells)
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

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum BoardSectionSide {
    Top,
    Bottom,
    Left,
    Right,
}

pub trait BoardSection {
    fn subscribe(&mut self, side: BoardSectionSide, callback: CellStateCallback);
    fn unsubscribe(&mut self, side: BoardSectionSide, callback: &CellStateCallback);

    fn add_receiver(&mut self, side: BoardSectionSide, rx: Receiver<Arc<Vec<Cell>>>);
    fn try_iteration(&mut self, upto_iteration: usize);
    fn get_board(&self) -> &Board;
}

pub struct LocalBoardSection {
    board: Board,

    subscribes: HashMap<BoardSectionSide, HashSet<CellStateCallback>>,

    receivers: HashMap<BoardSectionSide, Receiver<Arc<Vec<Cell>>>>,

    state_sender_registerer: Receiver<Sender<Box<[Box<[Cell]>]>>>,

    state_senders: Vec<Sender<Box<[Box<[Cell]>]>>>,
}

impl LocalBoardSection {
    pub fn create(board: Board) -> (LocalBoardSection, Sender<Sender<Box<[Box<[Cell]>]>>>) {
        let (tx, rx) = channel();

        let section = LocalBoardSection {
            board: board,
            subscribes: HashMap::new(),
            receivers: HashMap::new(),
            state_sender_registerer: rx,
            state_senders: Vec::new(),
        };

        (section, tx)
    }

    fn update(board: &mut Board, side: BoardSectionSide, cells: Arc<Vec<Cell>>) {
        // TODO: actual implementation to update state
        // TODO: check the cells array has right length?
        match side {
            BoardSectionSide::Top => {
                for (x, &update_cell) in cells.iter().enumerate() {
                    let stable_coord = 0;

                    board.set_cell(x as u32, stable_coord, update_cell);
                }
            }
            BoardSectionSide::Bottom => {
                for (x, &update_cell) in cells.iter().enumerate() {
                    let stable_coord = board.get_height() - 1;

                    board.set_cell(x as u32, stable_coord, update_cell);
                }
            }
            BoardSectionSide::Left => {
                for (y, &update_cell) in cells.iter().enumerate() {
                    let stable_coord = 0;

                    board.set_cell(stable_coord, y as u32, update_cell);
                }
            }
            BoardSectionSide::Right => {
                for (y, &update_cell) in cells.iter().enumerate() {
                    let stable_coord = board.get_width() - 1;

                    board.set_cell(stable_coord, y as u32, update_cell);
                }
            }
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

    fn unsubscribe(&mut self, side: BoardSectionSide, callback: &CellStateCallback) {
        match self.subscribes.get_mut(&side) {
            Some(callbacks) => callbacks.remove(callback),
            None => false,
        };
    }

    fn add_receiver(&mut self, side: BoardSectionSide, rx: Receiver<Arc<Vec<Cell>>>) {
        match self.receivers.insert(side, rx) {
            Some(_) => {
                panic!("Should never call multiple times, at least not for now. Maybe in future \
                        we might move sections around dynamically or something")
            }
            None => {}
        };
    }

    fn try_iteration(&mut self, upto_iteration: usize) {
        // Read updates from other sections we are subscribed to
        {
            let mut board = &mut self.board;

            for (side, rx) in self.receivers.iter() {
                match rx.try_recv() {
                    Ok(cells) => LocalBoardSection::update(board, *side, cells),
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => {} //TODO: do something??
                }

            }
        }

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
                cells.push(*self.board.get_cell(x, 1))
            }

            let cells = Arc::new(cells);

            for sender in callbacks {
                match sender.try_send(cells.clone()) {
                    Ok(_) => {}
                    Err(TrySendError::Full(_)) => {}
                    Err(TrySendError::Disconnected(_)) => {
                        // TODO: unsubscribe
                    }
                }
            }
        }
        for callbacks in self.subscribes.get(&BoardSectionSide::Bottom) {
            let mut cells = Vec::with_capacity(self.board.get_width() as usize);
            for x in 0..self.board.get_width() {
                cells.push(*self.board.get_cell(x, self.board.get_height() - 2))
            }
            let cells = Arc::new(cells);

            for sender in callbacks {
                match sender.try_send(cells.clone()) {
                    Ok(_) => {}
                    Err(TrySendError::Full(_)) => {}
                    Err(TrySendError::Disconnected(_)) => {
                        // TODO: unsubscribe
                    }
                }
            }
        }
        for callbacks in self.subscribes.get(&BoardSectionSide::Left) {
            let mut cells = Vec::with_capacity(self.board.get_height() as usize);
            for y in 0..self.board.get_height() {
                cells.push(*self.board.get_cell(1, y))
            }
            let cells = Arc::new(cells);

            for sender in callbacks {
                match sender.try_send(cells.clone()) {
                    Ok(_) => {}
                    Err(TrySendError::Full(_)) => {}
                    Err(TrySendError::Disconnected(_)) => {
                        // TODO: unsubscribe
                    }
                }
            }
        }
        for callbacks in self.subscribes.get(&BoardSectionSide::Right) {
            let mut cells = Vec::with_capacity(self.board.get_height() as usize);
            for y in 0..self.board.get_height() {
                cells.push(*self.board.get_cell(self.board.get_width() - 2, y))
            }
            let cells = Arc::new(cells);

            for sender in callbacks {
                match sender.try_send(cells.clone()) {
                    Ok(_) => {}
                    Err(TrySendError::Full(_)) => {}
                    Err(TrySendError::Disconnected(_)) => {
                        // TODO: unsubscribe
                    }
                }
            }
        }

        // Update our state senders
        loop {
            match self.state_sender_registerer.try_recv() {
                Ok(state_sender) => {
                    self.state_senders.push(state_sender);
                }
                Err(TryRecvError::Empty) => {
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    break; //We aren't going to get any more
                }
            }
        }

        let mut i = 0;
        while i < self.state_senders.len() {
            let send_result = {
                let sender = &self.state_senders[i];

                let mut cells = Vec::with_capacity(self.board.get_width() as usize);
                for col in self.board.cells.iter() {
                    let mut copy = Vec::with_capacity(col.len());

                    for &cell in col.iter() {
                        copy.push(cell);
                    }

                    cells.push(copy.into_boxed_slice());
                }

                sender.send(cells.into_boxed_slice())
            };

            match send_result {
                Ok(_) => {
                    i += 1;
                }
                Err(_) => {
                    self.state_senders.remove(i);
                }
            }
        }
    }
}
