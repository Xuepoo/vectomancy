use vectomancy_export::{encode_json, encode_svg, encode_zlib_base64};
use vectomancy_geometry::{BoundingBox, Point2D, Polyline, PolylineScene, StyledPath};

#[test]
fn test_json_export() {
    let polyline = Polyline::new(
        vec![Point2D::new(0.0, 0.0), Point2D::new(10.0, 10.0)],
        false,
    );
    let json = encode_json(&polyline).unwrap();
    assert!(json.contains("points"));
}

#[test]
fn test_zlib_base64_export() {
    let polyline = Polyline::new(vec![Point2D::new(0.0, 0.0)], false);
    let encoded = encode_zlib_base64(&polyline).unwrap();
    assert!(!encoded.is_empty());
}

#[test]
fn test_svg_export() {
    let scene = PolylineScene {
        paths: vec![StyledPath::new(
            Polyline::new(
                vec![Point2D::new(0.0, 0.0), Point2D::new(10.0, 10.0)],
                false,
            ),
            Some("#ff0000".to_string()),
        )],
        dimensions: (100, 100),
        bounds: BoundingBox::new(0.0, 0.0, 10.0, 10.0),
    };

    let svg = encode_svg(&scene, 2.0);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("stroke=\"#ff0000\""));
    assert!(svg.contains("stroke-width=\"2\""));
}
