use std::path::PathBuf;

use crate::error::Result;
use crate::{
    types::{Benchmark, BenchmarkFrame, Benchmarks},
    wrap,
};
use plotters::prelude::*;

const LABEL_FONT_SIZE: u32 = 30;
const X_AXIS_LABEL_FONT_SIZE: u32 = 10;
const PLOT_WIDTH: u32 = 1200;
const PLOT_HEIGHT: u32 = 400;

pub fn generate_plots(benchmarks: &Benchmarks, output_file: &str) -> Result<Vec<PathBuf>> {
    let mut plots = Vec::new();
    for benchmark in &benchmarks.benchmarks {
        let metrics: [(&str, &str, fn(&BenchmarkFrame) -> i64); 7] = [
            (
                "cpu_usage",
                "CPU Usage Over Time",
                |frame: &BenchmarkFrame| safe_f32_to_i64(frame.cpu_usage),
            ),
            (
                "memory_usage",
                "Memory Usage Over Time",
                |frame: &BenchmarkFrame| {
                    frame
                        .memory_usage
                        .try_into()
                        .expect("Failed to convert memory usage to i64")
                },
            ),
            (
                "virtual_memory_usage",
                "Virtual Memory Usage Over Time",
                |frame: &BenchmarkFrame| {
                    frame
                        .virtual_memory_usage
                        .try_into()
                        .expect("Failed to convert virtual memory usage to i64")
                },
            ),
            (
                "disk_total_written_bytes",
                "Disk Total Written Bytes Over Time",
                |frame: &BenchmarkFrame| {
                    frame
                        .disk_total_written_bytes
                        .try_into()
                        .expect("Failed to convert disk_total_written_bytes usage to i64")
                },
            ),
            (
                "disk_written_bytes",
                "Disk Written Bytes Over Time",
                |frame: &BenchmarkFrame| {
                    frame
                        .disk_written_bytes
                        .try_into()
                        .expect("Failed to convert disk_written_bytes usage to i64")
                },
            ),
            (
                "disk_total_read_bytes",
                "Disk Total Read Bytes Over Time",
                |frame: &BenchmarkFrame| {
                    frame
                        .disk_total_read_bytes
                        .try_into()
                        .expect("Failed to convert disk_total_read_bytes usage to i64")
                },
            ),
            (
                "disk_read_bytes",
                "Disk Read Bytes Over Time",
                |frame: &BenchmarkFrame| {
                    frame
                        .disk_read_bytes
                        .try_into()
                        .expect("Failed to convert disk_read_bytes usage to i64")
                },
            ),
        ];

        let frames = benchmark
            .frames
            .lock()
            .expect("Failed to get frames lock")
            .clone();

        for (suffix, title, value_extractor) in &metrics {
            let output_file = format!("{output_file}_{}_{}.png", benchmark.name, suffix);

            let y_max_calculator: Box<dyn Fn(&[BenchmarkFrame]) -> Result<i64>> =
                if *suffix == "cpu_usage" {
                    Box::new(|_: &[BenchmarkFrame]| Ok(100))
                } else {
                    Box::new(|frames: &[BenchmarkFrame]| calculate_y_max(frames, *value_extractor))
                };

            generate(
                &frames,
                benchmark,
                &output_file,
                title,
                y_max_calculator,
                |frame| {
                    (
                        i64::try_from(frame.relative_timestamp.as_millis())
                            .expect("Failed to convert relative timestamp to i64"),
                        value_extractor(frame),
                    )
                },
            )?;
            plots.push(PathBuf::from(output_file));
        }
    }

    Ok(plots)
}

fn generate<F, G>(
    frames: &[BenchmarkFrame],
    benchmark: &Benchmark,
    output_file: &str,
    title: &str,
    y_max_calculator: F,
    data_mapper: G,
) -> Result<()>
where
    F: Fn(&[BenchmarkFrame]) -> Result<i64>,
    G: Fn(&BenchmarkFrame) -> (i64, i64),
{
    let y_max = y_max_calculator(frames)?;

    create(benchmark, output_file, title, y_max, data_mapper)
}

pub fn create<F>(
    benchmark: &Benchmark,
    output_file: &str,
    title: &str,
    y_max: i64,
    data_mapper: F,
) -> Result<()>
where
    F: Fn(&BenchmarkFrame) -> (i64, i64),
{
    let root = BitMapBackend::new(output_file, (PLOT_WIDTH, PLOT_HEIGHT)).into_drawing_area();
    root.fill(&WHITE)?;

    // Split the drawing area into two: one for the chart and one for the legend
    let (upper, lower) = root.split_vertically(PLOT_HEIGHT - 20);

    let frames = benchmark
        .frames
        .lock()
        .expect("Failed to get benchmarking lock")
        .clone();
    let phases = benchmark.phases.clone();

    let start_time = i64::try_from(
        benchmark
            .start_time
            .ok_or_else(|| wrap!("Failed to get start time from benchmark".into()))?
            .as_millis(),
    )
    .map_err(|e| wrap!(e.into()))?;
    let end_time = i64::try_from(
        benchmark
            .end_time
            .ok_or_else(|| wrap!("Failed to get end time from benchmark".into()))?
            .as_millis(),
    )
    .map_err(|e| wrap!(e.into()))?;
    let total_duration = end_time - start_time;

    // Find the minimum start time among all phases
    let min_start_time = phases
        .iter()
        .map(|phase| {
            i64::try_from(
                phase
                    .start_time
                    .expect("Failed to get start time from phase")
                    .as_millis(),
            )
            .expect("Failed to convert start time to i64")
        })
        .min()
        .ok_or_else(|| wrap!("Failed to get min start time".into()))?;

    // Normalize the start and end times of each phase
    let normalized_phases: Vec<_> = phases
        .iter()
        .map(|phase| {
            let start_time = i64::try_from(
                phase
                    .start_time
                    .expect("Failed to get phase start time for normalized phases")
                    .as_millis(),
            )
            .expect("Failed to convert start time to i64")
                - min_start_time;
            let end_time = i64::try_from(
                phase
                    .end_time
                    .expect("Failed to get phase end time for normalized phases")
                    .as_millis(),
            )
            .expect("Failed to convert end time to i64")
                - min_start_time;
            (start_time, end_time, phase.name.clone())
        })
        .collect();

    let mut chart = ChartBuilder::on(&upper)
        .caption(
            title,
            ("sans-serif", LABEL_FONT_SIZE).into_font().color(&BLACK),
        ) // Set title color to black
        .margin(10)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(0..total_duration, 0..y_max)
        .map_err(|e| wrap!(e.into()))?;

    chart
        .configure_mesh()
        .x_labels(normalized_phases.len())
        .y_labels(10)
        .x_label_formatter(&|x| {
            let is_between = |x: &i64, start: &i64, end: &i64| -> bool { x >= start && x <= end };

            let phase = normalized_phases
                .iter()
                .find(|(start_time, end_time, _)| is_between(x, start_time, &(end_time + 100)));

            if let Some((_, _, name)) = phase {
                format!(
                    "{}-{}",
                    x,
                    name.to_string()
                        .split_whitespace()
                        .next()
                        .expect("Failed to get phase name for x label")
                )
            } else {
                format!("{x}")
            }
        })
        .label_style(
            ("sans-serif", X_AXIS_LABEL_FONT_SIZE)
                .into_font()
                .color(&BLACK),
        )
        .draw()
        .map_err(|e| wrap!(e.into()))?;

    let opacity = 0.5;

    let colors = [
        ("compile to ast", RGBAColor(255, 0, 0, opacity)),
        (
            "parse the program to a concrete syntax tree (CST)",
            RGBAColor(0, 255, 0, opacity),
        ),
        (
            "parse the concrete syntax tree (CST) to a typed AST",
            RGBAColor(0, 0, 255, opacity),
        ),
        ("compile ast to asm", RGBAColor(255, 165, 0, opacity)),
        ("generate JSON ABI program", RGBAColor(255, 0, 255, opacity)),
        ("compile asm to bytecode", RGBAColor(0, 255, 255, opacity)),
    ];

    // Draw phase overlays
    for (start_time, end_time, name) in &normalized_phases {
        let color = colors
            .iter()
            .find(|(color_name, _)| name.contains(color_name))
            .map(|(_, color)| color)
            .ok_or_else(|| wrap!("Failed to construct a color.".into()))?;

        chart
            .draw_series(std::iter::once(Rectangle::new(
                [(*start_time, 0), (*end_time, y_max)],
                color.filled(),
            )))
            .map_err(|e| wrap!(e.into()))?;
    }

    chart
        .draw_series(LineSeries::new(frames.iter().map(data_mapper), &RED))
        .map_err(|e| wrap!(e.into()))?;

    // Draw legend at the bottom
    let legend_font = ("sans-serif", 10).into_font().color(&BLACK);

    let legend_spacing = 210;
    let legend_items: Vec<_> = colors
        .iter()
        .enumerate()
        .map(|(i, (name, color))| {
            // Draw the colored circle
            let circle = Circle::new((10 + i as i32 * legend_spacing, 10), 5, color.filled());

            // Draw the text next to the circle
            let text = Text::new(
                (*name).to_string(),
                (20 + i as i32 * legend_spacing, 10),
                legend_font.clone(),
            );

            (circle, text)
        })
        .collect();

    for (circle, text) in legend_items {
        lower.draw(&circle).map_err(|e| wrap!(e.into()))?;
        lower.draw(&text).map_err(|e| wrap!(e.into()))?;
    }

    root.present()?;
    Ok(())
}

fn calculate_y_max(
    frames: &[BenchmarkFrame],
    value_extractor: fn(&BenchmarkFrame) -> i64,
) -> Result<i64> {
    let max_value = frames.iter().map(value_extractor).max().ok_or(wrap!(
        "Failed to get max_value while calculating y_max".into()
    ))?;
    let offset = max_value / 10;
    let y_max = max_value + offset;
    Ok(if y_max == 0 { 100 } else { y_max })
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_precision_loss)]
fn safe_f32_to_i64(value: f32) -> i64 {
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
