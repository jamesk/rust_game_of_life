use std::collections::HashMap;

#[derive(Copy, Clone, Debug)]
pub struct Cell {
    pub alive: bool, //TODO: write getter
    iteration: usize,
    previous_alive: bool,
}

impl Cell {
	pub fn new(alive: bool, iteration: usize, previous_alive: bool) -> Cell {
		Cell {
			alive: alive,
			iteration: iteration,
			previous_alive: previous_alive,
		}
	}
	
	pub fn get_iteration(&self) -> usize {
		self.iteration
	}
}

pub struct Board {
    width: u32,
    height: u32,
    cells: Box<[Box<[Cell]>]>,
}

impl Board {
    pub fn new(width: u32, height: u32, alive_cells: &HashMap<(u32, u32), bool>) -> Board {
        let mut cells = Vec::new();
        for x in 0..width {
            let mut col = Vec::new();

            for y in 0..height {
                let false_pointer = &false;
                let alive = alive_cells.get(&(x, y)).unwrap_or(false_pointer);
                // TODO: had lifetime issue
                let c = Cell {
                    alive: *alive,
                    iteration: 0,
                    previous_alive: false,
                };

                col.push(c);
            }

            cells.push(col.into_boxed_slice());
        }

        Board {
            width: width,
            height: height,
            cells: cells.into_boxed_slice(),
        }
    }

    pub fn get_cell(&self, x: u32, y: u32) -> &Cell {
        &self.cells[x as usize][y as usize]
    }

    pub fn set_cell(&mut self, x: u32, y: u32, cell: Cell) {
        self.cells[x as usize][y as usize] = cell;
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn within_bounds(&self, x: u32, y: u32) -> bool {
        x < self.width && y < self.height
    }

    pub fn get_cell_option(&self, x: u32, y: u32) -> Option<&Cell> {
        if self.within_bounds(x, y) {
            Some(self.get_cell(x, y))
        } else {
            None
        }
    }

    pub fn neighbour_alive_count(&self, x: u32, y: u32, iteration: usize) -> Option<u8> {
        let mut count = 0;

        for x_offset in 0..3 {
            for y_offset in 0..3 {
                if !(x_offset == 1 && y_offset == 1) {
                    for xi in x.checked_sub(1).and_then(|x| x.checked_add(x_offset)) {
                        for yi in y.checked_sub(1).and_then(|y| y.checked_add(y_offset)) {
                            match self.get_cell_option(xi, yi) {
                                Some(c) => {
                                    let alive = if c.iteration == iteration {
                                        c.alive
                                    } else if iteration > 0 && c.iteration == iteration - 1 {
                                        c.previous_alive
                                    } else {
                                        return None;
                                    };

                                    if alive {
                                        count += 1
                                    }
                                }
                                None => {}
                            }
                        }
                    }
                }
            }
        }

        Some(count)
    }

    // Any live cell with fewer than two live neighbours dies, as if caused by under-population.
    // Any live cell with two or three live neighbours lives on to the next generation.
    // Any live cell with more than three live neighbours dies, as if by over-population.
    // Any dead cell with exactly three live neighbours becomes a live cell, as if by reproduction.
    pub fn should_be_alive(alive: bool, alive_neighbours: u8) -> bool {
        if alive {
            if alive_neighbours < 2 {
                false
            } else if alive_neighbours > 3 {
                false
            } else {
                true
            }
        } else {
            alive_neighbours == 3
        }

    }

    pub fn next_cell(&self, x: u32, y: u32) -> Option<Cell> {
        self.get_cell_option(x, y).and_then(|c| {
            let alive_count_option = self.neighbour_alive_count(x, y, c.iteration);

            let should_be_alive =
                alive_count_option.map(|alive_count| Board::should_be_alive(c.alive, alive_count));

            should_be_alive.map(|alive| {
                Cell {
                    alive: alive,
                    iteration: c.iteration + 1,
                    previous_alive: c.alive,
                }
            })
        })
    }
}
