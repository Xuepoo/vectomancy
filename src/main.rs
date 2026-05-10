use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use vectomancy::cli::Cli;
use vectomancy::error::VectomancyError;
use vectomancy::models::MathExpressionAST;
use vectomancy::{cli, emitter, math, models, parser};

fn main() -> Result<(), VectomancyError> {
    let cli = Cli::parse();

    let verbose = cli.verbose;
    let log_level = if verbose { Level::DEBUG } else { Level::INFO };
    let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("Starting Vectomancy");

    let config = vectomancy::config::Config::load();
    let use_gpu = cli.gpu || config.gpu.unwrap_or(false);
    if use_gpu {
        tracing::warn!("GPU acceleration (wgpu) is currently an experimental stub and has no effect in this version. Falling back to CPU.");
    }

    let mut flattened_inputs = Vec::new();
    for input in &cli.inputs {
        if input.is_dir() {
            if let Ok(entries) = std::fs::read_dir(input) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            let ext_lower = ext.to_lowercase();
                            if ["png", "jpg", "jpeg", "svg"].contains(&ext_lower.as_str()) {
                                flattened_inputs.push(path);
                            }
                        }
                    }
                }
            }
        } else if input.is_file() {
            flattened_inputs.push(input.clone());
        }
    }

    if flattened_inputs.is_empty() {
        info!("No valid input files found.");
        return Ok(());
    }

    for input_path in flattened_inputs.iter() {
        info!("Running with input: {:?}", input_path);
        let output = parser::parse_file(input_path, cli.color)?;

        let (ast, original_dimensions) = match output {
            models::ParserOutput::Paths {
                paths,
                original_dimensions,
            } => {
                info!("Successfully extracted {} paths.", paths.len());
                let iters = cli.chaikin_iters.or(config.chaikin_iters).unwrap_or(0);
                let tolerance = if cli.tolerance != 0.5 {
                    cli.tolerance
                } else {
                    config.tolerance.unwrap_or(0.5)
                };
                let min_path_len = if cli.min_path_len != 5 {
                    cli.min_path_len
                } else {
                    config.min_path_len.unwrap_or(5)
                };
                let mode = cli.mode.clone().unwrap_or(cli::Mode::Fourier);
                let ast = match mode {
                    cli::Mode::Fourier => {
                        let mut strokes = Vec::new();
                        for path in paths {
                            if path.data.len() < min_path_len {
                                continue;
                            }
                            let reduced = math::simplify_rdp(&path.data, tolerance);
                            if reduced.len() > 3 {
                                let terms = math::perform_fft(&reduced, cli.terms)?;
                                strokes.push(models::ColoredPath {
                                    color_rgb: path.color_rgb,
                                    data: terms,
                                });
                            }
                        }
                        MathExpressionAST::Fourier { strokes }
                    }
                    cli::Mode::Spline => {
                        let mut all_equations = Vec::new();
                        for path in paths {
                            if path.data.len() < min_path_len {
                                continue;
                            }
                            let reduced = math::simplify_rdp(&path.data, tolerance);
                            if reduced.len() > 2 {
                                let segments = math::spline::fit_cubic_bezier(&reduced);
                                let equations = math::spline::build_splines(&segments);
                                all_equations.push(models::ColoredPath {
                                    color_rgb: path.color_rgb,
                                    data: equations,
                                });
                            }
                        }
                        MathExpressionAST::Spline {
                            equations: all_equations,
                        }
                    }
                    cli::Mode::Chaikin => {
                        let mut smoothed_paths = Vec::new();
                        for path in paths {
                            if path.data.len() < min_path_len {
                                continue;
                            }
                            let reduced = math::simplify_rdp(&path.data, tolerance);
                            let smoothed = if iters > 0 {
                                math::chaikin_smooth(&reduced, iters)
                            } else {
                                reduced
                            };
                            smoothed_paths.push(models::ColoredPath {
                                color_rgb: path.color_rgb,
                                data: smoothed,
                            });
                        }
                        MathExpressionAST::Polyline {
                            paths: smoothed_paths,
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
                let mode = cli.mode.clone().unwrap_or(cli::Mode::Spline);
                let ast = match mode {
                    cli::Mode::Spline => {
                        let mut all_equations = Vec::new();
                        for seg in segs {
                            let equations = math::spline::build_splines(&seg.data);
                            all_equations.push(models::ColoredPath {
                                color_rgb: seg.color_rgb,
                                data: equations,
                            });
                        }
                        MathExpressionAST::Spline {
                            equations: all_equations,
                        }
                    }
                    cli::Mode::Fourier => {
                        let mut strokes = Vec::new();
                        for seg in segs {
                            let pts = math::spline::sample_segments(&seg.data, 100);
                            info!("Sampled {} points from segments.", pts.len());
                            let ordered_points = math::solve_tsp_nearest_neighbor(pts);
                            let terms = math::perform_fft(&ordered_points, cli.terms)?;
                            strokes.push(models::ColoredPath {
                                color_rgb: seg.color_rgb,
                                data: terms,
                            });
                        }
                        MathExpressionAST::Fourier { strokes }
                    }
                    cli::Mode::Chaikin => {
                        let iters = cli.chaikin_iters.or(config.chaikin_iters).unwrap_or(0);
                        let mut paths = Vec::new();
                        for seg in segs {
                            let pts = math::spline::sample_segments(&seg.data, 100);
                            info!("Sampled {} points from segments.", pts.len());
                            let ordered_points = math::solve_tsp_nearest_neighbor(pts);
                            let smoothed = if iters > 0 {
                                math::chaikin_smooth(&ordered_points, iters)
                            } else {
                                ordered_points
                            };
                            paths.push(models::ColoredPath {
                                color_rgb: seg.color_rgb,
                                data: smoothed,
                            });
                        }
                        MathExpressionAST::Polyline { paths }
                    }
                };
                (ast, original_dimensions)
            }
        };

        match &ast {
            MathExpressionAST::Fourier { strokes } => {
                info!("Generated AST with {} strokes", strokes.len());
            }
            MathExpressionAST::Spline { equations } => {
                info!("Generated AST with {} equations", equations.len());
            }
            MathExpressionAST::Polyline { paths } => {
                info!("Generated AST with {} paths", paths.len());
            }
        }

        let format = cli
            .format
            .as_ref()
            .or(config.format.as_ref())
            .unwrap_or(&cli::OutputFormat::Png);
        let ext = match format {
            cli::OutputFormat::Png => "png",
            cli::OutputFormat::Jpg => "jpg",
            cli::OutputFormat::Webp => "webp",
            cli::OutputFormat::Python => "py",
            cli::OutputFormat::Latex => "tex",
            cli::OutputFormat::Html => "html",
            cli::OutputFormat::Json => "json",
            cli::OutputFormat::Geogebra => "ggb",
            cli::OutputFormat::Wolfram => "txt",
            cli::OutputFormat::Kmplot => "fkt",
        };

        let base_name = input_path.file_stem().unwrap().to_string_lossy();
        let target_filename = format!("{}_vectomancy.{}", base_name, ext);

        let final_output = if let Some(ref out_path) = cli.output {
            if flattened_inputs.len() == 1 && !out_path.is_dir() && out_path.extension().is_some() {
                out_path.clone()
            } else {
                std::fs::create_dir_all(out_path)?;
                out_path.join(&target_filename)
            }
        } else {
            let out_dir = config
                .default_output_dir
                .clone()
                .unwrap_or_else(|| std::path::PathBuf::from("."));
            std::fs::create_dir_all(&out_dir)?;
            out_dir.join(&target_filename)
        };

        match format {
            cli::OutputFormat::Png | cli::OutputFormat::Jpg | cli::OutputFormat::Webp => {
                let target_dimensions = match (cli.width, cli.height) {
                    (None, None) => original_dimensions,
                    (Some(w), None) => {
                        let h = (w as f32 * original_dimensions.1 as f32
                            / original_dimensions.0 as f32) as u32;
                        (w, h)
                    }
                    (None, Some(h)) => {
                        let w = (h as f32 * original_dimensions.0 as f32
                            / original_dimensions.1 as f32) as u32;
                        (w, h)
                    }
                    (Some(w), Some(h)) => (w, h),
                };

                let bit_depth = cli.bit_depth.or(config.bit_depth);
                let color_space = cli
                    .color_space
                    .clone()
                    .or_else(|| config.color_space.clone());
                let bg_transparent = if cli.bg_transparent {
                    true
                } else {
                    config.bg_transparent.unwrap_or(false)
                };

                emitter::native::render_to_image(
                    &ast,
                    &final_output,
                    format,
                    bg_transparent,
                    original_dimensions,
                    target_dimensions,
                    cli.stroke_width,
                    bit_depth,
                    color_space,
                )?;
            }
            _ => {
                emitter::emit_file(&ast, format, &final_output)?;
            }
        }
        info!("Saved output to {:?}", final_output);
    }

    info!("Vectomancy batch execution completed successfully.");
    Ok(())
}
