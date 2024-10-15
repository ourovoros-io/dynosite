use std::{
    collections::HashSet,
    io::Read,
    path::{Path, PathBuf},
};

use crate::{cli::Options, error::Result, types::Benchmarks, wrap};
use serde::{Deserialize, Serialize};

/// Represents the execution structure for the site
#[derive(Serialize, Deserialize, Clone)]
pub struct Execution {
    pub previous_benchmarks: PathBuf,
    pub current_benchmarks: PathBuf,
    pub github_information: crate::types::PRInformation,
    pub root_folder: PathBuf,
    pub runs_folder: PathBuf,
    pub stats_folder: PathBuf,
    pub plots_folder: PathBuf,
    pub flamegraphs_folder: PathBuf,
    pub runs: Vec<PathBuf>,
    pub stats: Vec<PathBuf>,
    pub plots: Vec<PathBuf>,
    pub flamegraphs: Vec<PathBuf>,
}

impl Execution {
    /// Create a new execution
    pub fn new(root_folder: &Path, options: &Options) -> Result<Self> {
        // Create the necessary folder structures
        let (root_folder, runs_folder, stats_folder, plots_folder, flamegraphs_folder) =
            Self::create_structures(root_folder, options).map_err(|e| wrap!(e))?;

        // Update the runs, stats, and flamegraphs folders with the latest entries
        let runs = Self::get_latest_entries(&options.benchmarks_folder.join("runs"), 2)
            .map_err(|e| wrap!(e))?;

        for run in &runs {
            let run_file_name = run.file_name().unwrap().to_str().unwrap();
            let run_file_path = runs_folder.join(run_file_name);
            std::fs::copy(run, &run_file_path).map_err(|e| wrap!(e.into()))?;
        }

        let stats = Self::get_latest_entries(&options.benchmarks_folder.join("stats"), 1)
            .map_err(|e| wrap!(e))?;

        for stat in &stats {
            let stat_file_name = stat.file_name().unwrap().to_str().unwrap();
            let stat_file_path = stats_folder.join(stat_file_name);
            std::fs::copy(stat, &stat_file_path).map_err(|e| wrap!(e.into()))?;
        }

        let flamegraphs =
            Self::get_latest_entries(&options.benchmarks_folder.join("flamegraphs"), 2)
                .map_err(|e| wrap!(e))?;
        for flamegraph_folder in &flamegraphs {
            copy_dir_all(
                flamegraph_folder,
                &flamegraphs_folder.join(flamegraph_folder.file_name().unwrap().to_str().unwrap()),
            )
            .map_err(|e| wrap!(e))?;
        }

        // Get the latest benchmarks
        let (current_benchmarks_path, previous_benchmarks_path) =
            get_latest_benchmarks(&runs_folder).map_err(|e| wrap!(e))?;

        // Deserialize the benchmarks
        let previous_benchmarks =
            parse_json_benchmarks(&previous_benchmarks_path).map_err(|e| wrap!(e))?;

        let current_benchmarks =
            parse_json_benchmarks(&current_benchmarks_path).map_err(|e| wrap!(e))?;

        // Generate the plots
        let plots = Self::generate_plots(&previous_benchmarks, &current_benchmarks, &plots_folder)
            .map_err(|e| wrap!(e))?;

        // Create the github information structure
        let github_information = crate::types::PRInformation {
            hash: options
                .pr_hash
                .clone()
                .unwrap_or("dyno local execution".to_string()),
            title: options
                .pr_title
                .clone()
                .unwrap_or("dyno local execution".to_string()),
            link: options
                .pr_link
                .clone()
                .unwrap_or("dyno local execution".to_string()),
        };

        Ok(Self {
            previous_benchmarks: previous_benchmarks_path,
            current_benchmarks: current_benchmarks_path,
            github_information,
            root_folder,
            runs_folder,
            stats_folder,
            plots_folder,
            flamegraphs_folder,
            runs,
            stats,
            plots,
            flamegraphs,
        })
    }

    /// Create the necessary folder structures for the current execution
    fn create_structures(
        root_folder: &Path,
        options: &Options,
    ) -> Result<(PathBuf, PathBuf, PathBuf, PathBuf, PathBuf)> {
        // Create the root folder for the current execution and its sub folders
        let root_folder = root_folder
            .join(Self::get_current_execution_identifier(&options.benchmarks_folder).unwrap());

        if !root_folder.exists() {
            std::fs::create_dir(&root_folder).map_err(|e| wrap!(e.into()))?;
        }

        let runs_folder = root_folder.join("runs");
        if !runs_folder.exists() {
            std::fs::create_dir(&runs_folder).map_err(|e| wrap!(e.into()))?;
        }

        let stats_folder = root_folder.join("stats");
        if !stats_folder.exists() {
            std::fs::create_dir(&stats_folder).map_err(|e| wrap!(e.into()))?;
        }

        let plots_folder = root_folder.join("plots");
        if !plots_folder.exists() {
            std::fs::create_dir(&plots_folder).map_err(|e| wrap!(e.into()))?;
        }

        let flamegraphs_folder = root_folder.join("flamegraphs");
        if !flamegraphs_folder.exists() {
            std::fs::create_dir(&flamegraphs_folder).map_err(|e| wrap!(e.into()))?;
        }
        Ok((
            root_folder,
            runs_folder,
            stats_folder,
            plots_folder,
            flamegraphs_folder,
        ))
    }

    /// Get the current execution identifier from the latest stats file
    fn get_current_execution_identifier(benchmarks_folder: &Path) -> Result<PathBuf> {
        let benchmarks_stats_path = benchmarks_folder.join("stats");
        let latest_stats_file_path =
            get_latest_stats_file(&benchmarks_stats_path).map_err(|e| wrap!(e))?;

        // Keep the last part of the path
        let stats_file_last_part = latest_stats_file_path.components().last().unwrap();
        let current_execution_identifier = PathBuf::from(stats_file_last_part.as_os_str());

        // Remove the file extension from the path and keep the folder path
        let current_execution_identifier = current_execution_identifier
            .file_stem()
            .ok_or_else(|| wrap!("Failed to remove file stem".into()))?;

        let current_execution_identifier =
            current_execution_identifier.to_str().ok_or_else(|| {
                wrap!("Failed to convert current execution identifier to string".into())
            })?;
        Ok(PathBuf::from(current_execution_identifier))
    }

    /// Sync directories from source to target and all their contents
    /// Returns a list of paths regarding the final files and directories in source
    fn sync_directories(source: &Path, target: &Path) -> Result<Vec<PathBuf>> {
        // Ensure the target directory exists
        if !target.exists() {
            std::fs::create_dir_all(target)?;
        }

        // Get the list of entries in the source and target directories
        let source_entries: HashSet<PathBuf> = std::fs::read_dir(source)?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .collect();
        let target_entries: HashSet<PathBuf> = std::fs::read_dir(target)?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .collect();

        // Sync files and directories from source to target
        for source_path in &source_entries {
            let target_path = target.join(source_path.file_name().unwrap());

            if source_path.is_dir() {
                // Recursively sync directories
                Self::sync_directories(source_path, &target_path)?;
            } else {
                // Copy or update files
                if !target_path.exists() || Self::is_file_modified(source_path, &target_path)? {
                    std::fs::copy(source_path, target_path)?;
                }
            }
        }

        // Remove files and directories from target that are not in source
        for target_path in &target_entries {
            let source_path = source.join(target_path.file_name().unwrap());

            if !source_path.exists() {
                if target_path.is_dir() {
                    std::fs::remove_dir_all(target_path)?;
                } else {
                    std::fs::remove_file(target_path)?;
                }
            }
        }

        // Return the list of items in the source folder
        Ok(source_entries.into_iter().collect())
    }

    /// Check if a file is modified
    fn is_file_modified(source: &Path, target: &Path) -> Result<bool> {
        let source_metadata = std::fs::metadata(source)?;
        let target_metadata = std::fs::metadata(target)?;

        let source_modified = source_metadata.modified()?;
        let target_modified = target_metadata.modified()?;

        Ok(source_modified > target_modified)
    }

    /// Get the latest two files or folders from a folder
    fn get_latest_entries(target: &Path, count: usize) -> Result<Vec<PathBuf>> {
        let mut entries: Vec<_> = std::fs::read_dir(target)
            .map_err(|e| wrap!(e.into()))?
            .filter_map(std::result::Result::ok)
            .collect();

        // Sort entries by modification time in descending order
        entries.sort_by_key(|entry| {
            entry
                .metadata()
                .and_then(|meta| meta.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });
        entries.reverse();

        // Take the latest `count` entries
        let latest_entries: Vec<PathBuf> = entries
            .into_iter()
            .take(count)
            .map(|entry| entry.path())
            .collect();
        Ok(latest_entries)
    }

    /// Generate plots for the previous and current benchmarks
    fn generate_plots(
        previous_benchmarks: &Benchmarks,
        current_benchmarks: &Benchmarks,
        plots_folder: &Path,
    ) -> Result<Vec<PathBuf>> {
        let mut plots = Vec::new();
        // Generate plots
        plots.extend(
            super::plot::generate_plots(
                previous_benchmarks,
                plots_folder.join("previous").display().to_string().as_str(),
            )
            .map_err(|e| wrap!(e))?,
        );

        plots.extend(
            super::plot::generate_plots(
                current_benchmarks,
                plots_folder.join("current").display().to_string().as_str(),
            )
            .map_err(|e| wrap!(e))?,
        );
        Ok(plots)
    }
}

/// Get the latest two benchmark files in the folder
/// First item in the tuple is the previous benchmark file
/// Second item in the tuple is the current benchmark file
fn get_latest_benchmarks(folder: &Path) -> Result<(PathBuf, PathBuf)> {
    let mut entries: Vec<_> = std::fs::read_dir(folder)
        .map_err(|e| wrap!(e.into()))?
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() && path.file_name()?.to_str()?.ends_with("json") {
                let metadata = std::fs::metadata(&path).ok()?;
                let modified = metadata.modified().ok()?;
                Some((path, modified))
            } else {
                None
            }
        })
        .collect();

    // Sort entries by modification time in descending order
    entries.sort_by_key(|&(_, modified)| std::cmp::Reverse(modified));

    if entries.len() < 2 {
        return Err("Not enough files in the folder".into());
    }

    Ok((entries[0].0.clone(), entries[1].0.clone()))
}

/// Parse the JSON benchmarks from a file
fn parse_json_benchmarks(file_path: &Path) -> Result<Benchmarks> {
    let mut file = std::fs::File::open(file_path).map_err(|e| wrap!(e.into()))?;
    let mut data = String::new();
    file.read_to_string(&mut data)
        .map_err(|e| wrap!(e.into()))?;
    let benchmark_data: Benchmarks = serde_json::from_str(&data).map_err(|e| wrap!(e.into()))?;
    Ok(benchmark_data)
}

/// Get the latest stats file in the folder
pub fn get_latest_stats_file(folder: &Path) -> Result<PathBuf> {
    let mut entries: Vec<_> = std::fs::read_dir(folder)
        .map_err(|e| wrap!(e.into()))?
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() && path.file_name()?.to_str()?.ends_with("json") {
                let metadata = entry.metadata().ok()?;
                let modified = metadata.modified().ok()?;
                Some((path, modified))
            } else {
                None
            }
        })
        .collect();

    // Sort entries by modification time in descending order
    entries.sort_by_key(|&(_, modified)| std::cmp::Reverse(modified));
    if let Some((latest_path, _)) = entries.first() {
        Ok(latest_path.clone())
    } else {
        Err("No stats files found in the folder".into())
    }
}

/// Copy the contents of a directory to another directory
pub fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}
