use clap::Parser;
use gps_trajectory_validation::{cli, journeys::Journey, validation};

fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();

    let file = std::fs::read_to_string(args.file_path)?;

    let journey: Journey = serde_json::from_str(&file)?;
    let (trace_1, trace_2) = journey.gps_trace;

    validation(trace_1, trace_2)?;

    Ok(())
}
