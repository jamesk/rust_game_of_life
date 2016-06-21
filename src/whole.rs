use section::*;
use board::Cell;
use std::sync::{Arc, Mutex};
use std::cmp;

pub struct Whole {
    sections: Vec<Vec<Arc<Mutex<Box<BoardSection>>>>>,
}

impl Whole {
    pub fn new(mut sections: Vec<Vec<Box<BoardSection>>>) -> Whole {
        // TODO: assert columns have same height?

        let safe_sections = sections.drain(0..)
            .map(|mut row| {
                row.drain(0..)
                    .map(|s| Arc::new(Mutex::new(s)))
                    .collect()
            })
            .collect();

        Whole::connect_sections(&safe_sections);

        Whole { sections: safe_sections }
    }

    fn get_callback(side: BoardSectionSide,
                    section: &Arc<Mutex<Box<BoardSection>>>)
                    -> Box<Fn(&[&Cell])> {
        let s = section.clone();

        Box::new(move |cells: &[&Cell]| {
            s.lock().unwrap().update(side, cells);
        })
    }

    fn connect_sections(sections: &Vec<Vec<Arc<Mutex<Box<BoardSection>>>>>) {
        for (x, column) in sections.iter().enumerate() {
            for (y, section) in column.iter().enumerate() {
                for ox in x.checked_sub(1) {
                    for other_section in sections.get(ox).and_then(|c| c.get(y)) {
                        let side = BoardSectionSide::Left;
                        let callback = CellStateCallback::new((ox, y),
                                                              Whole::get_callback(side,
                                                                                  other_section));

                        section.lock().unwrap().subscribe(side, callback);
                    }
                }

                for ox in x.checked_add(1) {
                    for other_section in sections.get(ox).and_then(|c| c.get(y)) {
                        let side = BoardSectionSide::Right;
                        let callback = CellStateCallback::new((ox, y),
                                                              Whole::get_callback(side,
                                                                                  other_section));

                        section.lock().unwrap().subscribe(side, callback);
                    }
                }

                for oy in y.checked_sub(1) {
                    for other_section in sections.get(x).and_then(|c| c.get(oy)) {
                        let side = BoardSectionSide::Top;
                        let callback = CellStateCallback::new((x, oy),
                                                              Whole::get_callback(side,
                                                                                  other_section));

                        section.lock().unwrap().subscribe(side, callback);
                    }
                }

                for oy in y.checked_add(1) {
                    for other_section in sections.get(x).and_then(|c| c.get(oy)) {
                        let side = BoardSectionSide::Bottom;
                        let callback = CellStateCallback::new((x, oy),
                                                              Whole::get_callback(side,
                                                                                  other_section));

                        section.lock().unwrap().subscribe(side, callback);
                    }
                }
            }
        }
    }

	pub fn sections_width(&self) -> usize {
		self.sections.len()
	}
	
	pub fn sections_height(&self) -> usize {
		self.sections.get(0).map(|c| c.len()).unwrap_or(0)
	}

    pub fn columns_count(&self) -> usize {
        let section_width = self.sections
            .get(0)
            .and_then(|c| {
                c.get(0).map(|s| {
                    let raw_width = s.lock().unwrap().get_board().get_width() as usize;

                    // Take off 2 for the joining columns
                    cmp::max(0, raw_width - 1)
                })
            })
            .unwrap_or(0);

        let raw_width = self.sections.len() * section_width;

        // Add 2 for the far left/right columns
        if raw_width > 0 {
            raw_width + 1
        } else {
            0
        }
    }

    pub fn rows_count(&self) -> usize {
        let section_height = self.sections
            .get(0)
            .and_then(|c| {
                c.get(0).map(|s| {
                    let raw_height = s.lock().unwrap().get_board().get_height() as usize;

                    // Take off 2 for the joining columns
                    cmp::max(0, raw_height - 1)
                })
            })
            .unwrap_or(0);

        let raw_height = self.sections.get(0).map(|c| c.len()).unwrap_or(0) * section_height;
        // Add 2 for the top and bottom rows
        if raw_height > 0 {
            raw_height + 1
        } else {
            0
        }
    }
    
    pub fn foreach_cell(&self, callback: &Fn(Cell, u32, u32)) {
    	for (sx, col) in self.sections.iter().enumerate() {
    		for (sy, sec) in col.iter().enumerate() {
    			let s = sec.lock().unwrap();
    			let b = s.get_board();
    			
    			let offset_x = ((sx as u32) * (b.get_width() - 1)) + 1;
    			let offset_y = ((sy as u32) * (b.get_height() - 1)) + 1;
    			
    			let (start_x, offset_x) = if sx == 0 { (0, offset_x-1) } else { (1, offset_x) };
    			let (start_y, offset_y) = if sy == 0 { (0, offset_y-1) } else { (1, offset_y) };

    			for x in start_x..b.get_width() {
    				for y in start_y..b.get_height() {
    					let &cell = b.get_cell(x, y);
    					
    					callback(cell, offset_x + x, offset_y + y);
    				}
    			}
    		}
    	}
    }
    
    pub fn get_section(&self, x: usize, y: usize) -> Arc<Mutex<Box<BoardSection>>> {
    	self.sections[x][y].clone()
    }
}
