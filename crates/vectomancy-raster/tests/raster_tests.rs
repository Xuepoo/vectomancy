use image::{ImageBuffer, Rgb};
use vectomancy_raster::decode_raster_memory;

fn encode_png(imgbuf: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> Vec<u8> {
    let mut bytes = Vec::new();
    imgbuf
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .unwrap();
    bytes
}

#[test]
fn test_raster_decoding() {
    let mut imgbuf: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(20, 20);
    for x in 0..20 {
        imgbuf.put_pixel(x, 10, Rgb([255u8, 255u8, 255u8]));
    }

    let bytes = encode_png(&imgbuf);

    let (polylines, dimensions) = decode_raster_memory(&bytes, false).unwrap();
    assert_eq!(dimensions, (20, 20));
    assert!(!polylines.is_empty());
    // A 1px-wide stroke produces Sobel edges on both of its sides, so Otsu
    // thinning yields two parallel skeleton lines (one per edge) rather than
    // the source stroke itself. What matters here is that each edge is
    // traced as a single continuous path end-to-end, not fragmented into
    // many short segments the way a naive branch-stopping walk would.
    assert_eq!(
        polylines.len(),
        2,
        "expected one skeleton path per Sobel edge"
    );
    for path in &polylines {
        assert!(
            path.geometry.points.len() >= 15,
            "each edge should be traced as one continuous run, not fragmented; got {} points",
            path.geometry.points.len()
        );
    }
}

#[test]
fn test_blank_image_has_no_paths() {
    // A uniformly colored image has no Sobel edges, so Otsu binarization
    // should leave the grid empty and skeleton extraction should return
    // nothing rather than panicking or fabricating stray paths.
    let imgbuf: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::from_pixel(16, 16, Rgb([128, 128, 128]));
    let bytes = encode_png(&imgbuf);

    let (polylines, dimensions) = decode_raster_memory(&bytes, false).unwrap();
    assert_eq!(dimensions, (16, 16));
    assert!(polylines.is_empty());
}

#[test]
fn test_closed_square_loop_is_traced() {
    // A hollow square has no degree-1 (endpoint) pixels, so it must be
    // picked up by the loop/remnant sweep in `extract_paths`, not the
    // endpoint-driven pass.
    let mut imgbuf: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(20, 20);
    for i in 4..16u32 {
        imgbuf.put_pixel(i, 4, Rgb([255, 255, 255]));
        imgbuf.put_pixel(i, 15, Rgb([255, 255, 255]));
        imgbuf.put_pixel(4, i, Rgb([255, 255, 255]));
        imgbuf.put_pixel(15, i, Rgb([255, 255, 255]));
    }

    let bytes = encode_png(&imgbuf);
    let (polylines, dimensions) = decode_raster_memory(&bytes, false).unwrap();
    assert_eq!(dimensions, (20, 20));
    assert!(
        !polylines.is_empty(),
        "closed loop must still be traced without any endpoint pixels"
    );
}

#[test]
fn test_color_sampling_produces_hex_style() {
    let mut imgbuf: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(20, 20);
    for x in 0..20 {
        imgbuf.put_pixel(x, 10, Rgb([200, 30, 30]));
    }
    let bytes = encode_png(&imgbuf);

    let (polylines, _) = decode_raster_memory(&bytes, true).unwrap();
    assert!(!polylines.is_empty());
    assert!(polylines.iter().any(|p| p.color_style.is_some()));
    for path in &polylines {
        if let Some(style) = &path.color_style {
            assert!(style.starts_with('#'));
            assert_eq!(style.len(), 7);
        }
    }
}

#[test]
fn test_tiny_image_does_not_panic() {
    // Degenerate 1x1 and 2x2 inputs are smaller than the thinning/tracing
    // algorithms' 3x3 neighborhood requirement; they must return cleanly
    // instead of panicking on out-of-bounds access.
    let imgbuf: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_pixel(1, 1, Rgb([255, 255, 255]));
    let bytes = encode_png(&imgbuf);
    let (polylines, dimensions) = decode_raster_memory(&bytes, false).unwrap();
    assert_eq!(dimensions, (1, 1));
    assert!(polylines.is_empty());

    let imgbuf: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_pixel(2, 2, Rgb([255, 255, 255]));
    let bytes = encode_png(&imgbuf);
    let (_polylines, dimensions) = decode_raster_memory(&bytes, false).unwrap();
    assert_eq!(dimensions, (2, 2));
}
