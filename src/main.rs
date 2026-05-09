use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use vectomancy::cli::{Cli, Commands};
use vectomancy::error::VectomancyError;
use vectomancy::models::MathExpressionAST;
use vectomancy::{cli, emitter, math, models, parser};

fn main() -> Result<(), VectomancyError> {
    let cli = Cli::parse();

    let (verbose, _run_args) = match &cli.command {
        Commands::Run(args) => (args.verbose, args),
    };

    let log_level = if verbose { Level::DEBUG } else { Level::INFO };
    let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("Starting Vectomancy");

    match cli.command {
        Commands::Run(args) => {
            info!("Running with input: {:?}", args.input);
            let output = parser::parse_file(&args.input)?;

            let ast = match output {
                models::ParserOutput::Paths(paths) => {
                    info!("Successfully extracted {} paths.", paths.len());
                    let config = vectomancy::config::Config::load();
                    let iters = args.chaikin_iters.or(config.chaikin_iters).unwrap_or(0);
                    let mode = args.mode.clone().unwrap_or(cli::Mode::Fourier);
                    match mode {
                        cli::Mode::Fourier => {
                            let mut strokes = Vec::new();
                            for path in paths {
                                let reduced = math::simplify_rdp(&path, 0.5); // Slightly higher tolerance to reduce noise
                                let smoothed = if iters > 0 {
                                    math::chaikin_smooth(&reduced, iters)
                                } else {
                                    reduced
                                };
                                if smoothed.len() > 3 {
                                    // We don't need TSP for raster paths since they are already ordered contours!
                                    let terms = math::perform_fft(&smoothed, args.terms)?;
                                    strokes.push(terms);
                                }
                            }
                            MathExpressionAST::Fourier { strokes }
                        }
                        cli::Mode::Spline => {
                            let mut all_equations = Vec::new();
                            for path in paths {
                                let reduced = math::simplify_rdp(&path, 0.5);
                                let smoothed = if iters > 0 {
                                    math::chaikin_smooth(&reduced, iters)
                                } else {
                                    reduced
                                };
                                if smoothed.len() > 2 {
                                    let segments = math::spline::fit_cubic_bezier(&smoothed);
                                    let equations = math::spline::build_splines(&segments);
                                    all_equations.extend(equations);
                                }
                            }
                            MathExpressionAST::Spline {
                                equations: all_equations,
                            }
                        }
                    }
                }
                models::ParserOutput::Segments(segs) => {
                    info!("Successfully extracted {} segments.", segs.len());
                    let mode = args.mode.clone().unwrap_or(cli::Mode::Spline);
                    match mode {
                        cli::Mode::Spline => {
                            let equations = math::spline::build_splines(&segs);
                            MathExpressionAST::Spline { equations }
                        }
                        cli::Mode::Fourier => {
                            let pts = math::spline::sample_segments(&segs, 100);
                            info!("Sampled {} points from segments.", pts.len());
                            // TSP
                            let ordered_points = math::solve_tsp_nearest_neighbor(pts);
                            // FFT
                            let terms = math::perform_fft(&ordered_points, args.terms)?;
                            MathExpressionAST::Fourier {
                                strokes: vec![terms],
                            }
                        }
                    }
                }
            };
            match &ast {
                MathExpressionAST::Fourier { strokes } => {
                    info!("Generated AST with {} strokes", strokes.len());
                }
                MathExpressionAST::Spline { equations } => {
                    info!("Generated AST with {} equations", equations.len());
                }
            }

            // Render
            emitter::emit_file(
                &ast,
                args.format.as_ref().unwrap_or(&cli::OutputFormat::Python),
                &args.output,
            )?;
            info!("Vectomancy execution completed successfully.");
        }
    }

    Ok(())
}
