use clap::Parser;
use gps_trajectory_validation::{cli::Cli, journeys::Journey, output::Output};
use std::io::{self, Write};

fn main() {
    let cli = Cli::parse();

    let journey_result = match cli.file_path {
        Some(path) => Journey::try_from(path),
        None => Journey::from_stin(),
    };

    let output = match journey_result {
        Ok(_) => Output::success(),
        Err(err) => Output::from(err),
    };

    let output_json = serde_json::to_string(&output).unwrap();

    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();

    let writer: &mut dyn Write = if output.success {
        &mut stdout
    } else {
        &mut stderr
    };

    write!(writer, "{}", output_json).unwrap();
}
