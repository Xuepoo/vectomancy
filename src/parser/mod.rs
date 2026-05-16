pub mod raster;
pub mod vector;

use crate::error::VectomancyError;
use crate::models::ParserOutput;
use std::path::Path;

pub fn parse_file(path: &Path, color: bool) -> Result<ParserOutput, VectomancyError> {
    let ext_str = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    match ext_str.to_lowercase().as_str() {
        "png" | "jpg" | "jpeg" | "webp" => {
            let (paths, original_dimensions) = raster::process_raster_image(path, color)?;
            Ok(ParserOutput::Paths {
                paths,
                original_dimensions,
            })
        }
        "svg" => {
            let (segments, original_dimensions) = vector::process_svg(path, color)?;
            Ok(ParserOutput::Segments {
                segments,
                original_dimensions,
            })
        }
        _ => Err(VectomancyError::InvalidInput(format!(
            "Unsupported file extension: {}",
            ext_str
        ))),
    }
}

pub fn parse_memory(
    bytes: &[u8],
    format: &str,
    color: bool,
) -> Result<ParserOutput, VectomancyError> {
    match format.to_lowercase().as_str() {
        "png" | "jpg" | "jpeg" | "webp" => {
            let (paths, original_dimensions) = raster::process_raster_from_memory(bytes, color)?;
            Ok(ParserOutput::Paths {
                paths,
                original_dimensions,
            })
        }
        "svg" => {
            let (segments, original_dimensions) = vector::process_svg_from_memory(bytes, color)?;
            Ok(ParserOutput::Segments {
                segments,
                original_dimensions,
            })
        }
        _ => Err(VectomancyError::InvalidInput(format!(
            "Unsupported file format: {}",
            format
        ))),
    }
}
