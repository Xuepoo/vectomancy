use crate::error::VectomancyError;
use crate::models::{MathExpressionAST, Point2D};
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn emit_scratch(
    ast: &MathExpressionAST,
    output_path: &Path,
    dimensions: (u32, u32),
) -> Result<(), VectomancyError> {
    // Collect all paths to draw
    let mut all_paths: Vec<Vec<Point2D>> = Vec::new();
    match ast {
        MathExpressionAST::Polyline { paths } => {
            for p in paths {
                all_paths.push(p.data.clone());
            }
        }
        MathExpressionAST::Spline { equations } => {
            for p in equations {
                // sample spline
                let mut pts = Vec::new();
                for eq in &p.data {
                    for i in 0..=10 {
                        let t = i as f64 / 10.0;
                        let mut x = 0.0;
                        for (j, c) in eq.x_poly.iter().enumerate() {
                            x += c * t.powi(j as i32);
                        }
                        let mut y = 0.0;
                        for (j, c) in eq.y_poly.iter().enumerate() {
                            y += c * t.powi(j as i32);
                        }
                        pts.push(Point2D { x, y });
                    }
                }
                all_paths.push(pts);
            }
        }
        MathExpressionAST::Fourier { strokes } => {
            for p in strokes {
                let mut pts = Vec::new();
                for i in 0..=100 {
                    let t = (i as f64 / 100.0) * std::f64::consts::TAU;
                    let mut x = 0.0;
                    let mut y = 0.0;
                    for term in &p.data {
                        x += term.amplitude * (term.frequency * t + term.phase).cos();
                        y += term.amplitude * (term.frequency * t + term.phase).sin();
                    }
                    pts.push(Point2D { x, y });
                }
                all_paths.push(pts);
            }
        }
    }

    let mut blocks = serde_json::Map::new();
    let mut current_block_id = "start".to_string();

    blocks.insert(
        current_block_id.clone(),
        json!({
            "opcode": "event_whenflagclicked",
            "next": "clear",
            "parent": null,
            "inputs": {},
            "fields": {},
            "shadow": false,
            "topLevel": true,
            "x": 100,
            "y": 100
        }),
    );

    blocks.insert(
        "clear".to_string(),
        json!({
            "opcode": "pen_clear",
            "next": "pu_start",
            "parent": "start",
            "inputs": {},
            "fields": {},
            "shadow": false,
            "topLevel": false
        }),
    );

    blocks.insert(
        "pu_start".to_string(),
        json!({
            "opcode": "pen_penUp",
            "next": if all_paths.is_empty() { None } else { Some("path_0_pt_0_move".to_string()) },
            "parent": "clear",
            "inputs": {},
            "fields": {},
            "shadow": false,
            "topLevel": false
        }),
    );

    let mut prev_id = "pu_start".to_string();

    // Scratch stage is 480x360. Center is 0,0.
    // X goes from -240 to 240. Y goes from -180 to 180.
    let scale = (480.0 / dimensions.0 as f64).min(360.0 / dimensions.1 as f64);
    let off_x = -(dimensions.0 as f64 * scale) / 2.0;
    let off_y = (dimensions.1 as f64 * scale) / 2.0; // Y is inverted in Scratch vs standard image coords

    for (p_idx, path) in all_paths.iter().enumerate() {
        if path.is_empty() {
            continue;
        }
        for (pt_idx, pt) in path.iter().enumerate() {
            let sx = pt.x * scale + off_x;
            let sy = off_y - pt.y * scale;

            let move_id = format!("path_{}_pt_{}_move", p_idx, pt_idx);
            let next_id = if pt_idx == 0 {
                format!("path_{}_pd", p_idx)
            } else if pt_idx == path.len() - 1 {
                format!("path_{}_pu", p_idx)
            } else {
                format!("path_{}_pt_{}_move", p_idx, pt_idx + 1)
            };

            blocks.insert(
                move_id.clone(),
                json!({
                    "opcode": "motion_gotoxy",
                    "next": next_id,
                    "parent": prev_id,
                    "inputs": {
                        "X": [1, [4, sx.to_string()]],
                        "Y": [1, [4, sy.to_string()]]
                    },
                    "fields": {},
                    "shadow": false,
                    "topLevel": false
                }),
            );
            prev_id = move_id.clone();

            if pt_idx == 0 {
                let pd_id = format!("path_{}_pd", p_idx);
                let next_pd_id = if path.len() > 1 {
                    format!("path_{}_pt_1_move", p_idx)
                } else {
                    format!("path_{}_pu", p_idx)
                };
                blocks.insert(
                    pd_id.clone(),
                    json!({
                        "opcode": "pen_penDown",
                        "next": next_pd_id,
                        "parent": move_id,
                        "inputs": {},
                        "fields": {},
                        "shadow": false,
                        "topLevel": false
                    }),
                );
                prev_id = pd_id.clone();
            }
        }

        let pu_id = format!("path_{}_pu", p_idx);
        let next_pu_id = if p_idx == all_paths.len() - 1 {
            None
        } else {
            Some(format!("path_{}_pt_0_move", p_idx + 1))
        };

        blocks.insert(
            pu_id.clone(),
            json!({
                "opcode": "pen_penUp",
                "next": next_pu_id,
                "parent": prev_id,
                "inputs": {},
                "fields": {},
                "shadow": false,
                "topLevel": false
            }),
        );
        prev_id = pu_id;
    }

    let project = json!({
        "targets": [
            {
                "isStage": true,
                "name": "Stage",
                "variables": {},
                "lists": {},
                "broadcasts": {},
                "blocks": {},
                "comments": {},
                "currentCostume": 0,
                "costumes": [
                    {
                        "assetId": "cd21514d0531fdffb22204e0ec5ed84a",
                        "name": "backdrop1",
                        "md5ext": "cd21514d0531fdffb22204e0ec5ed84a.svg",
                        "dataFormat": "svg",
                        "rotationCenterX": 240,
                        "rotationCenterY": 180
                    }
                ],
                "sounds": [],
                "volume": 100,
                "layerOrder": 0,
                "tempo": 60,
                "videoTransparency": 50,
                "videoState": "on",
                "textToSpeechLanguage": null
            },
            {
                "isStage": false,
                "name": "Vectomancy",
                "variables": {},
                "lists": {},
                "broadcasts": {},
                "blocks": blocks,
                "comments": {},
                "currentCostume": 0,
                "costumes": [
                    {
                        "assetId": "cd21514d0531fdffb22204e0ec5ed84a",
                        "name": "costume1",
                        "bitmapResolution": 1,
                        "md5ext": "cd21514d0531fdffb22204e0ec5ed84a.svg",
                        "dataFormat": "svg",
                        "rotationCenterX": 0,
                        "rotationCenterY": 0
                    }
                ],
                "sounds": [],
                "volume": 100,
                "layerOrder": 1,
                "visible": true,
                "x": 0,
                "y": 0,
                "size": 100,
                "direction": 90,
                "draggable": false,
                "rotationStyle": "all around"
            }
        ],
        "monitors": [],
        "extensions": ["pen"],
        "meta": {
            "semver": "3.0.0",
            "vm": "0.2.0-prerelease.20200501160352",
            "agent": "Vectomancy"
        }
    });

    let empty_svg = r#"<svg version="1.1" width="2" height="2" viewBox="-1 -1 2 2" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"></svg>"#;

    let file = File::create(output_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("project.json", options)
        .map_err(|e| VectomancyError::InvalidInput(format!("Zip error: {}", e)))?;
    zip.write_all(serde_json::to_string(&project).unwrap().as_bytes())
        .map_err(|e| VectomancyError::InvalidInput(format!("Zip write error: {}", e)))?;

    zip.start_file("cd21514d0531fdffb22204e0ec5ed84a.svg", options)
        .map_err(|e| VectomancyError::InvalidInput(format!("Zip error: {}", e)))?;
    zip.write_all(empty_svg.as_bytes())
        .map_err(|e| VectomancyError::InvalidInput(format!("Zip write error: {}", e)))?;

    zip.finish()
        .map_err(|e| VectomancyError::InvalidInput(format!("Zip finish error: {}", e)))?;

    Ok(())
}
