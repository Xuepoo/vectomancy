use image::{ImageBuffer, Rgb};
use vectomancy_pipeline::{ConversionMode, ConvertedScene, Pipeline, PipelineOptions};

#[test]
fn test_pipeline_conversion() {
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

    let pipeline = Pipeline::new(PipelineOptions {
        mode: ConversionMode::Chaikin { iterations: 1 },
        rdp_epsilon: 0.1,
        color: false,
    });

    let scene = pipeline.convert_raster_bytes(&bytes).unwrap();
    match scene {
        ConvertedScene::Polyline(poly_scene) => {
            assert_eq!(poly_scene.dimensions, (20, 20));
            assert!(!poly_scene.paths.is_empty());
        }
        _ => panic!("Expected Polyline scene"),
    }
}
