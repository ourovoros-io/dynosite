use crate::error::Result;
use crate::{
    types::{Benchmark, BenchmarkFrame, Benchmarks},
    wrap,
};
use plotters::prelude::*;

const LABEL_FONT_SIZE: u32 = 30;
const X_AXIS_LABEL_FONT_SIZE: u32 = 8;
const PLOT_WIDTH: u32 = 1200;
const PLOT_HEIGHT: u32 = 400;

pub fn generate_plots(benchmarks: &Benchmarks, output_file: &str) -> Result<()> {
    for benchmark in &benchmarks.benchmarks {
        let metrics: [(&str, &str, fn(&BenchmarkFrame) -> i64); 7] = [
            (
                "cpu_usage",
                "CPU Usage Over Time",
                |frame: &BenchmarkFrame| crate::utils::safe_f32_to_i64(frame.cpu_usage),
            ),
            (
                "memory_usage",
                "Memory Usage Over Time",
                |frame: &BenchmarkFrame| frame.memory_usage.try_into().unwrap(),
            ),
            (
                "virtual_memory_usage",
                "Virtual Memory Usage Over Time",
                |frame: &BenchmarkFrame| frame.virtual_memory_usage.try_into().unwrap(),
            ),
            (
                "disk_total_written_bytes",
                "Disk Total Written Bytes Over Time",
                |frame: &BenchmarkFrame| frame.disk_total_written_bytes.try_into().unwrap(),
            ),
            (
                "disk_written_bytes",
                "Disk Written Bytes Over Time",
                |frame: &BenchmarkFrame| frame.disk_written_bytes.try_into().unwrap(),
            ),
            (
                "disk_total_read_bytes",
                "Disk Total Read Bytes Over Time",
                |frame: &BenchmarkFrame| frame.disk_total_read_bytes.try_into().unwrap(),
            ),
            (
                "disk_read_bytes",
                "Disk Read Bytes Over Time",
                |frame: &BenchmarkFrame| frame.disk_read_bytes.try_into().unwrap(),
            ),
        ];

        let frames = benchmark.frames.lock().unwrap().clone(); // Clone the frames to avoid holding the lock

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
                        i64::try_from(frame.relative_timestamp.as_millis()).unwrap(),
                        value_extractor(frame),
                    )
                },
            )?;
        }
    }

    Ok(())
}

fn generate<F, G>(
    frames: &[BenchmarkFrame],
    benchmark: &Benchmark,
    output_file: &str,
    title: &str,
    y_max_calculator: F,
    data_extractor: G,
) -> Result<()>
where
    F: Fn(&[BenchmarkFrame]) -> Result<i64>,
    G: Fn(&BenchmarkFrame) -> (i64, i64),
{
    let y_max = y_max_calculator(frames)?;

    create(benchmark, output_file, title, y_max, data_extractor)
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

    let frames = benchmark.frames.lock().unwrap().clone(); // Clone the frames to avoid holding the lock
    let phases = benchmark.phases.clone(); // Clone the phases to avoid holding the lock

    let start_time =
        i64::try_from(benchmark.start_time.unwrap().as_millis()).map_err(|e| wrap!(e.into()))?;
    let end_time =
        i64::try_from(benchmark.end_time.unwrap().as_millis()).map_err(|e| wrap!(e.into()))?;
    let total_duration = end_time - start_time;

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", LABEL_FONT_SIZE).into_font())
        .margin(10)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(0..total_duration, 0..y_max)
        .map_err(|e| wrap!(e.into()))?;

    chart
        .configure_mesh()
        .x_labels(20)
        .y_labels(10)
        .x_label_formatter(&|x| {
            let phase = phases.iter().find(|p| {
                i64::try_from(p.start_time.unwrap().as_millis()).unwrap() <= *x
                    && i64::try_from(p.end_time.unwrap().as_millis()).unwrap() >= *x
            });
            if let Some(phase) = phase {
                phase.name.to_string()
            } else {
                format!("{x}")
            }
        })
        .label_style(("sans-serif", X_AXIS_LABEL_FONT_SIZE).into_font())
        .draw()
        .map_err(|e| wrap!(e.into()))?;

    chart
        .draw_series(LineSeries::new(frames.iter().map(data_mapper), &RED))
        .map_err(|e| wrap!(e.into()))?;

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
