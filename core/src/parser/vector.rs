use crate::error::VectomancyError;
use crate::models::{BezierSegment, ColoredPath, Point2D};
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
use std::path::Path;
use usvg::{Options, Tree, TreeParsing};

#[allow(clippy::type_complexity)]
pub fn process_svg(
    path: &Path,
    color: bool,
) -> Result<(Vec<ColoredPath<Vec<BezierSegment>>>, (u32, u32)), VectomancyError> {
    let svg_data = std::fs::read(path).map_err(|e| VectomancyError::InvalidInput(e.to_string()))?;
    process_svg_from_memory(&svg_data, color)
}

#[allow(clippy::type_complexity)]
pub fn process_svg_from_memory(
    svg_data: &[u8],
    color: bool,
) -> Result<(Vec<ColoredPath<Vec<BezierSegment>>>, (u32, u32)), VectomancyError> {
    let opt = Options::default();
    let tree = Tree::from_data(svg_data, &opt)
        .map_err(|e| VectomancyError::InvalidInput(e.to_string()))?;

    let width = tree.size.width() as u32;
    let height = tree.size.height() as u32;

    let mut all_segments = Vec::new();

    fn traverse(
        group: &usvg::Group,
        all_segments: &mut Vec<(
            Option<(u8, u8, u8)>,
            Vec<usvg::tiny_skia_path::PathSegment>,
            usvg::Transform,
        )>,
        color: bool,
    ) {
        for child in &group.children {
            match child {
                usvg::Node::Group(g) => {
                    traverse(g, all_segments, color);
                }
                usvg::Node::Path(p) => {
                    let mut color_rgb = None;
                    if color {
                        if let Some(stroke) = &p.stroke {
                            if let usvg::Paint::Color(c) = stroke.paint {
                                color_rgb = Some((c.red, c.green, c.blue));
                            }
                        } else if let Some(fill) = &p.fill {
                            if let usvg::Paint::Color(c) = fill.paint {
                                color_rgb = Some((c.red, c.green, c.blue));
                            }
                        }
                    }

                    let segments: Vec<_> = p.data.segments().collect();
                    if !segments.is_empty() {
                        all_segments.push((color_rgb, segments, p.abs_transform));
                    }
                }
                _ => {}
            }
        }
    }

    traverse(&tree.root, &mut all_segments, color);

    #[cfg(not(target_arch = "wasm32"))]
    let segment_iter = all_segments.into_par_iter();
    #[cfg(target_arch = "wasm32")]
    let segment_iter = all_segments.into_iter();

    let colored_paths: Vec<_> = segment_iter
        .map(|(color_rgb, segments, transform)| {
            let mut bezier_segments = Vec::new();
            for seg in segments {
                match seg {
                    usvg::tiny_skia_path::PathSegment::MoveTo(pt) => {
                        let mut pt = pt;
                        transform.map_point(&mut pt);
                        bezier_segments.push(BezierSegment::MoveTo(Point2D {
                            x: pt.x as f64,
                            y: pt.y as f64,
                        }));
                    }
                    usvg::tiny_skia_path::PathSegment::LineTo(pt) => {
                        let mut pt = pt;
                        transform.map_point(&mut pt);
                        bezier_segments.push(BezierSegment::LineTo(Point2D {
                            x: pt.x as f64,
                            y: pt.y as f64,
                        }));
                    }
                    usvg::tiny_skia_path::PathSegment::QuadTo(pt1, pt2) => {
                        let mut pt1 = pt1;
                        let mut pt2 = pt2;
                        transform.map_point(&mut pt1);
                        transform.map_point(&mut pt2);
                        bezier_segments.push(BezierSegment::QuadraticTo(
                            Point2D {
                                x: pt1.x as f64,
                                y: pt1.y as f64,
                            },
                            Point2D {
                                x: pt2.x as f64,
                                y: pt2.y as f64,
                            },
                        ));
                    }
                    usvg::tiny_skia_path::PathSegment::CubicTo(pt1, pt2, pt3) => {
                        let mut pt1 = pt1;
                        let mut pt2 = pt2;
                        let mut pt3 = pt3;
                        transform.map_point(&mut pt1);
                        transform.map_point(&mut pt2);
                        transform.map_point(&mut pt3);
                        bezier_segments.push(BezierSegment::CubicTo(
                            Point2D {
                                x: pt1.x as f64,
                                y: pt1.y as f64,
                            },
                            Point2D {
                                x: pt2.x as f64,
                                y: pt2.y as f64,
                            },
                            Point2D {
                                x: pt3.x as f64,
                                y: pt3.y as f64,
                            },
                        ));
                    }
                    usvg::tiny_skia_path::PathSegment::Close => {
                        bezier_segments.push(BezierSegment::Close);
                    }
                }
            }

            ColoredPath {
                color_rgb,
                data: bezier_segments,
            }
        })
        .collect();

    Ok((colored_paths, (width, height)))
}
