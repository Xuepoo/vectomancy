pub mod parser;

#[cfg(test)]
mod tests {
    use crate::parser::extract_char_outline;

    #[test]
    fn test_character_a_outline() {
        let font_data = include_bytes!("../tests/font.ttf");
        let paths = extract_char_outline('A', font_data, 64.0).unwrap();
        assert!(!paths.is_empty(), "Glyph outline paths must not be empty");
    }

    #[test]
    fn test_newline_and_controls() {
        let font_data = include_bytes!("../tests/font.ttf");
        let text = "Line 1\nLine 2\r\nLine 3\u{0007}\nLine 4\\nLine 5";
        let (paths, (_w, h)) =
            crate::parser::extract_text_outlines(text, font_data, 64.0, 0.0).unwrap();
        assert!(!paths.is_empty(), "Paths should not be empty");
        let single_line_height = (64.0 * 1.5) as u32;
        assert!(
            h > single_line_height,
            "Height should account for multiple lines, got h = {}",
            h
        );
    }

    #[test]
    fn test_woff2_decompression() {
        let font_data_ttf = include_bytes!("../tests/font.ttf");
        let font_data_woff2 = ttf2woff2::encode(font_data_ttf, Default::default())
            .expect("Failed to compress TTF to WOFF2");
        assert!(
            woff2::decode::is_woff2(&font_data_woff2),
            "Compressed data should be recognized as WOFF2"
        );

        let (paths_ttf, dim_ttf) =
            crate::parser::extract_text_outlines("Hello WOFF2", font_data_ttf, 32.0, 0.0).unwrap();
        let (paths_woff2, dim_woff2) =
            crate::parser::extract_text_outlines("Hello WOFF2", &font_data_woff2, 32.0, 0.0)
                .unwrap();

        assert_eq!(dim_ttf, dim_woff2, "Dimensions should match");
        assert_eq!(
            paths_ttf.len(),
            paths_woff2.len(),
            "Path counts should match"
        );
        for (p_ttf, p_woff2) in paths_ttf.iter().zip(paths_woff2.iter()) {
            assert_eq!(p_ttf.data, p_woff2.data);
        }
    }

    #[test]
    fn test_pressstart2p_woff2() {
        let font_data_woff2 =
            include_bytes!("../../../vectomancy-web/zola-site/static/fonts/pressstart2p.woff2");
        assert!(
            woff2::decode::is_woff2(font_data_woff2),
            "pressstart2p should be recognized as WOFF2"
        );
        let (paths, _dim) =
            crate::parser::extract_text_outlines("Hello", font_data_woff2, 32.0, 0.0).unwrap();
        assert!(!paths.is_empty(), "Paths should not be empty");
    }
}
