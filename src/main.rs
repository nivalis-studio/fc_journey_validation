use clap::Parser;
use gps_trajectory_validation::{cli::Cli, input::JourneyInput, journey::Journey, output::Output};
use std::io::{self, Write};

fn main() {
    let cli = Cli::parse();

    let journey_result = match cli.file_path {
        Some(path) => JourneyInput::try_from(path),
        None => JourneyInput::from_stdin(),
    }
    .and_then(Journey::try_from);

    let output = match journey_result {
        Ok(journey) => journey.validate(),
        Err(err) => Output::from(err),
    };

    let output_json = serde_json::to_string(&output).unwrap();

    let mut stdout = io::stdout();

    write!(stdout, "{}", output_json).unwrap();
}
