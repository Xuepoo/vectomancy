use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use vectomancy::models::{BezierSegment, Point2D};

#[derive(Serialize)]
struct Command {
    #[serde(rename = "type")]
    cmd_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    y: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    x1: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    y1: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    x2: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    y2: Option<f64>,
}

#[derive(Serialize)]
struct PathAST {
    color: String,
    commands: Vec<Command>,
}

#[derive(Serialize)]
struct GlyphAST {
    #[serde(rename = "type")]
    ast_type: String,
    paths: Vec<PathAST>,
}

#[derive(Serialize)]
struct GlyphInfo {
    width: f64,
    #[serde(rename = "baseSize")]
    base_size: f64,
    ast: GlyphAST,
}

fn main() {
    // We use the woff2 font from the test fixtures
    let font_bytes = include_bytes!("../../../test-fixtures/fonts/pressstart2p.woff2");
    let size = 64.0;

    let mut atlas = HashMap::new();

    // Printable ASCII characters (A-Z, a-z, 0-9, punctuation)
    for code in 32..=126 {
        let c = code as u8 as char;

        let (paths, dim) =
            vectomancy_text::parser::extract_text_outlines(&c.to_string(), font_bytes, size, 0.0)
                .unwrap();

        let mut ast_paths = Vec::new();
        for p in paths {
            let mut commands = Vec::new();
            let mut current_pt = Point2D { x: 0.0, y: 0.0 };

            for seg in p.data {
                match seg {
                    BezierSegment::MoveTo(pt) => {
                        current_pt = pt;
                        commands.push(Command {
                            cmd_type: "M".to_string(),
                            x: Some(pt.x),
                            y: Some(pt.y),
                            x1: None,
                            y1: None,
                            x2: None,
                            y2: None,
                        });
                    }
                    BezierSegment::LineTo(pt) => {
                        current_pt = pt;
                        commands.push(Command {
                            cmd_type: "L".to_string(),
                            x: Some(pt.x),
                            y: Some(pt.y),
                            x1: None,
                            y1: None,
                            x2: None,
                            y2: None,
                        });
                    }
                    BezierSegment::QuadraticTo(pt1, pt2) => {
                        // Convert Quadratic Bezier to Cubic Bezier for Web Canvas compatibility
                        let cp1x = current_pt.x + (2.0 / 3.0) * (pt1.x - current_pt.x);
                        let cp1y = current_pt.y + (2.0 / 3.0) * (pt1.y - current_pt.y);

                        let cp2x = pt2.x + (2.0 / 3.0) * (pt1.x - pt2.x);
                        let cp2y = pt2.y + (2.0 / 3.0) * (pt1.y - pt2.y);

                        current_pt = pt2;
                        commands.push(Command {
                            cmd_type: "C".to_string(),
                            x: Some(pt2.x),
                            y: Some(pt2.y),
                            x1: Some(cp1x),
                            y1: Some(cp1y),
                            x2: Some(cp2x),
                            y2: Some(cp2y),
                        });
                    }
                    BezierSegment::CubicTo(pt1, pt2, pt3) => {
                        current_pt = pt3;
                        commands.push(Command {
                            cmd_type: "C".to_string(),
                            x: Some(pt3.x),
                            y: Some(pt3.y),
                            x1: Some(pt1.x),
                            y1: Some(pt1.y),
                            x2: Some(pt2.x),
                            y2: Some(pt2.y),
                        });
                    }
                    BezierSegment::Close => {
                        commands.push(Command {
                            cmd_type: "Z".to_string(),
                            x: None,
                            y: None,
                            x1: None,
                            y1: None,
                            x2: None,
                            y2: None,
                        });
                    }
                }
            }
            ast_paths.push(PathAST {
                color: "#fff".to_string(),
                commands,
            });
        }

        atlas.insert(
            c.to_string(),
            GlyphInfo {
                width: dim.0 as f64,
                base_size: size as f64,
                ast: GlyphAST {
                    ast_type: "Path".to_string(),
                    paths: ast_paths,
                },
            },
        );
    }

    let json = serde_json::to_string(&atlas).unwrap();
    let mut file = File::create(
        "/mnt/data/Workspace/Projects/xuepoo/xuepoo-www/static/ast/font_glyph_map.json",
    )
    .unwrap();
    file.write_all(json.as_bytes()).unwrap();
    println!("Successfully Generated Vector Atlas: /mnt/data/Workspace/Projects/xuepoo/xuepoo-www/static/ast/font_glyph_map.json!");
}
