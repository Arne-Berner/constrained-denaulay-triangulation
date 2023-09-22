
#[derive(Debug)]
pub enum CustomError {
    PointOutOfBounds,
    TriangulationFailed,
    CouldntFindExistingTriangle,
    TesselationFailed,
}