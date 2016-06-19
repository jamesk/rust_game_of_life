#[macro_use]
extern crate log;
extern crate env_logger;

extern crate piston;
extern crate opengl_graphics;
extern crate graphics;
extern crate sdl2_window;
extern crate rust_game_of_life;

use opengl_graphics::{GlGraphics, OpenGL};
use graphics::Graphics;
use graphics::grid::Grid;
use graphics::line::Line;
use graphics::math::Matrix2d;
use graphics::rectangle;
use std::collections::HashMap;
use piston::window::WindowSettings;
use sdl2_window::Sdl2Window as Window;
use piston::input::*;
use piston::event_loop::*;
use graphics::clear;
use rust_game_of_life::section::*;
use rust_game_of_life::board::*;
use std::cmp;
use std::sync::{Arc, Mutex};

fn do_life(sectionTop: Arc<Mutex<BoardSection>>, sectionBottom: Arc<Mutex<BoardSection>>, iteration: usize) {
    let cell = Cell::new(false, iteration, false);
    
    {
    	let mut section = sectionTop.lock().unwrap();
    	
	    let cells = vec![&cell; section.get_board().get_width() as usize];
	    section.update(BoardSectionSide::Top, &cells);
	    
	    let cells = vec![&cell; section.get_board().get_height() as usize];
	    section.update(BoardSectionSide::Left, &cells);
	    section.update(BoardSectionSide::Right, &cells);
	    
	    section.try_iteration(iteration);	
    }
    
	{
		let mut section = sectionBottom.lock().unwrap();
		
		let cells = vec![&cell; section.get_board().get_width() as usize];
		section.update(BoardSectionSide::Bottom, &cells);
	
	    let cells = vec![&cell; section.get_board().get_height() as usize];
	    section.update(BoardSectionSide::Left, &cells);
	    section.update(BoardSectionSide::Right, &cells);
	
	    section.try_iteration(iteration);
	}
}

fn draw_state<G>(board: &Board,
                 iteration: usize,
                 grid: &Grid,
                 x_offset: u32, y_offset: u32,
                 cell_size: f64,
                 transform: Matrix2d,
                 g: &mut G)
    where G: Graphics
{
    let colour = [0.0, 1.0, 0.0, 1.0]; // green

    // TODO: extract cell iterator?
    for x in 0..board.get_width() {
        for y in 0..board.get_height() {
            let c = board.get_cell(x, y);
            
            let cell_rectangle = [grid.x_pos((x + x_offset, y + y_offset)), grid.y_pos((x + x_offset, y + y_offset)), cell_size, cell_size];

            if c.alive {
                rectangle(colour, cell_rectangle, transform, g);
            }

            {
            	trace!("On iteration [{}], cell's iteration is [{}] ({}, {})", iteration, c.get_iteration(), x, y);
            	const AGE_LEVELS: usize = 3;
            	const AGE_MAX_DARK: f32 = 0.75;
            	const AGE_DARK_INCREMENT: f32 = AGE_MAX_DARK / ((AGE_LEVELS - 1) as f32);
            	
                let age = cmp::min(AGE_LEVELS - 1, iteration - c.get_iteration());
                let age_dark = AGE_DARK_INCREMENT * (age as f32);
                trace!("Age of cell is [{}], setting darkness to [{}]. Note, dark increment is [{}]", age, age_dark, AGE_DARK_INCREMENT);
                let age_colour = [0.0, 0.0, 0.0, age_dark];

                rectangle(age_colour, cell_rectangle, transform, g);
            }
        }
    }
}

fn main() {
    env_logger::init().unwrap();

    info!("starting up");

    let mut alives = HashMap::new();
    //Glider
    alives.insert((1, 8), true);
	alives.insert((2, 8), true);
	alives.insert((3, 8), true);
	alives.insert((3, 7), true);
	alives.insert((2, 6), true);

    let board = Board::new(10, 10, &alives);
    let mut sectionTop = Arc::new(Mutex::new(LocalBoardSection::new(board)));

    let mut alives = HashMap::new();
    let board = Board::new(10, 10, &alives);
    let mut sectionBottom = Arc::new(Mutex::new(LocalBoardSection::new(board)));

	let bottom_callback: Box<Fn(&[&Cell])> = {
		let sb = sectionBottom.clone();
		
		Box::new(move |cells: &[&Cell]| {
			sb.lock().unwrap().update(BoardSectionSide::Top, cells);
		})
	};
	let bottom_subscribe = CellStateCallback::new(0, bottom_callback);
	sectionTop.lock().unwrap().subscribe(BoardSectionSide::Bottom, bottom_subscribe);
	
	let top_callback: Box<Fn(&[&Cell])> = {
		let st = sectionTop.clone();
		
			Box::new(move |cells: &[&Cell]| {
			st.lock().unwrap().update(BoardSectionSide::Bottom, cells);
		})
	};
	let top_subscribe = CellStateCallback::new(0, top_callback);
	sectionBottom.lock().unwrap().subscribe(BoardSectionSide::Top, top_subscribe);

	let total_rows = sectionTop.lock().unwrap().get_board().get_height() + sectionBottom.lock().unwrap().get_board().get_height();
	let total_columns = sectionTop.lock().unwrap().get_board().get_width();  

    let window_width = 500;
    let window_height = 500;
    let opengl = OpenGL::V3_2;
    let mut window: Window = WindowSettings::new("Hello World!", [window_width, window_height])
        .opengl(opengl)
        .build()
        .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));
    let ref mut gl = GlGraphics::new(opengl);

    let max_cell_size_x = window_width as f64 / total_rows as f64;
    let max_cell_size_y = window_height as f64 / total_columns as f64;

    let cell_size = f64::min(max_cell_size_x, max_cell_size_y);
    //    let grid_width = (window_width as f64 / cell_size).floor() as u32;
    //    let grid_height = (window_height as f64 / cell_size).floor() as u32;
    let grid = Grid {
        rows: total_rows,
        cols: total_columns,
        units: cell_size,
    };
    let grid_line = Line::new([0.0, 0.0, 0.0, 1.0], 1.0);

    let mut events = window.events().max_fps(1);
    let mut iteration = 0;
    let mut tick = 0;
    
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, g| {
                do_life(sectionTop.clone(), sectionBottom.clone(), iteration);
                
                clear([1.0, 1.0, 1.0, 1.0], g);

                draw_state(sectionTop.lock().unwrap().get_board(),
                           iteration,
                           &grid,
                           0, 0,
                           cell_size,
                           c.transform,
                           g);

				draw_state(sectionBottom.lock().unwrap().get_board(),
                           iteration,
                           &grid,
                           0, sectionTop.lock().unwrap().get_board().get_height() - 1,
                           cell_size,
                           c.transform,
                           g);

                grid.draw(&grid_line, &c.draw_state, c.transform, g);
                
                tick+=1;
                iteration+=if tick % 3 == 0 { 1 } else { 0 };
            });
        }
    }
}
