use std::path::Path;
use tracing::info;
use vectomancy_geometry::{chaikin_smooth, simplify_rdp, Polyline, PolylineScene};
use vectomancy_raster::decode_raster_memory;
use vectomancy_svg::decode_svg_memory;
use vectomancy_transform::{
    build_splines, perform_fft, BezierSegment, FourierTerm, SplineEquation,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConversionMode {
    Polyline,
    Chaikin { iterations: usize },
    Spline,
    Fourier { terms: usize },
}

#[derive(Debug, Clone)]
pub enum ConvertedScene {
    Polyline(PolylineScene),
    Spline {
        equations: Vec<SplineEquation>,
        dimensions: (u32, u32),
    },
    Fourier {
        terms: Vec<Vec<FourierTerm>>,
        dimensions: (u32, u32),
    },
}

pub struct PipelineOptions {
    pub mode: ConversionMode,
    pub rdp_epsilon: f64,
    pub color: bool,
}

pub struct Pipeline {
    options: PipelineOptions,
}

impl Pipeline {
    pub fn new(options: PipelineOptions) -> Self {
        Self { options }
    }

    pub fn convert_raster_bytes(&self, bytes: &[u8]) -> Result<ConvertedScene, String> {
        info!("Converting raster image bytes via Pipeline");
        let (raw_paths, dimensions) = decode_raster_memory(bytes, self.options.color)?;

        let mut processed_paths = Vec::new();
        for path in raw_paths {
            let simplified = simplify_rdp(&path.geometry.points, self.options.rdp_epsilon);
            let polyline = Polyline::new(simplified, path.geometry.closed);

            let final_poly = match self.options.mode {
                ConversionMode::Chaikin { iterations } => chaikin_smooth(&polyline, iterations),
                _ => polyline,
            };

            processed_paths.push(vectomancy_geometry::StyledPath::new(
                final_poly,
                path.color_style,
            ));
        }

        let bounds = vectomancy_geometry::BoundingBox::from_points(
            &processed_paths
                .iter()
                .flat_map(|p| p.geometry.points.clone())
                .collect::<Vec<_>>(),
        );

        let scene = PolylineScene {
            paths: processed_paths,
            dimensions,
            bounds,
        };

        match self.options.mode {
            ConversionMode::Polyline | ConversionMode::Chaikin { .. } => {
                Ok(ConvertedScene::Polyline(scene))
            }
            ConversionMode::Spline => {
                let mut splines = Vec::new();
                for p in &scene.paths {
                    let segs: Vec<BezierSegment> = p
                        .geometry
                        .points
                        .iter()
                        .enumerate()
                        .map(|(i, pt)| {
                            if i == 0 {
                                BezierSegment::MoveTo(*pt)
                            } else {
                                BezierSegment::LineTo(*pt)
                            }
                        })
                        .collect();
                    splines.extend(build_splines(&segs, true));
                }
                Ok(ConvertedScene::Spline {
                    equations: splines,
                    dimensions,
                })
            }
            ConversionMode::Fourier { terms } => {
                let mut all_terms = Vec::new();
                for p in &scene.paths {
                    let fft_res = perform_fft(&p.geometry.points, terms, false, false, 0.99)?;
                    all_terms.push(fft_res);
                }
                Ok(ConvertedScene::Fourier {
                    terms: all_terms,
                    dimensions,
                })
            }
        }
    }

    pub fn convert_svg_bytes(&self, bytes: &[u8]) -> Result<ConvertedScene, String> {
        info!("Converting SVG bytes via Pipeline");
        let (raw_paths, dimensions) = decode_svg_memory(bytes, self.options.color)?;

        let mut splines = Vec::new();
        for path in raw_paths {
            splines.extend(build_splines(&path.geometry, true));
        }

        Ok(ConvertedScene::Spline {
            equations: splines,
            dimensions,
        })
    }

    pub fn convert_raster_file(&self, path: &Path) -> Result<ConvertedScene, String> {
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        self.convert_raster_bytes(&bytes)
    }

    pub fn convert_svg_file(&self, path: &Path) -> Result<ConvertedScene, String> {
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        self.convert_svg_bytes(&bytes)
    }
}
