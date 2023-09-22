use super::vec2::Vec2;

#[derive(Debug)]
pub struct PointBinGrid {
    cells: Vec<Vec<Vec2>>,
    cell_size: Vec2,
    grid_size: Vec2,
    cells_per_side: usize,
}

impl PointBinGrid {
    pub fn new(cells_per_side: usize) -> Self {
        let grid_size = Vec2::new(1., 1.);
        let cells = vec![vec![]; (cells_per_side * cells_per_side) as usize];
        let cell_size = grid_size / cells_per_side as f32;

        PointBinGrid {
            cells,
            cell_size,
            grid_size,
            cells_per_side,
        }
    }
    pub fn add_point(&mut self, new_point: Vec2) {
        // grid size should be one
        let row_index =
            (0.99 * self.cells_per_side as f32 * new_point.y / self.grid_size.y) as usize;
        let column_index =
            (0.99 * self.cells_per_side as f32 * new_point.x / self.grid_size.x) as usize;

        // it will be filled like so:
        // 6 7 8 ->
        // 5 4 3 <-
        // 0 1 2 ->
        let bin_index = if row_index % 2 == 0 {
            (row_index * self.cells_per_side + column_index) as usize
        } else {
            ((row_index + 1) * self.cells_per_side - column_index - 1) as usize
        };

        self.cells[bin_index as usize].push(new_point);
    }

    pub fn cells(&self) -> &Vec<Vec<Vec2>> {
        &self.cells
    }
}
