pub mod cli;
pub mod emitter;
pub mod error;
pub mod math;
pub mod models;
pub mod parser;

use clap::Parser;
use cli::{Cli, Commands};
use models::MathExpressionAST;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

fn main() -> Result<(), error::VectomancyError> {
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

            let points = match output {
                models::ParserOutput::Points(pts) => {
                    info!("Successfully extracted {} points.", pts.len());
                    pts
                }
                models::ParserOutput::Segments(segs) => {
                    info!("Successfully extracted {} segments.", segs.len());
                    // For now, just return empty points or convert segments to points
                    // This is a placeholder since main.rs is expected to be broken/decoupled later
                    Vec::new()
                }
            };

            // RDP
            let reduced_points = math::simplify_rdp(&points, 1.5); // Default tolerance
            info!("RDP reduced points to {}", reduced_points.len());

            // TSP
            let ordered_points = math::solve_tsp_nearest_neighbor(reduced_points);

            // FFT
            let ast = math::perform_fft(&ordered_points, args.terms)?;
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
