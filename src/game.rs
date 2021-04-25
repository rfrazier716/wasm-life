use std::fmt::Error;
use std::fmt;

use wasm_bindgen::prelude::*;

#[derive(Copy,Clone, PartialEq, Debug)]
pub enum Cell {
    Dead = 0,
    Alive = 1
}

#[wasm_bindgen]
pub struct World {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

// these are functions that will be exposed to WASM bindgen
#[wasm_bindgen]
impl World{
    pub fn wasm_create() -> Self {
        return World::default()
    }

    pub fn wasm_tick(&mut self){
        self.tick().unwrap()
    }

    pub fn wasm_render(&self) -> String{
        self.to_string()
    }

    pub fn get_width(&self) -> u32 {self.width}
    pub fn get_height(&self) -> u32 {self.height}
}


// These functions stay internal to the wasm compilation -- JS cannot access
impl World{
    pub fn tick(&mut self) -> Result<(), String> {
        let mut new_world = self.cells.clone();
        for row in 0..self.height {
            for col in 0..self.width {
                let index = self.index(row, col)?;
                let neighbors = self.count_neighbors(row, col)?;
                let cell = self.cells[index];
                let next_cell = match (cell, neighbors) {
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    (Cell::Alive, x) if x >= 2 && x <= 3 => Cell::Alive,
                    (Cell::Dead, x) if x == 3 => Cell::Alive,
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    (cell, _) => cell,
                };
                new_world[index] = next_cell;
            }
        }
        self.cells = new_world;
        Ok(())
    }

    fn get_cell(&self, row: u32, column:u32) -> Result<&Cell, String>{
        let index = self.index(row, column)?;
        Ok(&self.cells[index])
    }

    fn index(&self, row: u32, column: u32) -> Result<usize, String>{
        match (row, column){
            (r,c) if r < self.height && c < self.width => Ok((r* self.width + c) as usize),
            (r,c) => {
                let error_message = format!("Index out of Bounds, tried accessing ({},{}) for a universe with size ({},{})",
                                            r, c, self.height, self.width);
                Err(error_message)
            }
            }

        }

    fn count_neighbors(&self, row:u32, column:u32) -> Result<u8, String> {
        let mut living_neighbors = 0;
        for r in [self.height - 1, 0, 1].iter().cloned(){
            for c in [self.width - 1, 0, 1].iter().cloned(){
                if r == 0 && c == 0 {continue;} // skip if we're on the target cell
                let neighbor_row = (row + r) % self.height;
                let neighbor_column = (column + c) % self.width;
                living_neighbors+= self.cells[self.index(neighbor_row,neighbor_column)?] as u8;
            }
        }
        Ok(living_neighbors)
    }

    pub fn new(width:u32, height: u32) -> Self {
        let cells = (0..width * height).map(|_x| {Cell::Dead}).collect();
        World {
            width,
            height,
            cells
        }
    }
}

impl Default for World {
    fn default() -> Self {
        let width = 64;
        let height = 64;
        let cells = (0..width * height)
            .map(|i| {
                if i % 2 == 0 || i % 7 == 0 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        World {
            width,
            height,
            cells
        }
    }
}

impl fmt::Display for World {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn test_default_constructor() {
        let world = World::default();
        assert_eq!(world.get_width(), 64);
        assert_eq!(world.get_height(), 64);
    }

    #[test]
    fn test_nondefault_constructor() {
        let world = World::new(128,16);
        assert_eq!(world.get_width(), 128);
        assert_eq!(world.get_height(), 16);
    }

    #[test]
    fn test_indexing(){
        let world = World::default();
        assert_eq!(world.index(0,0).unwrap(), 0);
        assert_eq!(world.index(0,63).unwrap(), 63);
        assert_eq!(world.index(2,0).unwrap(), 128);
    }

    #[test]
    #[should_panic]
    fn test_invalid_indexing()    {
        let world = World::default();
        world.index(0,1000).unwrap();
    }

    #[test]
    fn test_counting_neighbors(){
        let mut world = World::new(3,3); // make an itty bitty world

        // add a single neighbor, check that we get one
        world.cells[0] = Cell::Alive;
        assert_eq!(world.count_neighbors(1,1).unwrap(), 1);

        // add another neighbor, should now have 2!
        let mut update_index = world.index(2,0).unwrap();
        world.cells[update_index] = Cell::Alive; // add another
        assert_eq!(world.count_neighbors(1,1).unwrap(), 2);

        //if the cell itself is alive that shouldn't effect the count
        update_index = world.index(1,1).unwrap();
        world.cells[update_index] = Cell::Alive;
        assert_eq!(world.count_neighbors(1,1).unwrap(), 2);
    }

    #[test]
    fn test_extinction_starvation() -> Result<(),String>{
        let mut world = World::new(3,3); // make an itty bitty world
        world.cells[4] = Cell::Alive;
        world.tick()?;
        for cell in world.cells{
            assert_eq!(cell, Cell::Dead);
        }
        Ok(())
    }

    #[test]
    fn test_extinction_overpopulation() -> Result<(),String>{
        let mut world = World::new(3,3); // make an itty bitty world
        world.cells[0] = Cell::Alive;
        world.cells[2] = Cell::Alive;
        world.cells[6] = Cell::Alive;
        world.cells[8] = Cell::Alive;
        world.cells[4] = Cell::Alive;

        world.tick()?;
        assert_eq!(world.cells[4], Cell::Dead);
        Ok(())
    }

    #[test]
    fn test_continued_life() -> Result<(),String>{
        let mut world = World::new(3,3); // make an itty bitty world
        for idx in 0..3 {
            world.cells[idx] = Cell::Alive;
        }
        world.cells[4] = Cell::Alive;

        world.tick()?;
        assert_eq!(world.cells[4], Cell::Alive);
        Ok(())
    }

    #[test]
    fn test_creating_life() -> Result<(),String>{
        let mut world = World::new(3,3); // make an itty bitty world
        for idx in 0..3 {
            world.cells[idx] = Cell::Alive;
        }

        world.tick()?;
        assert_eq!(world.cells[4], Cell::Alive);
        Ok(())
    }
}