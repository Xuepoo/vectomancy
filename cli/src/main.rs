use clap::{CommandFactory, Parser};
use rayon::prelude::*;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
mod cli;

use cli::{Cli, Commands};
use vectomancy::config::{Mode, OutputFormat};
use vectomancy::error::VectomancyError;
use vectomancy::models::MathExpressionAST;
use vectomancy::{emitter, math, models, parser};

fn parse_hex_color(col: &str) -> Option<[f32; 3]> {
    let col = col.trim();
    if !col.is_ascii() {
        return None;
    }
    if !col.starts_with('#') || col.len() != 7 {
        return None;
    }
    let r = u8::from_str_radix(&col[1..3], 16).ok()?;
    let g = u8::from_str_radix(&col[3..5], 16).ok()?;
    let b = u8::from_str_radix(&col[5..7], 16).ok()?;
    Some([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0])
}

fn main() -> Result<(), VectomancyError> {
    let cli = Cli::parse();

    if let Some(shell) = cli.generate_completions {
        let mut cmd = Cli::command();
        clap_complete::generate(shell, &mut cmd, "vectomancy", &mut std::io::stdout());
        return Ok(());
    }

    let verbose = cli.verbose;
    let log_level = if verbose { Level::DEBUG } else { Level::INFO };
    let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let start_time = std::time::Instant::now();
    info!("Starting Vectomancy");

    let config = vectomancy::config::Config::load(cli.config.clone());

    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            return Err(VectomancyError::InvalidInput(
                "A subcommand (image, video, text) is required.".to_string(),
            ));
        }
    };

    match command {
        Commands::Image(args) => {
            let image_config = config.image.unwrap_or_default();

            let use_gpu = args
                .gpu
                .unwrap_or_else(|| image_config.gpu.unwrap_or(false));
            let color = args
                .color
                .unwrap_or_else(|| image_config.color.unwrap_or(false));
            let bg_transparent = args
                .bg_transparent
                .unwrap_or_else(|| image_config.bg_transparent.unwrap_or(false));

            if use_gpu {
                tracing::info!("GPU acceleration (wgpu) is enabled.");

                let power_pref_str = args
                    .gpu_power
                    .clone()
                    .or_else(|| image_config.gpu_power.clone())
                    .unwrap_or_else(|| "HighPerformance".to_string());

                let power_pref = match power_pref_str.to_lowercase().as_str() {
                    "lowpower" => wgpu::PowerPreference::LowPower,
                    "none" => wgpu::PowerPreference::None,
                    _ => wgpu::PowerPreference::HighPerformance,
                };
                #[cfg(not(target_arch = "wasm32"))]
                vectomancy::math::wgpu_math::init_context(power_pref);
            }

            let requested_threads = args.threads.or(image_config.threads).unwrap_or(1);
            let max_threads = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1);
            let num_threads = if requested_threads == 0 {
                max_threads
            } else {
                requested_threads.clamp(1, max_threads)
            };

            if let Err(e) = rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build_global()
            {
                tracing::warn!("Failed to initialize rayon thread pool: {}", e);
            } else if num_threads > 1 {
                tracing::info!("CPU Multithreading enabled with {} threads.", num_threads);
            } else {
                tracing::info!("Running in single-threaded CPU mode.");
            }

            let mut flattened_inputs = Vec::new();
            for input in &args.inputs {
                if !input.exists() {
                    return Err(VectomancyError::InvalidInput(format!(
                        "Input path does not exist: {:?}",
                        input
                    )));
                }
                if input.is_dir() {
                    let mut found = false;
                    if let Ok(entries) = std::fs::read_dir(input) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_file() {
                                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                                    let ext_lower = ext.to_lowercase();
                                    if ["png", "jpg", "jpeg", "svg"].contains(&ext_lower.as_str()) {
                                        flattened_inputs.push(path);
                                        found = true;
                                    }
                                }
                            }
                        }
                    }
                    if !found {
                        return Err(VectomancyError::InvalidInput(format!(
                            "No valid image files found in directory: {:?}",
                            input
                        )));
                    }
                } else if input.is_file() {
                    if let Some(ext) = input.extension().and_then(|e| e.to_str()) {
                        let ext_lower = ext.to_lowercase();
                        if ["png", "jpg", "jpeg", "svg"].contains(&ext_lower.as_str()) {
                            flattened_inputs.push(input.clone());
                        } else {
                            return Err(VectomancyError::InvalidInput(format!(
                                "Unsupported file extension: {:?}",
                                input
                            )));
                        }
                    } else {
                        return Err(VectomancyError::InvalidInput(format!(
                            "No file extension found: {:?}",
                            input
                        )));
                    }
                } else {
                    return Err(VectomancyError::InvalidInput(format!(
                        "Input path is neither a file nor a directory: {:?}",
                        input
                    )));
                }
            }

            if flattened_inputs.is_empty() {
                return Err(VectomancyError::InvalidInput(
                    "No valid input files provided.".to_string(),
                ));
            }

            for input_path in flattened_inputs.iter() {
                info!("Running with input: {:?}", input_path);
                let output = parser::parse_file(input_path, color)?;

                let (ast, original_dimensions) = match output {
                    models::ParserOutput::Paths {
                        paths,
                        original_dimensions,
                    } => {
                        info!("Successfully extracted {} paths.", paths.len());
                        let iters = args
                            .chaikin_iters
                            .or(image_config.chaikin_iters)
                            .unwrap_or(0);
                        let detail_val = args.detail.or(image_config.detail).unwrap_or(50);
                        let tolerance =
                            args.tolerance
                                .or(image_config.tolerance)
                                .unwrap_or_else(|| {
                                    let detail_clamped = detail_val.clamp(1, 100) as f64;
                                    5.0 * (1.0 - (detail_clamped / 100.0)).powi(2) + 0.1
                                });
                        let min_path_len =
                            args.min_path_len.or(image_config.min_path_len).unwrap_or(5);
                        let bbox = math::compute_bounding_box(&paths);
                        let mode = args
                            .mode
                            .map(Mode::from)
                            .or(image_config.mode)
                            .unwrap_or(Mode::Spline);
                        let ast = match mode {
                            Mode::Fourier => {
                                let mut valid_paths = Vec::new();
                                let mut valid_colors = Vec::new();
                                for path in paths {
                                    if path.data.len() < min_path_len {
                                        continue;
                                    }
                                    let reduced = math::simplify_rdp(&path.data, tolerance);
                                    if reduced.len() > 3 {
                                        valid_paths.push(reduced);
                                        valid_colors.push(path.color_style.clone());
                                    }
                                }

                                let path_refs: Vec<&[models::Point2D]> =
                                    valid_paths.iter().map(|p| p.as_slice()).collect();
                                let terms = args.terms.or(image_config.terms).unwrap_or(1000);
                                let batch_results =
                                    math::perform_fft_batch(&path_refs, terms, use_gpu)?;

                                let mut strokes = Vec::new();
                                for (terms, color) in batch_results.into_iter().zip(valid_colors) {
                                    strokes.push(models::ColoredPath {
                                        color_style: color,
                                        data: terms,
                                    });
                                }
                                MathExpressionAST::Fourier {
                                    strokes,
                                    bounding_box: [0.0, 0.0, 0.0, 0.0],
                                }
                            }
                            Mode::Spline => {
                                let all_equations: Vec<_> = paths
                                    .into_par_iter()
                                    .filter_map(|path| {
                                        if path.data.len() < min_path_len {
                                            return None;
                                        }
                                        let reduced = math::simplify_rdp(&path.data, tolerance);
                                        if reduced.len() > 2 {
                                            let segments = math::spline::fit_cubic_bezier(&reduced);
                                            let equations = math::spline::build_splines(&segments);
                                            Some(models::ColoredPath {
                                                color_style: path.color_style.clone(),
                                                data: equations,
                                            })
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                                MathExpressionAST::Spline {
                                    equations: all_equations,
                                    bounding_box: bbox,
                                }
                            }
                            Mode::Chaikin => {
                                let smoothed_paths: Vec<_> = paths
                                    .into_par_iter()
                                    .filter_map(|path| {
                                        if path.data.len() < min_path_len {
                                            return None;
                                        }
                                        let reduced = math::simplify_rdp(&path.data, tolerance);
                                        let smoothed = if iters > 0 {
                                            math::chaikin_smooth(&reduced, iters)
                                        } else {
                                            reduced
                                        };
                                        Some(models::ColoredPath {
                                            color_style: path.color_style.clone(),
                                            data: smoothed,
                                        })
                                    })
                                    .collect();
                                MathExpressionAST::Polyline {
                                    paths: smoothed_paths,
                                    bounding_box: bbox,
                                }
                            }
                        };
                        (ast, original_dimensions)
                    }
                    models::ParserOutput::Segments {
                        segments: segs,
                        original_dimensions,
                    } => {
                        info!("Successfully extracted {} segments.", segs.len());
                        let bbox = math::compute_bounding_box_segments(&segs);
                        let mode = args
                            .mode
                            .map(Mode::from)
                            .or(image_config.mode)
                            .unwrap_or(Mode::Spline);
                        let ast = match mode {
                            Mode::Spline => {
                                let all_equations: Vec<_> = segs
                                    .into_par_iter()
                                    .map(|seg| {
                                        let equations = math::spline::build_splines(&seg.data);
                                        models::ColoredPath {
                                            color_style: seg.color_style.clone(),
                                            data: equations,
                                        }
                                    })
                                    .collect();
                                MathExpressionAST::Spline {
                                    equations: all_equations,
                                    bounding_box: [0.0, 0.0, 0.0, 0.0],
                                }
                            }
                            Mode::Fourier => {
                                let mut valid_paths = Vec::new();
                                let mut valid_colors = Vec::new();
                                for seg in segs {
                                    let pts = math::spline::sample_segments(&seg.data, 100);
                                    let ordered_points = math::solve_tsp_nearest_neighbor(pts);
                                    valid_paths.push(ordered_points);
                                    valid_colors.push(seg.color_style.clone());
                                }

                                let path_refs: Vec<&[models::Point2D]> =
                                    valid_paths.iter().map(|p| p.as_slice()).collect();
                                let terms = args.terms.or(image_config.terms).unwrap_or(1000);
                                let batch_results =
                                    math::perform_fft_batch(&path_refs, terms, use_gpu)?;

                                let mut strokes = Vec::new();
                                for (terms, color) in batch_results.into_iter().zip(valid_colors) {
                                    strokes.push(models::ColoredPath {
                                        color_style: color,
                                        data: terms,
                                    });
                                }
                                MathExpressionAST::Fourier {
                                    strokes,
                                    bounding_box: [0.0, 0.0, 0.0, 0.0],
                                }
                            }
                            Mode::Chaikin => {
                                let iters = args
                                    .chaikin_iters
                                    .or(image_config.chaikin_iters)
                                    .unwrap_or(0);
                                let paths: Vec<_> = segs
                                    .into_par_iter()
                                    .map(|seg| {
                                        let pts = math::spline::sample_segments(&seg.data, 100);
                                        let ordered_points = math::solve_tsp_nearest_neighbor(pts);
                                        let smoothed = if iters > 0 {
                                            math::chaikin_smooth(&ordered_points, iters)
                                        } else {
                                            ordered_points
                                        };
                                        models::ColoredPath {
                                            color_style: seg.color_style.clone(),
                                            data: smoothed,
                                        }
                                    })
                                    .collect();
                                MathExpressionAST::Polyline {
                                    paths,
                                    bounding_box: bbox,
                                }
                            }
                        };
                        (ast, original_dimensions)
                    }
                };

                match &ast {
                    MathExpressionAST::Fourier {
                        strokes,
                        bounding_box: _,
                    } => {
                        info!("Generated AST with {} strokes", strokes.len());
                    }
                    MathExpressionAST::Spline {
                        equations,
                        bounding_box: _,
                    } => {
                        info!("Generated AST with {} equations", equations.len());
                    }
                    MathExpressionAST::Polyline {
                        paths,
                        bounding_box: _,
                    } => {
                        info!("Generated AST with {} paths", paths.len());
                    }
                }

                let format = args
                    .format
                    .map(OutputFormat::from)
                    .or(image_config.format)
                    .unwrap_or(OutputFormat::Png);
                let ext = match format {
                    OutputFormat::Png => "png",
                    OutputFormat::Jpg => "jpg",
                    OutputFormat::Webp => "webp",
                    OutputFormat::Python => "py",
                    OutputFormat::Html => "html",
                    OutputFormat::Json => "json",
                    OutputFormat::Desmos => "html",
                };

                let base_name = input_path.file_stem().unwrap().to_string_lossy();
                let target_filename = format!("{}_vectomancy.{}", base_name, ext);

                let final_output = if let Some(ref out_path) = args.output {
                    if flattened_inputs.len() == 1
                        && !out_path.is_dir()
                        && out_path.extension().is_some()
                    {
                        out_path.clone()
                    } else {
                        std::fs::create_dir_all(out_path)?;
                        out_path.join(&target_filename)
                    }
                } else {
                    let out_dir = image_config
                        .default_output_dir
                        .clone()
                        .unwrap_or_else(|| std::path::PathBuf::from("."));
                    std::fs::create_dir_all(&out_dir)?;
                    out_dir.join(&target_filename)
                };

                match format {
                    OutputFormat::Png | OutputFormat::Jpg | OutputFormat::Webp => {
                        let target_dimensions = match (args.width, args.height) {
                            (None, None) => original_dimensions,
                            (Some(w), None) => {
                                let h = (w as f32 * original_dimensions.1 as f32
                                    / original_dimensions.0 as f32)
                                    as u32;
                                (w, h)
                            }
                            (None, Some(h)) => {
                                let w = (h as f32 * original_dimensions.0 as f32
                                    / original_dimensions.1 as f32)
                                    as u32;
                                (w, h)
                            }
                            (Some(w), Some(h)) => (w, h),
                        };

                        let bit_depth = args.bit_depth.or(image_config.bit_depth);
                        let color_space = args
                            .color_space
                            .clone()
                            .or_else(|| image_config.color_space.clone());
                        let stroke_width = args.stroke_width.unwrap_or(1.0);

                        emitter::native::render_to_image(
                            &ast,
                            &final_output,
                            &format,
                            bg_transparent,
                            original_dimensions,
                            target_dimensions,
                            stroke_width,
                            bit_depth,
                            color_space,
                        )?;
                    }
                    _ => {
                        emitter::emit_file(&ast, &format, &final_output, original_dimensions)?;
                    }
                }
                info!("Saved output to {:?}", final_output);
            }
        }
        Commands::Video(args) => {
            info!("Running Video Subcommand on {:?}", args.input);
            if !args.input.exists() {
                return Err(VectomancyError::InvalidInput(format!(
                    "Input video path does not exist: {:?}",
                    args.input
                )));
            }

            let (receiver, join_handle) = vectomancy_video::decode_video_to_channel(&args.input)
                .map_err(|e| VectomancyError::ImageProcessing(e.to_string()))?;

            let image_config = config.image.unwrap_or_default();
            let mode = image_config.mode.unwrap_or(Mode::Spline);
            let detail_val = image_config.detail.unwrap_or(50);
            let tolerance = image_config.tolerance.unwrap_or_else(|| {
                let detail_clamped = detail_val.clamp(1, 100) as f64;
                5.0 * (1.0 - (detail_clamped / 100.0)).powi(2) + 0.1
            });
            let min_path_len = image_config.min_path_len.unwrap_or(5);
            let format = image_config.format.unwrap_or(OutputFormat::Json);
            let color = image_config.color.unwrap_or(false);

            let mut is_video_output = false;
            let out_target = args
                .output
                .clone()
                .unwrap_or_else(|| std::path::PathBuf::from("output_video_frames"));

            if let Some(ext) = out_target.extension().and_then(|e| e.to_str()) {
                if ["mp4", "mkv", "webm", "avi", "mov", "gif"]
                    .contains(&ext.to_lowercase().as_str())
                {
                    is_video_output = true;
                }
            }

            let temp_dir = if is_video_output {
                let dir =
                    std::env::temp_dir().join(format!("vectomancy_video_{}", std::process::id()));
                std::fs::create_dir_all(&dir)?;
                dir
            } else {
                std::fs::create_dir_all(&out_target)?;
                out_target.clone()
            };

            let format = if is_video_output {
                OutputFormat::Png
            } else {
                format
            };

            let mut frame_idx = 0;
            while let Ok(frame_wrap) = receiver.recv() {
                frame_idx += 1;
                info!("Processing video frame {}", frame_idx);

                let img = frame_wrap
                    .to_image()
                    .map_err(|e| VectomancyError::ImageProcessing(e.to_string()))?;

                let (paths, original_dimensions) =
                    parser::raster::process_raster_image_core(img, color)?;

                let bbox = math::compute_bounding_box(&paths);
                let ast = match mode {
                    Mode::Spline => {
                        let all_equations: Vec<_> = paths
                            .into_par_iter()
                            .filter_map(|path| {
                                if path.data.len() < min_path_len {
                                    return None;
                                }
                                let reduced = math::simplify_rdp(&path.data, tolerance);
                                if reduced.len() > 2 {
                                    let segments = math::spline::fit_cubic_bezier(&reduced);
                                    let equations = math::spline::build_splines(&segments);
                                    Some(models::ColoredPath {
                                        color_style: path.color_style.clone(),
                                        data: equations,
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect();
                        MathExpressionAST::Spline {
                            equations: all_equations,
                            bounding_box: bbox,
                        }
                    }
                    Mode::Fourier => {
                        let mut valid_paths = Vec::new();
                        let mut valid_colors = Vec::new();
                        for path in paths {
                            if path.data.len() < min_path_len {
                                continue;
                            }
                            let reduced = math::simplify_rdp(&path.data, tolerance);
                            if reduced.len() > 3 {
                                valid_paths.push(reduced);
                                valid_colors.push(path.color_style.clone());
                            }
                        }

                        let path_refs: Vec<&[models::Point2D]> =
                            valid_paths.iter().map(|p| p.as_slice()).collect();
                        let terms = image_config.terms.unwrap_or(100);
                        let use_gpu = image_config.gpu.unwrap_or(false);
                        let batch_results = math::perform_fft_batch(&path_refs, terms, use_gpu)?;

                        let mut strokes = Vec::new();
                        for (terms, color) in batch_results.into_iter().zip(valid_colors) {
                            strokes.push(models::ColoredPath {
                                color_style: color,
                                data: terms,
                            });
                        }
                        MathExpressionAST::Fourier {
                            strokes,
                            bounding_box: [0.0, 0.0, 0.0, 0.0],
                        }
                    }
                    Mode::Chaikin => {
                        let iters = image_config.chaikin_iters.unwrap_or(0);
                        let smoothed_paths: Vec<_> = paths
                            .into_par_iter()
                            .filter_map(|path| {
                                if path.data.len() < min_path_len {
                                    return None;
                                }
                                let reduced = math::simplify_rdp(&path.data, tolerance);
                                let smoothed = if iters > 0 {
                                    math::chaikin_smooth(&reduced, iters)
                                } else {
                                    reduced
                                };
                                Some(models::ColoredPath {
                                    color_style: path.color_style.clone(),
                                    data: smoothed,
                                })
                            })
                            .collect();
                        MathExpressionAST::Polyline {
                            paths: smoothed_paths,
                            bounding_box: bbox,
                        }
                    }
                };

                let ext = match format {
                    OutputFormat::Png => "png",
                    OutputFormat::Jpg => "jpg",
                    OutputFormat::Webp => "webp",
                    OutputFormat::Python => "py",
                    OutputFormat::Html => "html",
                    OutputFormat::Json => "json",
                    OutputFormat::Desmos => "html",
                };

                let frame_filename = format!("frame_{:04}.{}", frame_idx, ext);
                let final_output = temp_dir.join(&frame_filename);

                match format {
                    OutputFormat::Png | OutputFormat::Jpg | OutputFormat::Webp => {
                        let bg_transparent = image_config.bg_transparent.unwrap_or(false);
                        let stroke_width = 1.0;
                        let bit_depth = image_config.bit_depth;
                        let color_space = image_config.color_space.clone();

                        emitter::native::render_to_image(
                            &ast,
                            &final_output,
                            &format,
                            bg_transparent,
                            original_dimensions,
                            original_dimensions,
                            stroke_width,
                            bit_depth,
                            color_space,
                        )?;
                    }
                    _ => {
                        emitter::emit_file(&ast, &format, &final_output, original_dimensions)?;
                    }
                }
                info!("Saved video frame to {:?}", final_output);
            }

            join_handle
                .join()
                .map_err(|_| {
                    VectomancyError::ImageProcessing("Decoder thread panicked".to_string())
                })?
                .map_err(|e| VectomancyError::ImageProcessing(e.to_string()))?;

            if is_video_output {
                info!("Stitching frames into video: {:?}", out_target);
                let status = std::process::Command::new("ffmpeg")
                    .arg("-y")
                    .arg("-framerate")
                    .arg("30")
                    .arg("-i")
                    .arg(temp_dir.join("frame_%04d.png").to_string_lossy().as_ref())
                    .arg("-c:v")
                    .arg("libx264")
                    .arg("-pix_fmt")
                    .arg("yuv420p")
                    .arg(&out_target)
                    .status();

                match status {
                    Ok(s) if s.success() => {
                        info!("Successfully generated video {:?}", out_target);
                        let _ = std::fs::remove_dir_all(&temp_dir);
                    }
                    Ok(s) => tracing::error!("ffmpeg exited with error status: {}", s),
                    Err(e) => tracing::error!("Failed to execute ffmpeg: {}", e),
                }
            }
        }

        Commands::Text(args) => {
            let text_config = config.text.clone().unwrap_or_default();
            info!("Running Text Subcommand with {:?}", args.text);

            let font_path = args
                .font
                .clone()
                .or_else(|| text_config.font.map(std::path::PathBuf::from));
            let font_path = match font_path {
                Some(p) => p,
                None => {
                    return Err(VectomancyError::InvalidInput(
                        "No font provided. Pass --font or configure it in config.toml".to_string(),
                    ))
                }
            };

            if !font_path.exists() {
                return Err(VectomancyError::InvalidInput(format!(
                    "Font file path does not exist: {:?}",
                    font_path
                )));
            }
            let font_bytes = std::fs::read(&font_path)?;

            let (segs, original_dimensions) =
                vectomancy_text::parser::extract_text_outlines(&args.text, &font_bytes, 64.0)
                    .map_err(VectomancyError::InvalidInput)?;

            let bbox = math::compute_bounding_box_segments(&segs);

            // Parse color or gradient from CLI or Config
            let color_str = args.color.clone().or_else(|| text_config.color.clone());
            let grad_str = args
                .gradient
                .clone()
                .or_else(|| text_config.gradient.clone());

            let mut final_color_style = None;
            if let Some(grad) = grad_str {
                let parts: Vec<&str> = grad.split(',').collect();
                if parts.len() == 3 {
                    if let (Some(start), Some(end), Ok(angle)) = (
                        parse_hex_color(parts[0]),
                        parse_hex_color(parts[1]),
                        parts[2].trim().parse::<f32>(),
                    ) {
                        let rad = angle.to_radians();
                        let start_pos = [0.0, 0.5];
                        let end_pos = if rad.is_finite() {
                            [0.5 + rad.cos() * 0.5, 0.5 + rad.sin() * 0.5]
                        } else {
                            [1.0, 0.5]
                        };
                        let stops = vec![(0.0, start), (1.0, end)];
                        final_color_style = Some(models::ColorStyle::LinearGradient(
                            std::sync::Arc::new(models::GradientData {
                                stops,
                                start_pos,
                                end_pos,
                            }),
                        ));
                    }
                }
            } else if let Some(col) = color_str {
                if let Some(rgb) = parse_hex_color(&col) {
                    final_color_style = Some(models::ColorStyle::Solid(rgb));
                }
            }

            let all_equations: Vec<_> = segs
                .into_par_iter()
                .map(|seg| {
                    let equations = math::spline::build_splines(&seg.data);
                    models::ColoredPath {
                        color_style: final_color_style
                            .clone()
                            .or_else(|| seg.color_style.clone()),
                        data: equations,
                    }
                })
                .collect();

            let ast = MathExpressionAST::Spline {
                equations: all_equations,
                bounding_box: bbox,
            };

            let final_output = if let Some(ref out_path) = args.output {
                out_path.clone()
            } else {
                std::path::PathBuf::from("text_output.json")
            };

            let format = if let Some(ext) = final_output.extension().and_then(|e| e.to_str()) {
                match ext.to_lowercase().as_str() {
                    "py" => OutputFormat::Python,
                    "html" => OutputFormat::Html,
                    "json" => OutputFormat::Json,
                    "desmos" => OutputFormat::Desmos,
                    "png" => OutputFormat::Png,
                    "jpg" | "jpeg" => OutputFormat::Jpg,
                    "webp" => OutputFormat::Webp,
                    _ => OutputFormat::Json,
                }
            } else {
                OutputFormat::Json
            };

            match format {
                OutputFormat::Png | OutputFormat::Jpg | OutputFormat::Webp => {
                    let image_config = config.image.unwrap_or_default();
                    let bg_transparent = args
                        .bg_transparent
                        .or(text_config.bg_transparent)
                        .unwrap_or_else(|| image_config.bg_transparent.unwrap_or(false));
                    let stroke_width = args
                        .stroke_width
                        .or(text_config.stroke_width)
                        .unwrap_or(1.0);
                    let bit_depth = image_config.bit_depth;
                    let color_space = image_config.color_space.clone();

                    emitter::native::render_to_image(
                        &ast,
                        &final_output,
                        &format,
                        bg_transparent,
                        original_dimensions,
                        original_dimensions,
                        stroke_width,
                        bit_depth,
                        color_space,
                    )?;
                }
                _ => {
                    emitter::emit_file(&ast, &format, &final_output, original_dimensions)?;
                }
            }
            info!("Saved text output to {:?}", final_output);
        }
    }

    info!(
        "Vectomancy batch execution completed successfully in {:.2?}.",
        start_time.elapsed()
    );
    Ok(())
}
