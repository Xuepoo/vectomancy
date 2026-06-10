use std::fs;
use std::process::Command;

#[test]
fn test_invalid_subcommand() {
    let bin_path = env!("CARGO_BIN_EXE_vectomancy-cli");
    let output = Command::new(bin_path)
        .arg("invalid_subcommand_name")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
}

#[test]
fn test_no_subcommand() {
    let bin_path = env!("CARGO_BIN_EXE_vectomancy-cli");
    let output = Command::new(bin_path)
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
}

#[test]
fn test_image_subcommand_workflow() {
    let bin_path = env!("CARGO_BIN_EXE_vectomancy-cli");

    // Create a temporary directory via tempfile
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_dir_path = temp_dir.path();

    let input_path = temp_dir_path.join("dummy.png");
    let output_path = temp_dir_path.join("output.json");

    // Generate a valid 2x2 dummy PNG using image crate
    let img = image::ImageBuffer::from_fn(2, 2, |x, y| {
        if (x + y) % 2 == 0 {
            image::Rgb([0u8, 0u8, 0u8])
        } else {
            image::Rgb([255u8, 255u8, 255u8])
        }
    });
    img.save(&input_path).unwrap();

    let run_output = Command::new(bin_path)
        .arg("image")
        .arg(&input_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&run_output.stdout);
    let stderr = String::from_utf8_lossy(&run_output.stderr);

    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);

    assert!(
        run_output.status.success(),
        "CLI failed to process the image: {}",
        stderr
    );
    assert!(output_path.exists(), "Output JSON file was not created");

    // Read the output JSON
    let json_content = fs::read_to_string(&output_path).unwrap();
    assert!(!json_content.is_empty(), "Output JSON is empty");
}

#[test]
fn test_text_subcommand_workflow() {
    let bin_path = env!("CARGO_BIN_EXE_vectomancy-cli");

    // Create a temporary directory via tempfile
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_dir_path = temp_dir.path();

    let font_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("text")
        .join("tests")
        .join("font.ttf");

    let output_path = temp_dir_path.join("text_output.json");

    let run_output = Command::new(bin_path)
        .arg("text")
        .arg("hello")
        .arg("--font")
        .arg(&font_path)
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&run_output.stdout);
    let stderr = String::from_utf8_lossy(&run_output.stderr);

    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);

    assert!(
        run_output.status.success(),
        "CLI failed to process the text subcommand: {}",
        stderr
    );
    assert!(output_path.exists(), "Output JSON file was not created");

    // Read the output JSON
    let json_content = fs::read_to_string(&output_path).unwrap();
    assert!(!json_content.is_empty(), "Output JSON is empty");
}

#[test]
fn test_video_decoding_and_fitting_e2e() {
    let video_path = std::path::Path::new("/home/fuyu/Videos/YouTube/bad-apple.mp4");
    if !video_path.exists() {
        println!("Skipping video test because bad-apple.mp4 does not exist.");
        return;
    }

    // Call decode_video_to_channel directly
    let (receiver, join_handle) = vectomancy_video::decode_video_to_channel(video_path).unwrap();

    // Process only the first 10 frames
    let mut frames_processed = 0;
    for _ in 0..10 {
        if let Ok(frame) = receiver.recv() {
            let img = frame.to_image().unwrap();
            assert!(img.width() > 0);
            assert!(img.height() > 0);

            // Perform curve fitting to verify math pipeline
            let color = false;
            let (paths, _) =
                vectomancy::parser::raster::process_raster_image_core(img, color).unwrap();

            // Perform spline curve fitting
            let mut spline_count = 0;
            for path in paths {
                if path.data.len() > 5 {
                    let reduced = vectomancy::math::simplify_rdp(&path.data, 1.0);
                    if reduced.len() > 2 {
                        let segments = vectomancy::math::spline::fit_cubic_bezier(&reduced);
                        let _equations = vectomancy::math::spline::build_splines(&segments);
                        spline_count += 1;
                    }
                }
            }
            println!(
                "Processed frame {} with {} splines",
                frames_processed + 1,
                spline_count
            );
            frames_processed += 1;
        } else {
            break;
        }
    }

    // Dropping receiver causes the decoding thread to stop
    std::mem::drop(receiver);

    // Join the thread, it should exit successfully or handle the early termination cleanly
    let _ = join_handle.join().unwrap();

    assert_eq!(frames_processed, 10);
}
