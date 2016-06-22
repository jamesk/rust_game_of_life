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
use rust_game_of_life::whole::*;
use std::cmp;

// Might be handy later, if not delete
//
// struct ViewBox {
// start_x: u32,
// start_y: u32,
// width: u32,
// height: u32,
// }
//
// impl ViewBox {
// pub fn new(start_x: u32, start_y: u32, width: u32, height: u32) -> ViewBox {
// ViewBox {
// start_x: start_x,
// start_y: start_y,
// width: width,
// height: height,
// }
// }
//
// pub fn get_start_x(&self) -> u32 {
// self.start_x
// }
//
// pub fn get_end_x(&self) -> u32 {
// self.start_x + self.width
// }
//
// pub fn get_start_y(&self) -> u32 {
// self.start_y
// }
//
// pub fn get_end_y(&self) -> u32 {
// self.start_y + self.height
// }
// }
//


fn do_life(whole: &Whole, iteration: usize) {
    let cell = Cell::new(false, iteration, false);

    for x in 0..whole.sections_width() {
        {
            let section_arc = whole.get_section(x, 0);
            let mut section = section_arc.lock().unwrap();

            let cells = vec![&cell; section.get_board().get_width() as usize];
            section.update(BoardSectionSide::Top, &cells);
        }

        {
            // TODO: don't like this pattern, I'm not using section_arc anywhere else
            //      but it is needed for lifetimes to match up though
            let section_arc = whole.get_section(x, whole.sections_height() - 1);
            let mut section = section_arc.lock().unwrap();

            let cells = vec![&cell; section.get_board().get_width() as usize];
            section.update(BoardSectionSide::Bottom, &cells);
        }
    }

    for y in 0..whole.sections_height() {
        {
            let section_arc = whole.get_section(0, y);
            let mut section = section_arc.lock().unwrap();

            let cells = vec![&cell; section.get_board().get_width() as usize];
            section.update(BoardSectionSide::Left, &cells);
        }

        {
            let section_arc = whole.get_section(whole.sections_width() - 1, y);
            let mut section = section_arc.lock().unwrap();

            let cells = vec![&cell; section.get_board().get_width() as usize];
            section.update(BoardSectionSide::Right, &cells);
        }
    }

    for x in 0..whole.sections_width() {
        for y in 0..whole.sections_height() {
            let section_arc = whole.get_section(x, y);

            section_arc.lock().unwrap().try_iteration(iteration);
        }
    }
}

// fn draw_state<G>(board: &Board,
// view: ViewBox,
// iteration: usize,
// grid: &Grid,
// x_offset: u32,
// y_offset: u32,
// cell_size: f64,
// transform: Matrix2d,
// g: &mut G)
// where G: Graphics
// {
// let colour = [0.0, 1.0, 0.0, 1.0]; // green
//
// TODO: extract cell iterator?
// for x in view.get_start_x()..view.get_end_x() {
// for y in view.get_start_y()..view.get_end_y() {
// let c = board.get_cell(x, y);
//
// let cell_rectangle = [grid.x_pos((x + x_offset, y + y_offset)),
// grid.y_pos((x + x_offset, y + y_offset)),
// cell_size,
// cell_size];
//
// if c.alive {
// rectangle(colour, cell_rectangle, transform, g);
// }
//
// {
// trace!("On iteration [{}], cell's iteration is [{}] ({}, {})",
// iteration,
// c.get_iteration(),
// x,
// y);
// const AGE_LEVELS: usize = 3;
// const AGE_MAX_DARK: f32 = 0.75;
// const AGE_DARK_INCREMENT: f32 = AGE_MAX_DARK / ((AGE_LEVELS - 1) as f32);
//
// let age = cmp::min(AGE_LEVELS - 1, iteration - c.get_iteration());
// let age_dark = AGE_DARK_INCREMENT * (age as f32);
// trace!("Age of cell is [{}], setting darkness to [{}]. Note, dark increment is \
// [{}]",
// age,
// age_dark,
// AGE_DARK_INCREMENT);
// let age_colour = [0.0, 0.0, 0.0, age_dark];
//
// rectangle(age_colour, cell_rectangle, transform, g);
// }
// }
// }
// }
//

fn draw_cell<G>(cell: Cell,
                x: u32,
                y: u32,
                iteration: usize,
                grid: &Grid,
                cell_size: f64,
                transform: Matrix2d,
                g: &mut G)
    where G: Graphics
{
    let colour = [0.0, 1.0, 0.0, 1.0]; // green

    let cell_rectangle = [grid.x_pos((x, y)), grid.y_pos((x, y)), cell_size, cell_size];

    if cell.alive {
        rectangle(colour, cell_rectangle, transform, g);
    }

    {
        trace!("On iteration [{}], cell's iteration is [{}] ({}, {})",
               iteration,
               cell.get_iteration(),
               x,
               y);
        const AGE_LEVELS: usize = 3;
        const AGE_MAX_DARK: f32 = 0.75;
        const AGE_DARK_INCREMENT: f32 = AGE_MAX_DARK / ((AGE_LEVELS - 1) as f32);

        let age = cmp::min(AGE_LEVELS - 1, iteration - cell.get_iteration());
        let age_dark = AGE_DARK_INCREMENT * (age as f32);
        trace!("Age of cell is [{}], setting darkness to [{}]. Note, dark increment is [{}]",
               age,
               age_dark,
               AGE_DARK_INCREMENT);
        let age_colour = [0.0, 0.0, 0.0, age_dark];

        rectangle(age_colour, cell_rectangle, transform, g);
    }
}


fn main() {
    env_logger::init().unwrap();

    info!("starting up");
    let section_width = 10;
    let section_height = 10;

    let mut columns: Vec<Vec<Box<BoardSection>>> = Vec::new();
    let mut col: Vec<Box<BoardSection>> = Vec::new();

    let mut alives = HashMap::new();
    // Glider
    alives.insert((1, 7), true);
    alives.insert((2, 7), true);
    alives.insert((3, 7), true);
    alives.insert((3, 6), true);
    alives.insert((2, 5), true);

    let board = Board::new(section_width, section_height, &alives);
    col.push(Box::new(LocalBoardSection::new(board)));

    let mut alives = HashMap::new();
    alives.insert((7, 7), true);
    alives.insert((7, 8), true);
    alives.insert((8, 7), true);
    alives.insert((8, 8), true);
    let board = Board::new(section_width, section_height, &alives);
    col.push(Box::new(LocalBoardSection::new(board)));

    columns.push(col);

    let whole = Whole::new(columns);
    // let bottom_callback: Box<Fn(&[&Cell])> = {
    // let sb = sectionBottom.clone();
    //
    // Box::new(move |cells: &[&Cell]| {
    // sb.lock().unwrap().update(BoardSectionSide::Top, cells);
    // })
    // };
    // let bottom_subscribe = CellStateCallback::new((0, 1), bottom_callback);
    // sectionTop.lock().unwrap().subscribe(BoardSectionSide::Bottom, bottom_subscribe);
    //
    // let top_callback: Box<Fn(&[&Cell])> = {
    // let st = sectionTop.clone();
    //
    // Box::new(move |cells: &[&Cell]| {
    // st.lock().unwrap().update(BoardSectionSide::Bottom, cells);
    // })
    // };
    // let top_subscribe = CellStateCallback::new((0, 0), top_callback);
    // sectionBottom.lock().unwrap().subscribe(BoardSectionSide::Top, top_subscribe);
    //

    let total_rows = whole.rows_count() as u32;
    let total_columns = whole.columns_count() as u32;
    debug!("Total rows is [{}], total columns is [{}]",
           total_rows,
           total_columns);

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

    let mut events = window.events().max_fps(2);
    let mut iteration = 0;
    let mut tick = 0;

    let whole = &whole;

    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, g| {
        		debug!("Doing iteration [{}], tick is [{}]", iteration, tick);
            		
                do_life(whole, iteration);

                clear([1.0, 1.0, 1.0, 1.0], g);

            	whole.foreach_cell(&mut |cell, x, y| {
					draw_cell(cell, x, y, iteration, &grid, cell_size, c.transform, g);
                });

				//Draw grid over the top of squares
				grid.draw(&grid_line, &c.draw_state, c.transform, g);

                tick += 1;
                iteration += if tick % 3 == 0 {
                    1
                } else {
                    0
                };
            });
        }
    }
}
