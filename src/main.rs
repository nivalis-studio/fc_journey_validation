use gps_trajectory_validation::{journeys::Journey, validate_journey};
use std::io::{self, Read};

fn main() -> anyhow::Result<()> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    let journey = Journey::try_from(buffer.as_str())?;

    let res = validate_journey(journey)?;
    println!("{:?}", res);

    Ok(())
}
