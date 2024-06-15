use clap::Parser;
use gps_trajectory_validation::{cli::Cli, journeys::Journey, output::Output};
use std::io::{self, Write};

fn main() {
    let cli = Cli::parse();

    let journey_result = match cli.file_path {
        Some(path) => Journey::try_from(path),
        None => Journey::from_stdin(),
    };

    let output = match journey_result {
        Ok(journey) => match journey.validate() {
            Ok(output) => output,
            Err(err) => Output::from(err),
        },
        Err(err) => Output::from(err),
    };

    let output_json = serde_json::to_string(&output).unwrap();

    let mut stdout = io::stdout();

    write!(stdout, "{}", output_json).unwrap();
}
