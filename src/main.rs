use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use vectomancy::cli::{Cli, Commands};
use vectomancy::error::VectomancyError;
use vectomancy::{cli, emitter, math, models, parser};
use vectomancy::models::MathExpressionAST;

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
                models::ParserOutput::Points(pts) => {
                    info!("Successfully extracted {} points.", pts.len());
                    // RDP
                    let reduced_points = math::simplify_rdp(&pts, 0.2); // Default tolerance
                    info!("RDP reduced points to {}", reduced_points.len());

                    // TSP
                    let ordered_points = math::solve_tsp_nearest_neighbor(reduced_points);

                    // FFT
                    math::perform_fft(&ordered_points, args.terms)?
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
                            math::perform_fft(&ordered_points, args.terms)?
                        }
                    }
                }
            };
            match &ast {
                MathExpressionAST::Fourier { terms } => {
                    info!("Generated AST with {} terms", terms.len());
                }
                MathExpressionAST::Spline { equations } => {
                    info!("Generated AST with {} equations", equations.len());
                }
            }

            // Render
            emitter::emit_file(&ast, &args.format, &args.output)?;
            info!("Vectomancy execution completed successfully.");
        }
    }

    Ok(())
}
