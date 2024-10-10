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
    let (upper, lower) = root.split_vertically(PLOT_HEIGHT - 20); // Adjust the height of the legend area

    let frames = benchmark.frames.lock().unwrap().clone(); // Clone the frames to avoid holding the lock
    let phases = benchmark.phases.clone(); // Clone the phases to avoid holding the lock

    let start_time =
        i64::try_from(benchmark.start_time.unwrap().as_millis()).map_err(|e| wrap!(e.into()))?;
    let end_time =
        i64::try_from(benchmark.end_time.unwrap().as_millis()).map_err(|e| wrap!(e.into()))?;
    let total_duration = end_time - start_time;
    // println!("Total duration: {}", total_duration);
    // println!("Phases length: {}", phases.len());

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
        .x_labels(phases.len())
        .y_labels(10)
        .x_label_formatter(&|x| {
            let is_between = |x: i64, start: i64, end: i64| -> bool { x >= start && x <= end };

            let phase = phases.iter().find(|p| {
                // println!("Phase name : {}", p.name);
                // println!("x: {}, start_time: {}, end_time : {}", x, i64::try_from(p.start_time.unwrap().as_millis()).unwrap(), i64::try_from(p.end_time.unwrap().as_millis()).unwrap());
                is_between(
                    i64::try_from(p.start_time.unwrap().as_millis()).unwrap(),
                    *x,
                    x + 100,
                )
            });

            if let Some(phase) = phase {
                format!(
                    "{}-{}",
                    x,
                    phase.name.to_string().split_whitespace().next().unwrap()
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
        ("compile to ast", RGBAColor(255, 0, 0, opacity)), // Red
        (
            "parse the program to a concrete syntax tree (CST)",
            RGBAColor(0, 255, 0, opacity),
        ), // Green
        (
            "parse the concrete syntax tree (CST) to a typed AST",
            RGBAColor(0, 0, 255, opacity),
        ), // Blue
        ("compile ast to asm", RGBAColor(255, 165, 0, opacity)), // Orange
        ("generate JSON ABI program", RGBAColor(255, 0, 255, opacity)), // Magenta
        ("compile asm to bytecode", RGBAColor(0, 255, 255, opacity)), // Cyan
    ];

    // Draw phase overlays
    for phase in &phases {
        let phase_start = i64::try_from(phase.start_time.unwrap().as_millis()).unwrap();
        let phase_end = i64::try_from(phase.end_time.unwrap().as_millis()).unwrap();
        let color = colors
            .iter()
            .find(|(name, _)| phase.name.contains(name))
            .unwrap()
            .1;

        chart
            .draw_series(std::iter::once(Rectangle::new(
                [(phase_start, 0), (phase_end, y_max)],
                color.filled(),
            )))
            .map_err(|e| wrap!(e.into()))?;
    }

    chart
        .draw_series(LineSeries::new(frames.iter().map(data_mapper), &RED))
        .map_err(|e| wrap!(e.into()))?;

    // Draw legend at the bottom
    let legend_font = ("sans-serif", 10).into_font().color(&BLACK); // Smaller font size for the legend

    let legend_spacing = 210; // Adjust the spacing between legend items
    let legend_items: Vec<_> = colors
        .iter()
        .enumerate()
        .map(|(i, (name, color))| {
            // Draw the colored circle
            let circle = Circle::new(
                (10 + i as i32 * legend_spacing, 10), // Adjust the x position for each legend item
                5,                                    // Radius of the circle
                color.filled(),
            );

            // Draw the text next to the circle
            let text = Text::new(
                name.to_string(),
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
