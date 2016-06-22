use std::collections::HashMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    pub alive: bool, // TODO: write getter
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

    pub fn get_previous_alive(&self) -> bool {
        self.previous_alive
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
                                    } else if c.iteration > 0 &&
                                                   c.iteration - 1 == iteration {
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

    pub fn next_cell(&self, x: u32, y: u32, current: &Cell) -> Option<Cell> {
        let alive_count_option = self.neighbour_alive_count(x, y, current.iteration);

        let should_be_alive =
            alive_count_option.map(|alive_count|
            	Board::should_be_alive(current.alive, alive_count)
            );

        should_be_alive.map(|alive| {
            Cell {
                alive: alive,
                iteration: current.iteration + 1,
                previous_alive: current.alive,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::*;

    fn get_test_board() -> Board {
        let mut alives = HashMap::new();
        alives.insert((1, 1), true);

        Board::new(4, 4, &alives)
    }

    #[test]
    fn board_new_sets_alive_cells() {
        let board = get_test_board();

        assert!(board.get_cell(1, 1).alive);
        assert!(!board.get_cell(0, 0).alive);
    }

    #[test]
    fn board_set_cell_updates_cell() {
        let mut board = get_test_board();
        let cell = Cell::new(true, 10, false);

        board.set_cell(2, 2, cell);

        let actual = board.get_cell(2, 2);
        assert_eq!(actual, &cell);
    }

    #[test]
    fn board_neighbour_alive_count_correct_0th_iteration_0_alive() {
        let board = get_test_board();

        assert_eq!(board.neighbour_alive_count(1, 1, 0), Some(0));
    }

    #[test]
    fn board_neighbour_alive_count_correct_0th_iteration_1_alive() {
        let mut board = get_test_board();

        let cell = Cell::new(true, 0, false);
        board.set_cell(2, 2, cell);

        assert_eq!(board.neighbour_alive_count(1, 1, 0), Some(1));
    }

    #[test]
    fn board_neighbour_alive_count_correct_0th_iteration_3_alive() {
        let mut board = get_test_board();

        let cell = Cell::new(true, 0, false);
        board.set_cell(2, 2, cell);
        board.set_cell(1, 2, cell);
        board.set_cell(2, 1, cell);

        assert_eq!(board.neighbour_alive_count(1, 1, 0), Some(3));
    }

    #[test]
    fn board_neighbour_alive_count_correct_0th_iteration_3_alive_all_ahead() {
        let mut board = get_test_board();

        let cell = Cell::new(false, 1, true);
        board.set_cell(2, 2, cell);
        board.set_cell(1, 2, cell);
        board.set_cell(2, 1, cell);

        assert_eq!(board.neighbour_alive_count(1, 1, 0), Some(3));
    }

    #[test]
    fn board_neighbour_alive_count_correct_0th_iteration_1_alive_others_ahead() {
        let mut board = get_test_board();

        let cell = Cell::new(true, 1, false);
        board.set_cell(2, 2, cell);
        board.set_cell(1, 2, cell);
        let cell = Cell::new(true, 0, false);
        board.set_cell(2, 1, cell);

        assert_eq!(board.neighbour_alive_count(1, 1, 0), Some(1));
    }

    #[test]
    fn board_neighbour_alive_count_correct_0th_iteration_neighbour_behind() {
        let mut board = get_test_board();

        let cell = Cell::new(true, 0, false);
        board.set_cell(2, 2, cell);

        assert_eq!(board.neighbour_alive_count(1, 1, 1), None);
    }

    #[test]
    fn board_next_cell_waiting() {
        let mut board = get_test_board();
        let cell = Cell::new(true, 10, true);
        board.set_cell(1, 1, cell);

        assert_eq!(board.next_cell(1, 1, board.get_cell(1, 1)), None);
    }

    #[test]
    fn board_next_cell_die() {
        let board = get_test_board();

        let actual = board.next_cell(1, 1, board.get_cell(1, 1))
            .expect("No next cell found when all cells on iteration 0");

        assert_eq!(actual.alive, false);
    }

    #[test]
    fn board_next_cell_live() {
        let mut board = get_test_board();
        let cell = Cell::new(true, 0, false);
        board.set_cell(0, 0, cell);
        board.set_cell(0, 1, cell);

        let actual = board.next_cell(1, 1, board.get_cell(1, 1))
            .expect("No next cell found when all cells on iteration 0");

        assert_eq!(actual.alive, true);
    }
}
