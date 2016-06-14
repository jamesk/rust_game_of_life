extern crate piston;
extern crate opengl_graphics;
extern crate graphics;
extern crate sdl2_window;

use opengl_graphics::{ GlGraphics, OpenGL };
use graphics::{ Context, Graphics };
use graphics::grid::Grid;
use graphics::line::Line;
use graphics::math::Matrix2d;
use graphics::rectangle;
use std::collections::HashMap;
use piston::window::{ AdvancedWindow, WindowSettings };
use sdl2_window::Sdl2Window as Window;
use piston::input::*;
use piston::event_loop::*;
use graphics::clear;
use graphics::draw_state::DrawState;

#[derive(Copy, Clone, Debug)]
struct Cell {
	alive: bool,
}

struct Board {
	width: u32,
	height: u32,
	cells: Box<[Box<[Cell]>]>
}

impl Board {
	pub fn new(width: u32, height: u32, alive_cells: &HashMap<(u32, u32), bool>) -> Board {
		let mut cells = Vec::new();
		for x in 0..width {
			let mut col = Vec::new();
			
			for y in 0..height {
				let false_pointer = &false;
				let alive = alive_cells.get(&(x, y)).unwrap_or(false_pointer); //TODO: had lifetime issue
				let c = Cell { alive: *alive };
				
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
	
	pub fn neighbour_alive_count(&self, x: u32, y: u32) -> u8 {
		let mut count = 0;
		
		for x_offset in 0..3 {
			for y_offset in 0..3 {
				if !(x_offset == 1 && y_offset == 1) {
					for xi in x.checked_sub(1).and_then(|x| x.checked_add(x_offset)) {
						for yi in y.checked_sub(1).and_then(|y| y.checked_add(y_offset)) {
							match self.get_cell_option(xi, yi) {
								Some(c) if c.alive => count += 1,
								_ => (),
							}
						}
					}
				}
			}
		}
		
		count
	}
	
	//    Any live cell with fewer than two live neighbours dies, as if caused by under-population.
	//    Any live cell with two or three live neighbours lives on to the next generation.
	//    Any live cell with more than three live neighbours dies, as if by over-population.
	//    Any dead cell with exactly three live neighbours becomes a live cell, as if by reproduction.
	pub fn should_be_alive(&self, x: u32, y: u32) -> Option<bool> {
		self.get_cell_option(x, y).map(|c| {
			let alive_neighbours = self.neighbour_alive_count(x, y);
		
			if c.alive {
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
		})
	}
	
	pub fn next_cell(&self, x: u32, y: u32) -> Option<Cell> {
		self.should_be_alive(x, y).map(|alive| Cell { alive: alive })
	}
}

fn do_life(board: &Board) -> Board {
	let mut cells = Vec::new();
	for x in 0..board.width {
		let mut col = Vec::new();
		
		for y in 0..board.height {
			let c = board.next_cell(x, y).expect("looping through board, got a None for next cell");
			
			col.push(c);
		}
		
		cells.push(col.into_boxed_slice());
	}
	
	Board {
		width: board.width,
		height: board.height,
		cells: cells.into_boxed_slice(),
	}
}

fn draw_state<G>(board: &Board, grid: &Grid, cell_size: f64, transform: Matrix2d, g: &mut G) where G: Graphics {
	let colour = [0.0, 1.0, 0.0, 1.0]; // green
	
	//TODO: extract cell iterator?
	for x in 0..board.width {
		for y in 0..board.height {
			let c = board.get_cell(x, y);
			
			if c.alive {
				rectangle(colour, 
                      [grid.x_pos((x, y)), grid.y_pos((x, y)), cell_size, cell_size], // rectangle
                      transform, g);
			}
		}
	}
}

fn main() {
	let mut alives = HashMap::new();
	//Blinker
	alives.insert((5, 5), true);
	alives.insert((6, 5), true);
	alives.insert((7, 5), true);
    
    //Glider
    alives.insert((5, 10), true);
	alives.insert((6, 10), true);
	alives.insert((7, 10), true);
	alives.insert((7, 9), true);
	alives.insert((6, 8), true);
	
    //Pulsar
    alives.insert((20, 2), true);
	alives.insert((21, 2), true);
	alives.insert((22, 2), true);
	alives.insert((26, 2), true);
	alives.insert((27, 2), true);
	alives.insert((28, 2), true);
	
	alives.insert((18, 4), true);
	alives.insert((18, 5), true);
	alives.insert((18, 6), true);
	
	alives.insert((23, 4), true);
	alives.insert((23, 5), true);
	alives.insert((23, 6), true);
	
	alives.insert((25, 4), true);
	alives.insert((25, 5), true);
	alives.insert((25, 6), true);
	
	alives.insert((30, 4), true);
	alives.insert((30, 5), true);
	alives.insert((30, 6), true);
    
    alives.insert((20, 7), true);
	alives.insert((21, 7), true);
	alives.insert((22, 7), true);
	alives.insert((26, 7), true);
	alives.insert((27, 7), true);
	alives.insert((28, 7), true);
    
    
    alives.insert((20, 9), true);
	alives.insert((21, 9), true);
	alives.insert((22, 9), true);
	alives.insert((26, 9), true);
	alives.insert((27, 9), true);
	alives.insert((28, 9), true);
	
	alives.insert((18, 10), true);
	alives.insert((18, 11), true);
	alives.insert((18, 12), true);
	
	alives.insert((23, 10), true);
	alives.insert((23, 11), true);
	alives.insert((23, 12), true);
	
	alives.insert((25, 10), true);
	alives.insert((25, 11), true);
	alives.insert((25, 12), true);
	
	alives.insert((30, 10), true);
	alives.insert((30, 11), true);
	alives.insert((30, 12), true);
    
    alives.insert((20, 14), true);
	alives.insert((21, 14), true);
	alives.insert((22, 14), true);
	alives.insert((26, 14), true);
	alives.insert((27, 14), true);
	alives.insert((28, 14), true);
    
	let mut board = Board::new(50, 50, &alives);
	
	let next = board.neighbour_alive_count(6, 4);
	
	println!("next cell is {:?}", next);
	
	let window_width = 500;
	let window_height = 500;
	let opengl = OpenGL::V3_2;
    let mut window: Window =
        WindowSettings::new("Hello World!", [window_width, window_height])
        	.opengl(opengl).build()
        	.unwrap_or_else(|e| { panic!("Failed to build PistonWindow: {}", e) });
    let ref mut gl = GlGraphics::new(opengl);
     
    let max_cell_size_x = window_width as f64 / board.width as f64;
    let max_cell_size_y = window_height as f64 / board.height as f64;
    
    let cell_size = f64::min(max_cell_size_x, max_cell_size_y);
//    let grid_width = (window_width as f64 / cell_size).floor() as u32;
//    let grid_height = (window_height as f64 / cell_size).floor() as u32;
    let grid = Grid { rows: board.width, cols: board.height, units: cell_size};
    let grid_line = Line::new([0.0, 0.0, 0.0, 1.0], 1.0);
    
    let mut events = window.events().max_fps(3);
    while let Some(e) = events.next(&mut window) {
    	if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, g| {
	    		board = do_life(&board);
	        		
	            clear([1.0, 1.0, 1.0, 1.0], g);
	
				draw_state(&board, &grid, cell_size, c.transform, g);
	           
	            grid.draw(&grid_line,
	            		&c.draw_state,
	            		c.transform,
	            		g);
	        });
    	}
    }
}

#[test]
fn board_new_sets_alive_cells() {
	let mut alives = HashMap::new();
	//Blinker
	alives.insert((5, 5), true);
	
	let board = Board::new(50, 50, &alives);
	
	assert!(board.get_cell(5, 5).alive);
}
