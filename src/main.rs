extern crate piston;
extern crate opengl_graphics;
extern crate graphics;
extern crate sdl2_window;
extern crate rust_game_of_life;

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
use rust_game_of_life::section::*;
use rust_game_of_life::board::*;


fn do_life(section: &mut BoardSection) {
	assert_eq!(
		section.get_board().get_height(),
		section.get_board().get_width()
	);
	
	let iteration = section.get_board().get_cell(0, 0).get_iteration();
	let cell = Cell::new(false, iteration, false);
	let cells = vec![&cell; section.get_board().get_height() as usize];
	
	section.update(BoardSectionSide::Top, &cells);
	section.update(BoardSectionSide::Bottom, &cells);
	section.update(BoardSectionSide::Left, &cells);
	section.update(BoardSectionSide::Right, &cells);
	
	section.try_iteration();
	
	/*let mut cells = Vec::new();
	for x in 0..board.width {
		let mut col = Vec::new();
		
		for y in 0..board.height {
			let c = board.next_cell(x, y).expect("looping through board, got a None for next cell");
			
			col.push(c);
		}
		
		cells.push(col.into_boxed_slice());
	}
	*/
}

fn draw_state<G>(board: &Board, grid: &Grid, cell_size: f64, transform: Matrix2d, g: &mut G) where G: Graphics {
	let colour = [0.0, 1.0, 0.0, 1.0]; // green
	
	//TODO: extract cell iterator?
	for x in 0..board.get_width() {
		for y in 0..board.get_height() {
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
    
	let board = Board::new(50, 50, &alives);
	let mut section = LocalBoardSection::new(board);
	
	let window_width = 500;
	let window_height = 500;
	let opengl = OpenGL::V3_2;
    let mut window: Window =
        WindowSettings::new("Hello World!", [window_width, window_height])
        	.opengl(opengl).build()
        	.unwrap_or_else(|e| { panic!("Failed to build PistonWindow: {}", e) });
    let ref mut gl = GlGraphics::new(opengl);
     
    let max_cell_size_x = window_width as f64 / section.get_board().get_width() as f64;
    let max_cell_size_y = window_height as f64 / section.get_board().get_height() as f64;
    
    let cell_size = f64::min(max_cell_size_x, max_cell_size_y);
//    let grid_width = (window_width as f64 / cell_size).floor() as u32;
//    let grid_height = (window_height as f64 / cell_size).floor() as u32;
    let grid = Grid { rows: section.get_board().get_width(), cols: section.get_board().get_height(), units: cell_size};
    let grid_line = Line::new([0.0, 0.0, 0.0, 1.0], 1.0);
    
    let mut events = window.events().max_fps(3);
    while let Some(e) = events.next(&mut window) {
    	if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, g| {
	    		do_life(&mut section);
	        		
	            clear([1.0, 1.0, 1.0, 1.0], g);
	
				draw_state(section.get_board(), &grid, cell_size, c.transform, g);
	           
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
