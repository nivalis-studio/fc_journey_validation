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
        // TODO: handle error conversion to output
        Ok(journey) => journey.validate().unwrap(),
        Err(err) => Output::from(err),
    };

    let output_json = serde_json::to_string(&output).unwrap();

    let mut stdout = io::stdout().lock();

    let writer: &mut dyn Write = &mut stdout;

    write!(writer, "{}", output_json).unwrap();
}
