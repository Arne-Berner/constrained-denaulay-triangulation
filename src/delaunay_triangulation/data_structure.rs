use bevy::prelude::Vec2;

use crate::math_utils;

use super::math_utils::is_point_to_the_right_of_edge;

pub struct TriangleIndexPair {
    current: usize,
    adjacent: usize,
}

#[derive(PartialEq)]
pub enum FoundOrAdded {
    Found,
    Added,
}
pub struct TriangleSet {
    points: Vec<Vec2>,
    triangle_infos: Vec<TriangleInfo>,
}

impl TriangleSet {
    /// Constructor that receives the expected number of triangles to store. It will reserve memory accordingly.
    pub fn new(expected_triangles: usize) -> Self {
        TriangleSet {
            points: Vec::with_capacity(expected_triangles),
            triangle_infos: Vec::with_capacity(expected_triangles * 3),
        }
    }

    pub fn add_point(&mut self, point_to_add: Vec2) -> (usize, FoundOrAdded) {
        for (idx, point) in self.points.iter().enumerate() {
            if *point == point_to_add {
                return (idx, FoundOrAdded::Found);
            }
        }
        self.points.push(point_to_add);
        (self.points.len() - 1, FoundOrAdded::Added)
    }

    pub fn add_triangle(&self, triangle: &Triangle) {
        let (p0, _) = self.add_point(triangle.p(0));
        let (p1, _) = self.add_point(triangle.p(1));
        let (p2, _) = self.add_point(triangle.p(2));
        self.triangle_infos.push(TriangleInfo::new([p0, p1, p2]));
    }

    pub fn add_triangle_info(&self, triangle_info_to_add: TriangleInfo) -> usize {
        self.triangle_infos.push(triangle_info_to_add);
        self.triangle_infos.len() - 1
    }

    pub fn triangle_count(&self) -> usize {
        self.triangle_infos.len()
    }

    pub fn get_triangle(&self, index: usize) -> Triangle {
        let p0 = self.points[self.triangle_infos[index].vertex_indices[0]];
        let p1 = self.points[self.triangle_infos[index].vertex_indices[1]];
        let p2 = self.points[self.triangle_infos[index].vertex_indices[2]];
        Triangle::new(p0, p1, p2)
    }

    pub fn get_triangle_info(&self, index: usize) -> TriangleInfo {
        self.triangle_infos[index]
    }

    pub fn get_point_from_index(&self, triangle_index: usize, vertex_index: usize) -> Vec2 {
        self.points[self.triangle_infos[triangle_index].vertex_indices[vertex_index]]
    }

    pub fn get_adjacent_triangle_index(
        &self,
        triangle_index: usize,
        vertex_index: usize,
    ) -> Option<usize> {
        self.triangle_infos[triangle_index].adjacent_triangle_indices[vertex_index]
    }

    pub fn find_triangle_that_contains_point(
        &self,
        point: Vec2,
        start_triangle: usize,
    ) -> Option<usize> {
        let mut is_triangle_found = false;
        let mut triangle_index = start_triangle;
        let mut checked_triangles = 0;

        while !is_triangle_found && checked_triangles < self.triangle_count() {
            //weird place
            is_triangle_found = true;
            for vertex_index in 0..3 {
                // if it is outside of the triangle
                if is_point_to_the_right_of_edge(
                    self.get_point_from_index(triangle_index, vertex_index),
                    self.get_point_from_index(triangle_index, vertex_index + 1 % 3),
                    point,
                ) {
                    // The point is in the exterior of the triangle (vertices are sorted CCW, the right side is always the exterior from the perspective of the A->B edge)
                    // This "path finding" can not form a circle, because it will only be on the right side for max 2 edges
                    is_triangle_found = false;
                    if let Some(index) =
                        self.get_adjacent_triangle_index(triangle_index, vertex_index)
                    {
                        triangle_index = index;
                        break;
                    }
                }
            }
            checked_triangles += 1;
        }

        if checked_triangles >= self.triangle_count() && self.triangle_count() > 1 {
            println!("Unable to find a triangle that contains the point ({:?}), starting at triangle {}. Are you generating very small triangles?", point, start_triangle);
            return None;
        }

        Some(triangle_index)
    }

    pub fn replace_adjacent(
        &mut self,
        triangle_index: usize,
        old_adjacent_triangle: Option<usize>,
        new_adjacent_triangle: Option<usize>,
    ) {
        for vertex_index in 0..3 {
            if self.get_adjacent_triangle_index(triangle_index, vertex_index)
                == old_adjacent_triangle
            {
                self.triangle_infos[triangle_index].adjacent_triangle_indices[vertex_index] =
                    new_adjacent_triangle;
            }
        }
    }

    pub fn replace_triangle(&mut self, triangle_index: usize, new_triangle: &TriangleInfo) {
        for i in 0..3 {
            self.triangle_infos[triangle_index].vertex_indices[i] = new_triangle.vertex_indices[i];
            self.triangle_infos[triangle_index].adjacent_triangle_indices[i] =
                new_triangle.adjacent_triangle_indices[i];
        }
    }

    pub fn replace_adjacent_vertices(
        &mut self,
        triangle_index: usize,
        new_adjacent_indices: [Option<usize>;3],
    ) {
        self.triangle_infos[triangle_index].adjacent_triangle_indices = new_adjacent_indices;
    }

    pub fn replace_vertex_with_vertex(
        &mut self,
        vertex_position: usize,
        triangle_index: usize,
        new_vertex: usize,
    ) {
        self.triangle_infos[triangle_index].vertex_indices[vertex_position] = new_vertex;
    }

    // pub fn get_adjacent_triangles_except(&self, exception_index: usize, triangle_index: usize)-> Vec<Option<usize>> {
    //     let mut result = Vec::new();
    //     for i in 0..3 {
    //         if let Some(adjacent_index) =
    //             self.triangle_infos[triangle_index].adjacent_triangle_indices[i]
    //         {
    //             if !(adjacent_index == exception_index) {
    //                 result.push(Some(adjacent_index));
    //             }
    //         } else {
    //             result.push(None)
    //         }
    //     }
    //     result
    // }
}

pub struct Triangle {
    vertices: [Vec2; 3],
}

impl Triangle {
    pub fn new(vertex0: Vec2, vertex1: Vec2, vertex2: Vec2) -> Self {
        Triangle {
            vertices: [vertex0, vertex1, vertex2],
        }
    }

    pub fn p(&self, index: usize) -> Vec2 {
        self.vertices[index]
    }
}
// rename to something that makes more sense
pub struct TriangleInfo {
    pub vertex_indices: [usize; 3],
    pub adjacent_triangle_indices: [Option<usize>; 3],
}
impl TriangleInfo {
    pub fn new(index_vertices: [usize; 3]) -> Self {
        TriangleInfo {
            vertex_indices: index_vertices,
            adjacent_triangle_indices: [None, None, None],
        }
    }

    pub fn with_adjacent(
        mut self,
        adjacent0: Option<usize>,
        adjacent1: Option<usize>,
        adjacent2: Option<usize>,
    ) -> TriangleInfo {
        self.adjacent_triangle_indices[0] = adjacent0;
        self.adjacent_triangle_indices[1] = adjacent1;
        self.adjacent_triangle_indices[2] = adjacent2;
        self
    }
}

// struct Edge {
//     vertex_a: Vec2,
//     vertex_b: Vec2,
// }

// impl Edge {
//     pub fn new(vertex_a: Vec2, vertex_b: Vec2) -> Self {
//         Edge { vertex_a, vertex_b }
//     }
// }
