use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::channel;
use std::cmp;
use board::Cell;

#[derive(Copy, Clone)]
pub struct Rectangle {
    start_x: u32,
    start_y: u32,
    width: u32,
    height: u32,
}

impl Rectangle {
    pub fn new(start_x: u32, start_y: u32, width: u32, height: u32) -> Rectangle {
        Rectangle {
            start_x: start_x,
            start_y: start_y,
            width: width,
            height: height,
        }
    }

    pub fn get_start_x(&self) -> u32 {
        self.start_x
    }

    pub fn get_end_x(&self) -> u32 {
        self.start_x + self.width
    }

    pub fn get_start_y(&self) -> u32 {
        self.start_y
    }

    pub fn get_end_y(&self) -> u32 {
        self.start_y + self.height
    }
}

struct ViewReceiver {
	covered: Rectangle,
	receiver: Receiver<Box<[Box<[Cell]>]>>,
	last_known: Option<Box<[Box<[Cell]>]>>,
}

pub struct BoardView {
	view: Rectangle,
	part_receivers: Vec<ViewReceiver>,
}

impl BoardView {
	pub fn new(view: Rectangle, registerers: Box<[(Rectangle, Sender<Sender<Box<[Box<[Cell]>]>>>)]>) -> BoardView {
		let part_receivers = BoardView::create_links(registerers);
		
		BoardView {
			view: view,
			part_receivers: part_receivers,
		}
	}
	
	fn create_links(registerers: Box<[(Rectangle, Sender<Sender<Box<[Box<[Cell]>]>>>)]>) -> Vec<ViewReceiver>  {
		let mut parts = Vec::new();
		
		//TODO: nicer way to do this? do you just have to have copy on all the types?
		//Can't pattern match the tuple bc its a reference
		for tup in registerers.iter() {
			let covered = tup.0;
			let sender = &tup.1;
			
			/*
			assert!(covered.get_start_x() <= self.view.get_start_x());
			assert!(covered.get_end_x() >= self.view.get_end_x());
			assert!(covered.get_start_y() <= self.view.get_start_y());
			assert!(covered.get_end_y() <= self.view.get_end_y());
			*/
			
			let (tx, rx) = channel();
			
			let vr = ViewReceiver {
				covered: covered,
				receiver: rx,
				last_known: None
			};
			
			sender.send(tx);
			
			parts.push(vr);
		}
		
		parts
	}
	
	pub fn foreach_cell(&mut self, callback: &mut FnMut(Option<Cell>, u32, u32)) {
		//Get latest updates (if any)
		for part_receiver in &mut self.part_receivers {
			loop {
	            match part_receiver.receiver.try_recv() {
	                Ok(cells) => {
	                    part_receiver.last_known = Some(cells);
	                }
	                Err(TryRecvError::Empty) => {
	                    break;
	                }
	                Err(TryRecvError::Disconnected) => {
	                    panic!("Section disconnected its state sender");
	                }
	            }
	        }
		}
		
		//Call foreach on each view
		for part_receiver in &self.part_receivers {
			self.foreach_cell_in_view(
				(part_receiver.covered, &part_receiver.last_known),
				callback
			);
		}
	}
	
	fn foreach_cell_in_view(&self, part: (Rectangle, &Option<Box<[Box<[Cell]>]>>), callback: &mut FnMut(Option<Cell>, u32, u32)) {
		let covered = part.0;
		let cells: &Option<Box<[Box<[Cell]>]>> = part.1;

		//Assumes that part's rectangle overlaps our view rectangle (otherwise start could be > width/height)		
		let start_i = self.view.get_start_x().checked_sub(covered.get_start_x()).unwrap_or(0);
		let start_j = self.view.get_start_y().checked_sub(covered.get_start_y()).unwrap_or(0);
		
		let end_i = covered.get_end_x().checked_sub(self.view.get_end_x()).and_then(|ei| covered.width.checked_sub(ei)).unwrap_or(covered.width);
		let end_j = covered.get_end_y().checked_sub(self.view.get_end_y()).and_then(|ej| covered.height.checked_sub(ej)).unwrap_or(covered.height);

		for i in start_i..end_i {
			for j in start_j..end_j {
				let cell = cells.as_ref().map(|cs| cs[i as usize][j as usize]);
				
				callback(cell, i + covered.get_start_x(), j + covered.get_start_y());
			}
		}
	}
}




