#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_lines)]

use clap::Parser;

mod cli;
mod error;
mod site;
mod types;

pub use error::Result;
use site::execution::Execution;

pub fn main() -> Result<()> {
    init().map_err(|e| wrap!(e))?;
    Ok(())
}

/// Initialize the site
fn init() -> Result<()> {
    // Parse the command line arguments
    let options = cli::Options::parse();

    // Initialize the site
    let mut site =
        site::dynosite::DynoSite::init(&options.site_name.clone().unwrap_or("site".to_string()))
            .map_err(|e| wrap!(e))?;

    // Add the execution to the site
    site.add_execution(
        &Execution::new(&site.data.root_folder.clone(), &options).map_err(|e| wrap!(e))?,
        options.data_only,
    )
    .map_err(|e| wrap!(e))?;

    // Store the site locally
    site.store().map_err(|e| wrap!(e))?;

    Ok(())
}
