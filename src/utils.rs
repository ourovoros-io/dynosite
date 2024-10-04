use std::{
    io::Read,
    path::{Path, PathBuf},
};

use crate::{error::Result, types::Benchmarks, wrap};

pub fn get_latest_benchmarks(folder: &Path) -> Result<(PathBuf, PathBuf)> {
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

pub fn parse_json_benchmarks(file_path: &Path) -> Result<Benchmarks> {
    let mut file = std::fs::File::open(file_path).map_err(|e| wrap!(e.into()))?;
    let mut data = String::new();
    file.read_to_string(&mut data)
        .map_err(|e| wrap!(e.into()))?;
    let benchmark_data: Benchmarks = serde_json::from_str(&data).map_err(|e| wrap!(e.into()))?;
    Ok(benchmark_data)
}

pub fn write_data_to_file(data: &str, file_path: &Path) -> Result<()> {
    std::fs::write(file_path, data).map_err(|e| wrap!(e.into()))?;
    Ok(())
}

pub fn get_latest_stats_file(folder: &Path) -> Result<PathBuf> {
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

    if let Some((latest_path, _)) = entries.first() {
        Ok(latest_path.clone())
    } else {
        Err("No stats files found in the folder".into())
    }
}

/// Get the current date and time in the format "YYYY-MM-DD--HH:MM:SS"
#[must_use]
pub fn get_date_time() -> String {
    let datetime = chrono::Local::now();
    datetime.format("%Y-%m-%d_%H:%M:%S").to_string()
}

/// Get the stats collection from the latest stats file
pub fn get_stats_collection(
    options: &crate::cli::Options,
) -> crate::error::Result<crate::types::Collection> {
    let stats_collection = crate::utils::get_latest_stats_file(
        &options
            .benchmarks_folder
            .join(crate::constants::BENCHMARKS_STATS_FOLDER),
    )
    .map_err(|e| wrap!(e))?;

    let stats_collection =
        std::fs::read_to_string(stats_collection).map_err(|e| wrap!(e.into()))?;

    let stats_collection: crate::types::Collection =
        serde_json::from_str(&stats_collection).map_err(|e| wrap!(e.into()))?;

    Ok(stats_collection)
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_precision_loss)]
pub fn safe_f32_to_i64(value: f32) -> i64 {
    if value.is_finite() {
        if value >= i64::MIN as f32 && value <= i64::MAX as f32 {
            value as i64
        } else {
            panic!("Value is out of range")
        }
    } else {
        panic!("Value is not finite")
    }
}
