use clap::Parser;
use gps_trajectory_validation::{cli::Cli, journeys::Journey, validate_journey};
use std::io::{self, Read};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let journey = match cli.file_path {
        Some(path) => Journey::try_from(path),
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Journey::try_from(buffer)
        }
    }?;

    let res = validate_journey(journey)?;
    println!("{:?}", res);

    Ok(())
}
