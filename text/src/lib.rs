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
}
