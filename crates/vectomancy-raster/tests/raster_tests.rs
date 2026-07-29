use image::{ImageBuffer, Rgb};
use vectomancy_raster::decode_raster_memory;

#[test]
fn test_raster_decoding() {
    let mut imgbuf: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(20, 20);
    for x in 0..20 {
        imgbuf.put_pixel(x, 10, Rgb([255u8, 255u8, 255u8]));
    }

    let mut bytes = Vec::new();
    imgbuf
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .unwrap();

    let (polylines, dimensions) = decode_raster_memory(&bytes, false).unwrap();
    assert_eq!(dimensions, (20, 20));
    assert!(!polylines.is_empty());
}
