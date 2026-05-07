pub mod raster;
pub mod vector;

use crate::error::VectomancyError;
use crate::models::ParserOutput;
use std::path::Path;

pub fn parse_file(path: &Path) -> Result<ParserOutput, VectomancyError> {
    let ext_str = match path.extension().and_then(|s| s.to_str()) {
        Some(s) => s,
        None => "",
    };
    match ext_str.to_lowercase().as_str() {
        "png" | "jpg" | "jpeg" | "webp" => {
            let points = raster::process_raster_image(path)?;
            Ok(ParserOutput::Points(points))
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
