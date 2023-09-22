#[derive(Debug)]
pub enum CustomError {
    PointNotInTriangle,
    SwappingFailed,
    TrianglesDontShareIndex,
    TesselationFailed,
}
