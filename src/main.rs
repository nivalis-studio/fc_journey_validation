use std::path::PathBuf;

use clap::Parser;
use gps_trajectory_validation::{cli, journeys::Journey, validate_journey};

fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();

    let journey = Journey::try_from(PathBuf::from(args.file_path))?;

    let res = validate_journey(journey)?;
    println!("{:?}", res);

    // validate_traces(
    //     journey.gps_trace.get(0).unwrap().clone(),
    //     journey.gps_trace.get(1).unwrap().clone(),
    // )?;

    Ok(())
}
