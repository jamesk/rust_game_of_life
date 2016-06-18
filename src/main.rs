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

fn do_life(section: &mut BoardSection, iteration: usize) {
    assert_eq!(section.get_board().get_height(),
               section.get_board().get_width());

    let cell = Cell::new(false, iteration + 1, false);
    let cells = vec![&cell; section.get_board().get_height() as usize];

    section.update(BoardSectionSide::Top, &cells);
    section.update(BoardSectionSide::Bottom, &cells);
    section.update(BoardSectionSide::Left, &cells);
    section.update(BoardSectionSide::Right, &cells);

    section.try_iteration();

    // let mut cells = Vec::new();
    // for x in 0..board.width {
    // let mut col = Vec::new();
    //
    // for y in 0..board.height {
    // let c = board.next_cell(x, y).expect("looping through board, got a None for next cell");
    //
    // col.push(c);
    // }
    //
    // cells.push(col.into_boxed_slice());
    // }
    //
}

fn draw_state<G>(board: &Board,
                 iteration: usize,
                 grid: &Grid,
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
            let cell_rectangle = [grid.x_pos((x, y)), grid.y_pos((x, y)), cell_size, cell_size];

            if c.alive {
                rectangle(colour, cell_rectangle, transform, g);
            }

            {
            	trace!("On iteration [{}], cell's iteration is [{}] ({}, {})", iteration, c.get_iteration(), x, y);
            	const age_levels: usize = 3;
            	const age_max_dark: f32 = 0.75;
            	const age_dark_increment: f32 = age_max_dark / ((age_levels - 1) as f32);
            	
                let age = cmp::min(age_levels - 1, iteration - c.get_iteration());
                let age_dark = age_dark_increment * (age as f32);
                trace!("Age of cell is [{}], setting darkness to [{}]. Note, dark increment is [{}]", age, age_dark, age_dark_increment);
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
    // Blinker
    alives.insert((5, 5), true);
    alives.insert((6, 5), true);
    alives.insert((7, 5), true);

    let board = Board::new(10, 10, &alives);
    let mut section = LocalBoardSection::new(board);

    let window_width = 500;
    let window_height = 500;
    let opengl = OpenGL::V3_2;
    let mut window: Window = WindowSettings::new("Hello World!", [window_width, window_height])
        .opengl(opengl)
        .build()
        .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));
    let ref mut gl = GlGraphics::new(opengl);

    let max_cell_size_x = window_width as f64 / section.get_board().get_width() as f64;
    let max_cell_size_y = window_height as f64 / section.get_board().get_height() as f64;

    let cell_size = f64::min(max_cell_size_x, max_cell_size_y);
    //    let grid_width = (window_width as f64 / cell_size).floor() as u32;
    //    let grid_height = (window_height as f64 / cell_size).floor() as u32;
    let grid = Grid {
        rows: section.get_board().get_width(),
        cols: section.get_board().get_height(),
        units: cell_size,
    };
    let grid_line = Line::new([0.0, 0.0, 0.0, 1.0], 1.0);

    let mut events = window.events().max_fps(1);
    let mut iteration = 0;
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, g| {
                clear([1.0, 1.0, 1.0, 1.0], g);

                draw_state(section.get_board(),
                           iteration,
                           &grid,
                           cell_size,
                           c.transform,
                           g);

                grid.draw(&grid_line, &c.draw_state, c.transform, g);
                
                let lagged_iteration = if iteration % 2 == 1 { iteration - 1 } else { iteration };
                do_life(&mut section, iteration);
                iteration+=1;
            });
        }
    }
}
