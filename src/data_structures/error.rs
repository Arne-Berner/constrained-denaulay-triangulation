#[derive(Debug)]
pub enum CustomError {
    PointNotInTriangle,
    TriangulationFailed,
    CouldntFindExistingTriangle,
    TesselationFailed,
}
