use clap::Parser;
use gps_trajectory_validation::{cli, journeys::Journey, validate_journey};

fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();
    let file = std::fs::read_to_string(args.file_path)?;
    let journey: Option<Journey> = serde_json::from_str(&file)?;

    let res = validate_journey(journey)?;
    println!("{:?}", res);

    // validate_traces(
    //     journey.gps_trace.get(0).unwrap().clone(),
    //     journey.gps_trace.get(1).unwrap().clone(),
    // )?;

    Ok(())
}
