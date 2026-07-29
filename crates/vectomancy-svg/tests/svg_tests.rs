use vectomancy_svg::decode_svg_memory;

#[test]
fn test_svg_decoding() {
    let svg_xml = r#"<svg width="100" height="100" xmlns="http://www.w3.org/2000/svg">
        <path d="M 10 10 L 90 90" stroke="black"/>
    </svg>"#;

    let (paths, dimensions) = decode_svg_memory(svg_xml.as_bytes(), false).unwrap();
    assert_eq!(dimensions, (100, 100));
    assert_eq!(paths.len(), 1);
}
