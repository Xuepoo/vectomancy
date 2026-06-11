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
}
