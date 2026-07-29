use base64::Engine;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use serde::Serialize;
use std::io::Write;
use vectomancy_geometry::PolylineScene;

pub fn encode_json<T: Serialize>(data: &T) -> Result<String, String> {
    serde_json::to_string_pretty(data).map_err(|e| e.to_string())
}

pub fn encode_zlib_base64<T: Serialize>(data: &T) -> Result<String, String> {
    let json_str = serde_json::to_string(data).map_err(|e| e.to_string())?;

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(json_str.as_bytes())
        .map_err(|e| e.to_string())?;
    let compressed_bytes = encoder.finish().map_err(|e| e.to_string())?;

    Ok(base64::engine::general_purpose::STANDARD.encode(compressed_bytes))
}

pub fn encode_svg(scene: &PolylineScene) -> String {
    let (width, height) = scene.dimensions;
    let mut svg = format!(
        r#"<svg width="{}" height="{}" viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">"#,
        width, height, width, height
    );
    svg.push('\n');

    for path in &scene.paths {
        if path.geometry.points.is_empty() {
            continue;
        }

        let stroke_color = path.color_style.as_deref().unwrap_or("black");
        svg.push_str(&format!(
            r#"  <path d="M {} {}" fill="none" stroke="{}" stroke-width="1""#,
            path.geometry.points[0].x, path.geometry.points[0].y, stroke_color
        ));

        for pt in &path.geometry.points[1..] {
            svg.push_str(&format!(" L {} {}", pt.x, pt.y));
        }

        if path.geometry.closed {
            svg.push_str(" Z");
        }

        svg.push_str(r#"" />"#);
        svg.push('\n');
    }

    svg.push_str("</svg>");
    svg
}
