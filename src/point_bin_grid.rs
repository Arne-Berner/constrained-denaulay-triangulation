use bevy::prelude::{Color, Gizmos, Vec2};

#[derive(Debug)]
pub struct PointBinGrid {
    pub cells: Vec<Vec<Vec2>>,
    cell_size: Vec2,
    grid_size: Vec2,
    cells_per_side: usize,
}

impl PointBinGrid {
    pub fn new(cells_per_side: usize, grid_size: Vec2) -> PointBinGrid {
        let cells = vec![vec![]; (cells_per_side * cells_per_side) as usize];
        let cell_size = grid_size / cells_per_side as f32;
        let grid_size = grid_size;
        let cells_per_side = cells_per_side;

        PointBinGrid {
            cells,
            cell_size,
            grid_size,
            cells_per_side,
        }
    }
    pub fn add_point(&mut self, new_point: Vec2) {
        let row_index = (0.99 * self.cells_per_side as f32 * new_point.y / self.grid_size.y) as usize;
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

        // this should not be able to happen?
        if self.cells.get(bin_index).is_none() {
            self.cells[bin_index] = vec![];
        }

        self.cells[bin_index as usize].push(new_point);

        // draw_point_addition(new_point, column_index, row_index);
    }

    #[allow(dead_code)]
    fn draw_point_addition(
        &self,
        point: Vec2,
        column_index: usize,
        row_index: usize,
        mut gizmos: Gizmos,
    ) {
        let cell_bottom_left_corner = Vec2::new(
            column_index as f32 * self.cell_size.x,
            row_index as f32 * self.cell_size.y,
        );
        gizmos.line_2d(
            point,
            cell_bottom_left_corner + self.cell_size * 0.5,
            Color::CYAN,
        );
    }
}
