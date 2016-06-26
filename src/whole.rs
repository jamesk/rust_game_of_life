use section::*;
use board::Board;
use board::Cell;
use std::sync::Arc;
use std::cmp;
use std::sync::mpsc::Sender;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc;
use std::collections::HashMap;
use view::Rectangle;

pub struct Whole {
    sections: Vec<Vec<Box<BoardSection>>>,
}

impl Whole {
	pub fn new(sections: Vec<Vec<Box<BoardSection>>>) -> Whole {
		Whole {
			sections: sections
		}
	}
	
    pub fn create_sections(section_width: u32,
                           section_height: u32,
                           whole_size: usize)
                           -> (
                           	Vec<Vec<Box<BoardSection>>>,
                           	HashMap<BoardSectionSide, Vec<SyncSender<Arc<Vec<Cell>>>>>,
                           	Box<[(Rectangle, Sender<Sender<Box<[Box<[Cell]>]>>>)]>
   ) {
        let (mut sections, registerers) = Whole::create_sections_sub(section_height, section_width, whole_size);
        Whole::connect_sections(&mut sections);

        let edge_senders = Whole::create_edge_senders(whole_size, &mut sections);

        (sections, edge_senders, registerers)
    }

    fn create_sections_sub(section_width: u32,
                           section_height: u32,
                           whole_size: usize)
                           -> (Vec<Vec<Box<BoardSection>>>, Box<[(Rectangle, Sender<Sender<Box<[Box<[Cell]>]>>>)]>) {
       	let mut registerers = Vec::with_capacity(whole_size * whole_size);
        let mut rows = Vec::with_capacity(whole_size);

        for x in 0..whole_size {
            let mut col: Vec<Box<BoardSection>> = Vec::with_capacity(whole_size);

            for y in 0..whole_size {

                let mut alives = HashMap::new();

                if x == 0 && y == 0 {
                    // Glider
                    alives.insert((3, 5), true);
                    alives.insert((4, 5), true);
                    alives.insert((5, 5), true);
                    alives.insert((5, 4), true);
                    alives.insert((4, 3), true);
                }

                let board = Board::new(section_width, section_height, &alives);
                let (section, registerer) = LocalBoardSection::create(board);
                
                let start_x = (x as u32) * section_width;
                let start_y = (y as u32) * section_height;
                let area = Rectangle::new(start_x, start_y, section_width, section_height);
                
                registerers.push((area, registerer));
                col.push(Box::new(section));
            }

            rows.push(col);
        }

        (rows, registerers.into_boxed_slice())
    }

    fn create_edge_senders(whole_size: usize,
                           sections: &mut Vec<Vec<Box<BoardSection>>>)
                           -> HashMap<BoardSectionSide, Vec<SyncSender<Arc<Vec<Cell>>>>> {
        let edges_count = whole_size * 4;
        let max_section_index = whole_size - 1;
        let mut senders = HashMap::with_capacity(edges_count);
        for x in 0..whole_size {
            // Top edge
            {
                let section = &mut sections[x][0];
                let sender = Whole::create_sender(BoardSectionSide::Top, section);

                let c = senders.entry(BoardSectionSide::Top)
                    .or_insert_with(|| Vec::with_capacity(whole_size));

                c.push(sender);
            }

            // Bottom edge
            {
                let section = &mut sections[x][max_section_index];
                let sender = Whole::create_sender(BoardSectionSide::Bottom, section);

                let c = senders.entry(BoardSectionSide::Bottom)
                    .or_insert_with(|| Vec::with_capacity(whole_size));

                c.push(sender);
            }
        }

        for y in 0..whole_size {
            // Left edge
            {
                let section = &mut sections[0][y];
                let sender = Whole::create_sender(BoardSectionSide::Left, section);

                let c = senders.entry(BoardSectionSide::Left)
                    .or_insert_with(|| Vec::with_capacity(whole_size));

                c.push(sender);
            }

            // Right edge
            {
                let section = &mut sections[max_section_index][y];
                let sender = Whole::create_sender(BoardSectionSide::Right, section);

                let c = senders.entry(BoardSectionSide::Right)
                    .or_insert_with(|| Vec::with_capacity(whole_size));

                c.push(sender);
            }
        }

        senders
    }

    fn create_sender(side: BoardSectionSide,
                     section: &mut Box<BoardSection>)
                     -> SyncSender<Arc<Vec<Cell>>> {
        let (tx, rx) = mpsc::sync_channel(1);

        section.add_receiver(side, rx);

        tx
    }

    fn connect_sections(sections: &mut Vec<Vec<Box<BoardSection>>>) {
        for x in 0..sections.len() {
            for y in 0..sections[x].len() {
                {
                    let (top, bottom) = sections[x].split_at_mut(y + 1);
                    for section in top.last_mut() {
                        for other_section in bottom.first_mut() {
                            let side = BoardSectionSide::Bottom;
                            let other_side = BoardSectionSide::Top;

                            let callback =
                                CellStateCallback::new((x, y), Whole::create_sender(side, section));
                            other_section.subscribe(other_side, callback);

                            let other_callback =
                                CellStateCallback::new((x, y + 1),
                                                       Whole::create_sender(other_side,
                                                                            other_section));
                            section.subscribe(side, other_callback);
                        }
                    }
                }

                {
                    let (left, right) = sections.split_at_mut(x + 1);

                    for left_col in left.last_mut() {
                        for right_col in right.first_mut() {
                            for section in left_col.get_mut(y) {
                                for other_section in right_col.get_mut(y) {
                                    let side = BoardSectionSide::Right;
                                    let other_side = BoardSectionSide::Left;

                                    let callback =
                                        CellStateCallback::new((x, y),
                                                               Whole::create_sender(side, section));
                                    other_section.subscribe(other_side, callback);

                                    let other_callback =
                                        CellStateCallback::new((x + 1, y),
                                                               Whole::create_sender(other_side,
                                                                                    other_section));
                                    section.subscribe(side, other_callback);
                                }
                            }
                        }
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
                    let raw_width = s.get_board().get_width() as usize;

                    // Take off 2 for the joining columns
                    cmp::max(0, raw_width - 2)
                })
            })
            .unwrap_or(0);

        let raw_width = self.sections.len() * section_width;

        // Add 2 for the far left/right columns
        if raw_width > 0 {
            raw_width + 2
        } else {
            0
        }
    }

    pub fn rows_count(&self) -> usize {
        let section_height = self.sections
            .get(0)
            .and_then(|c| {
                c.get(0).map(|s| {
                    let raw_height = s.get_board().get_height() as usize;

                    // Take off 2 for the joining rows
                    cmp::max(0, raw_height - 2)
                })
            })
            .unwrap_or(0);

        let raw_height = self.sections.get(0).map(|c| c.len()).unwrap_or(0) * section_height;
        // Add 2 for the top and bottom rows
        if raw_height > 0 {
            raw_height + 2
        } else {
            0
        }
    }

    pub fn foreach_cell(&self, callback: &mut FnMut(Cell, u32, u32)) {
        for (sx, col) in self.sections.iter().enumerate() {
            for (sy, sec) in col.iter().enumerate() {
                let b = sec.get_board();

                let offset_x = (sx as u32) * (b.get_width() - 2);
                let offset_y = (sy as u32) * (b.get_height() - 2);

                let start_x = if sx == 0 {
                    0
                } else {
                    2
                };
                let start_y = if sy == 0 {
                    0
                } else {
                    2
                };

                for x in start_x..b.get_width() {
                    for y in start_y..b.get_height() {
                        let &cell = b.get_cell(x, y);

                        callback(cell, offset_x + x, offset_y + y);
                    }
                }
            }
        }
    }

    pub fn get_section(&mut self, x: usize, y: usize) -> &mut Box<BoardSection> {
        &mut self.sections[x][y]
    }
}
