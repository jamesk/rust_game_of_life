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
use rust_game_of_life::view::Rectangle;
use rust_game_of_life::view::BoardView;
use std::cmp;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::TrySendError;
use std::sync::Arc;

fn do_life(senders: &HashMap<BoardSectionSide, Vec<SyncSender<Arc<Vec<Cell>>>>>,
           width: u32,
           height: u32,
           whole: &mut Whole,
           iteration: usize) {
    let cell = Cell::new(false, iteration, false);

    let cells = Arc::new(vec![cell; width as usize]);
    for top_senders in senders.get(&BoardSectionSide::Top) {
        for sender in top_senders {
            match sender.try_send(cells.clone()) {
                Ok(_) => {}
                Err(TrySendError::Full(_)) => {}
                Err(TrySendError::Disconnected(_)) => {
                    // TODO: unsubscribe? panic?
                }
            }
        }
    }
    for bottom_senders in senders.get(&BoardSectionSide::Bottom) {
        for sender in bottom_senders {
            match sender.try_send(cells.clone()) {
                Ok(_) => {}
                Err(TrySendError::Full(_)) => {}
                Err(TrySendError::Disconnected(_)) => {
                    // TODO: unsubscribe? panic?
                }
            }
        }
    }


    let cells = Arc::new(vec![cell; height as usize]);
    for left_senders in senders.get(&BoardSectionSide::Left) {
        for sender in left_senders {
            match sender.try_send(cells.clone()) {
                Ok(_) => {}
                Err(TrySendError::Full(_)) => {}
                Err(TrySendError::Disconnected(_)) => {
                    // TODO: unsubscribe? panic?
                }
            }
        }
    }
    for right_senders in senders.get(&BoardSectionSide::Right) {
        for sender in right_senders {
            match sender.try_send(cells.clone()) {
                Ok(_) => {}
                Err(TrySendError::Full(_)) => {}
                Err(TrySendError::Disconnected(_)) => {
                    // TODO: unsubscribe? panic?
                }
            }
        }
    }

    // TODO: move this into a thread pool type thing
    for x in 0..whole.sections_width() {
        for y in 0..whole.sections_height() {
            let section_arc = whole.get_section(x, y);

            section_arc.try_iteration(iteration);
        }
    }
}

fn draw_cell<G>(cell_op: Option<Cell>,
                x: u32,
                y: u32,
                iteration: usize,
                grid: &Grid,
                cell_size: f64,
                transform: Matrix2d,
                g: &mut G)
    where G: Graphics
{
    let cell_rectangle = [grid.x_pos((x, y)), grid.y_pos((x, y)), cell_size, cell_size];

    match cell_op {
        Some(cell) => {
        	let colour = [0.0, 1.0, 0.0, 1.0]; // green
        	
            let alive = if iteration >= cell.get_iteration() {
                cell.alive
            } else if iteration + 1 == cell.get_iteration() {
                cell.get_previous_alive()
            } else {
                panic!("Asked to draw iteration [{}] but cell's iteration is [{}], cell too far \
                        ahead to draw. Cell's co-ords are [{}, {}]",
                       iteration,
                       cell.get_iteration(),
                       x,
                       y);
            };

            if alive {
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

                let cell_age = iteration.checked_sub(cell.get_iteration()).unwrap_or(0);
                let display_age = cmp::min(AGE_LEVELS - 1, cell_age);
                let age_dark = AGE_DARK_INCREMENT * (display_age as f32);
                trace!("Age of cell is [{}], display age is [{}], setting darkness to [{}]. \
                        Note, dark increment is [{}]",
                       cell_age,
                       display_age,
                       age_dark,
                       AGE_DARK_INCREMENT);
                let age_colour = [0.0, 0.0, 0.0, age_dark];

                rectangle(age_colour, cell_rectangle, transform, g);
            }
        }
        None => {
        	let colour = [1.0, 1.0, 1.0, 1.0]; // black
        	rectangle(colour, cell_rectangle, transform, g);
        }
    }


}


fn main() {
    env_logger::init().unwrap();

    info!("starting up");
    let section_width = 10;
    let section_height = 10;
    let whole_size = 6;

    let (sections, edge_senders, registerers) =
        Whole::create_sections(section_width, section_height, whole_size);
    let view_rectangle = Rectangle::new(0,
                                        0,
                                        section_width * (whole_size as u32),
                                        section_height * (whole_size as u32));
    let mut view = BoardView::new(view_rectangle, registerers);
    let mut whole = Whole::new(sections);

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

    let mut events = window.events().max_fps(24);
    let mut iteration = 0;
    let mut tick = 0;

    let whole = &mut whole;

    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, g| {
                debug!("Doing iteration [{}], tick is [{}]", iteration, tick);

                do_life(&edge_senders,
                        section_width,
                        section_height,
                        whole,
                        iteration);

                clear([1.0, 1.0, 1.0, 1.0], g);

                let iteration_to_draw = iteration.checked_sub(1).unwrap_or(0);
                view.foreach_cell(&mut |cell, x, y| {
                    draw_cell(cell,
                              x,
                              y,
                              iteration_to_draw,
                              &grid,
                              cell_size,
                              c.transform,
                              g);
                });

                // Draw grid over the top of squares
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
