use crate::error::VectomancyError;
use crate::models::{BezierSegment, ColoredPath, Point2D};
use std::path::Path;
use usvg::{Options, Tree, TreeParsing};

pub fn process_svg(
    path: &Path,
    color: bool,
) -> Result<(Vec<ColoredPath<Vec<BezierSegment>>>, (u32, u32)), VectomancyError> {
    let svg_data = std::fs::read(path).map_err(|e| VectomancyError::InvalidInput(e.to_string()))?;
    let opt = Options::default();
    let tree = Tree::from_data(&svg_data, &opt)
        .map_err(|e| VectomancyError::InvalidInput(e.to_string()))?;

    let width = tree.size.width() as u32;
    let height = tree.size.height() as u32;

    let mut colored_paths = Vec::new();

    fn traverse(
        group: &usvg::Group,
        colored_paths: &mut Vec<ColoredPath<Vec<BezierSegment>>>,
        color: bool,
    ) {
        for child in &group.children {
            match child {
                usvg::Node::Group(g) => {
                    traverse(g, colored_paths, color);
                }
                usvg::Node::Path(p) => {
                    let mut segments = Vec::new();
                    let transform = p.abs_transform;
                    for seg in p.data.segments() {
                        match seg {
                            usvg::tiny_skia_path::PathSegment::MoveTo(pt) => {
                                let mut pt = pt;
                                transform.map_point(&mut pt);
                                segments.push(BezierSegment::MoveTo(Point2D {
                                    x: pt.x as f64,
                                    y: pt.y as f64,
                                }));
                            }
                            usvg::tiny_skia_path::PathSegment::LineTo(pt) => {
                                let mut pt = pt;
                                transform.map_point(&mut pt);
                                segments.push(BezierSegment::LineTo(Point2D {
                                    x: pt.x as f64,
                                    y: pt.y as f64,
                                }));
                            }
                            usvg::tiny_skia_path::PathSegment::QuadTo(pt1, pt2) => {
                                let mut pt1 = pt1;
                                let mut pt2 = pt2;
                                transform.map_point(&mut pt1);
                                transform.map_point(&mut pt2);
                                segments.push(BezierSegment::QuadraticTo(
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
                                segments.push(BezierSegment::CubicTo(
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
                                segments.push(BezierSegment::Close);
                            }
                        }
                    }

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

                    if !segments.is_empty() {
                        colored_paths.push(ColoredPath {
                            color_rgb,
                            data: segments,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    traverse(&tree.root, &mut colored_paths, color);

    Ok((colored_paths, (width, height)))
}
