pub mod raster;
pub mod vector;

use crate::error::VectomancyError;
use crate::models::ParserOutput;
use std::path::Path;

pub fn parse_file(path: &Path) -> Result<ParserOutput, VectomancyError> {
    let ext_str = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    match ext_str.to_lowercase().as_str() {
        "png" | "jpg" | "jpeg" | "webp" => {
            let paths = raster::process_raster_image(path)?;
            Ok(ParserOutput::Paths(paths))
        }
        "svg" => {
            let segments = vector::process_svg(path)?;
            Ok(ParserOutput::Segments(segments))
        }
        _ => Err(VectomancyError::InvalidInput(format!(
            "Unsupported file extension: {}",
            ext_str
        ))),
    }
}
