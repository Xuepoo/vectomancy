pub mod algorithms;
pub mod types;

pub use algorithms::chaikin::chaikin_smooth;
pub use algorithms::rdp::simplify_rdp;
pub use algorithms::resampling::resample_by_arc_length;
pub use types::{BoundingBox, Point2D, Polyline, PolylineScene, Scene, StyledPath};
