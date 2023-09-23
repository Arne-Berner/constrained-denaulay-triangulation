#[derive(Debug)]
pub enum CustomError {
    PointNotInTriangle,
    SwappingFailed,
    TrianglesDontShareIndex,
    TesselationFailed,
    EdgeNotFoundInTriangles(usize, usize),
    PolygonIsOpen,
}
