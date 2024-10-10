#![warn(clippy::all, clippy::pedantic)]

use clap::Parser;

mod cli;
mod constants;
mod error;
mod html;
mod plot;
mod types;
mod utils;

pub use error::Result;

pub fn main() -> Result<()> {
    run().map_err(|e| wrap!(e))?;
    Ok(())
}

fn run() -> Result<()> {
    // Parse the command line arguments
    let options = cli::Options::parse();

    // Get the current date and time
    let date_time = utils::get_date_time();

    let plots_folder = setup(&date_time)?;

    // Create the benchmarks run folder path and canonicalize it for the current execution
    let benchmarks_run_folder = std::fs::canonicalize(
        options
            .benchmarks_folder
            .join(constants::BENCHMARKS_RUN_FOLDER),
    )?;

    // Get the latest benchmarks
    let (current_benchmarks_path, previous_benchmarks_path) =
        utils::get_latest_benchmarks(&benchmarks_run_folder).map_err(|e| wrap!(e))?;

    // Deserialize the benchmarks
    let previous_benchmarks =
        utils::parse_json_benchmarks(&previous_benchmarks_path).map_err(|e| wrap!(e))?;

    let current_benchmarks =
        utils::parse_json_benchmarks(&current_benchmarks_path).map_err(|e| wrap!(e))?;

    generate_html(
        &previous_benchmarks,
        &current_benchmarks,
        &previous_benchmarks_path,
        &current_benchmarks_path,
        &options,
    )?;

    generate_plots(&previous_benchmarks, &current_benchmarks, &plots_folder)?;

    copy_latest_benchmarks(
        &date_time,
        &previous_benchmarks_path,
        &current_benchmarks_path,
    )
    .map_err(|e| wrap!(e))?;

    Ok(())
}

/// Setting up the site data folder and sub folders for the current execution
fn setup(date_time: &str) -> Result<std::path::PathBuf> {
    // Construct the site data folder path for the current execution
    let data_folder = std::path::Path::new(constants::SITE_FOLDER)
        .join(constants::SITE_DATA_FOLDER)
        .join(date_time);

    // Construct the site plots folder path for the current execution
    let plots_folder = data_folder.join(constants::SITE_PLOTS_FOLDER);

    // Create the flamegraphs folder for the current execution
    let flamegraphs_folder = data_folder.join(constants::SITE_FLAMEGRAPHS_FOLDER);

    // Check if folders exist and create them if they don't along the parent directories
    if !plots_folder.exists() {
        std::fs::create_dir_all(&plots_folder).map_err(|e| wrap!(e.into()))?;
    }

    if !flamegraphs_folder.exists() {
        std::fs::create_dir_all(&flamegraphs_folder).map_err(|e| wrap!(e.into()))?;
    }

    Ok(plots_folder)
}

fn generate_html(
    previous_benchmarks: &types::Benchmarks,
    current_benchmarks: &types::Benchmarks,
    previous_benchmarks_filename: &std::path::Path,
    current_benchmarks_filename: &std::path::Path,
    options: &cli::Options,
) -> Result<()> {
    // Generate the HTML for the index page
    let index_html = html::generate(
        previous_benchmarks,
        current_benchmarks,
        previous_benchmarks_filename.display().to_string().as_str(),
        current_benchmarks_filename.display().to_string().as_str(),
        &utils::get_latest_stats_collection(&options.benchmarks_folder).map_err(|e| wrap!(e))?,
        options,
    )
    .map_err(|e| wrap!(e))?;

    // Generate the HTML for the error page
    let error_html = html::generate_error_page();

    // Write the index HTML to a file
    let index_html_file_path =
        std::path::Path::new(constants::SITE_FOLDER).join(constants::INDEX_FILENAME);
    utils::write_data_to_file(&index_html, &index_html_file_path).map_err(|e| wrap!(e))?;

    // Write the error HTML to a file
    let error_html_file_path =
        std::path::Path::new(constants::SITE_FOLDER).join(constants::ERROR_FILENAME);
    utils::write_data_to_file(&error_html, &error_html_file_path).map_err(|e| wrap!(e))?;
    Ok(())
}

fn generate_plots(
    previous_benchmarks: &types::Benchmarks,
    current_benchmarks: &types::Benchmarks,
    plots_folder: &std::path::Path,
) -> Result<()> {
    // Generate plots
    plot::generate_plots(
        previous_benchmarks,
        plots_folder.join("previous").display().to_string().as_str(),
    )
    .map_err(|e| wrap!(e))?;

    plot::generate_plots(
        current_benchmarks,
        plots_folder.join("current").display().to_string().as_str(),
    )
    .map_err(|e| wrap!(e))?;
    Ok(())
}

fn copy_latest_benchmarks(
    date_time: &str,
    previous_benchmarks_path: &std::path::Path,
    current_benchmarks_path: &std::path::Path,
) -> Result<()> {
    let site_benchmarks_run_folder = std::path::Path::new(constants::SITE_FOLDER)
        .join(constants::SITE_DATA_FOLDER)
        .join(date_time)
        .join(constants::SITE_BENCHMARKS_RUNS_FOLDER);

    if !site_benchmarks_run_folder.exists() {
        std::fs::create_dir_all(&site_benchmarks_run_folder).map_err(|e| wrap!(e.into()))?;
    }

    std::fs::copy(
        previous_benchmarks_path,
        site_benchmarks_run_folder.join(previous_benchmarks_path.file_name().unwrap()),
    )
    .map_err(|e| wrap!(e.into()))?;

    std::fs::copy(
        current_benchmarks_path,
        site_benchmarks_run_folder.join(current_benchmarks_path.file_name().unwrap()),
    )
    .map_err(|e| wrap!(e.into()))?;
    Ok(())
}
