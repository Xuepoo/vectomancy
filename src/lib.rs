pub mod config;
pub mod emitter;
pub mod error;
pub mod math;
pub mod models;
pub mod parser;

// Workspace crate re-exports
pub use vectomancy_export as export_crate;
pub use vectomancy_geometry as geometry_crate;
pub use vectomancy_pipeline as pipeline_crate;
pub use vectomancy_raster as raster_crate;
pub use vectomancy_svg as svg_crate;
pub use vectomancy_transform as transform_crate;

/// Prelude module re-exporting common types for quick import.
pub mod prelude {
    pub use vectomancy_export::{encode_json, encode_svg, encode_zlib_base64};
    pub use vectomancy_geometry::{BoundingBox, Point2D, Polyline, PolylineScene, StyledPath};
    pub use vectomancy_pipeline::{ConversionMode, ConvertedScene, Pipeline, PipelineOptions};
    pub use vectomancy_transform::{
        build_splines, solve_tsp_nearest_neighbor, BezierSegment, FourierTerm, SplineEquation,
    };
}
