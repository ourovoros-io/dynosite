use std::path::{Path, PathBuf};

use crate::site::dynosite::DynoSite;
use crate::types::Benchmarks;

use crate::error::Result;
use crate::wrap;

/// Generate the HTML for the site
pub fn generate(site: &DynoSite) -> Result<String> {
    let mut html = generate_header();

    // TODO : We might wanna have a check that makes sure the system settings are the same for all the benchmarks
    let system_settings_benchmarks = &serde_json::from_str(
        &std::fs::read_to_string(&site.data.executions[0].previous_benchmarks)
            .map_err(|e| wrap!(e.into()))?,
    )
    .map_err(|e| wrap!(e.into()))?;

    // Create the container
    html.push_str("<div class=\"container-fluid\">");

    // Title of the page
    html.push_str("<h1>Forc Performance Profiler</h1>");
    // Version of the page
    html.push_str(&format!("<p>Version: {}</p>", env!("CARGO_PKG_VERSION")));

    // Generate the system specifications
    html.push_str(&generate_system_specs(system_settings_benchmarks));

    // Add collapsible structure for each folder in the data directory
    html.push_str("<h5>Run Collection</h5>");
    html.push_str("<ul class=\"collapsible\">");

    // Get all the folders in the data directory
    let folders: Vec<_> = std::fs::read_dir(&site.data_folder)
        .map_err(|e| wrap!(e.into()))?
        .collect();

    // Sort folders by timestamp in descending order (newest first)
    let mut sorted_folders = folders;

    // TODO : refactor this
    sorted_folders.sort_by(|a, b| {
        let folder_name_a = a.as_ref().unwrap().file_name().into_string().unwrap();
        let folder_name_b = b.as_ref().unwrap().file_name().into_string().unwrap();

        let timestamp_a = parse_timestamp(&folder_name_a).unwrap();
        let timestamp_b = parse_timestamp(&folder_name_b).unwrap();

        timestamp_b.cmp(&timestamp_a)
    });

    // Iterate over the sorted folders
    for folder in sorted_folders {
        let folder = folder.map_err(|e| wrap!(e.into()))?;
        let folder_path = folder.path();

        // Get the current execution
        let current_execution = site
            .data
            .executions
            .iter()
            .find(|e| e.root_folder == folder_path)
            .ok_or_else(|| wrap!("No GitHub information found for the folder".into()))?;

        if folder_path.is_dir() {
            let folder_name = folder_path
                .file_name()
                .ok_or_else(
                    || wrap!("Failed to get the folder file name from folder path.".into()),
                )?
                .to_str()
                .ok_or_else(|| wrap!("Failed to get file name as str from folder path.".into()))?;

            let previous_benchmarks: Benchmarks = serde_json::from_str(
                &std::fs::read_to_string(&current_execution.previous_benchmarks)
                    .map_err(|e| wrap!(e.into()))?,
            )
            .map_err(|e| wrap!(e.into()))?;

            let current_benchmarks: Benchmarks = serde_json::from_str(
                &std::fs::read_to_string(&current_execution.current_benchmarks)
                    .map_err(|e| wrap!(e.into()))?,
            )
            .map_err(|e| wrap!(e.into()))?;

            html.push_str("<li>");
            html.push_str(&format!(
                "<div class=\"collapsible-header\"><i class=\"material-icons\">folder</i>{folder_name}</div>",
            ));
            html.push_str("<div class=\"collapsible-body\">");
            html.push_str("<div class=\"container-fluid\">");
            html.push_str(&format!(
                "<h5>PR Link : {}</h5>",
                current_execution.github_information.link
            ));
            html.push_str(&format!(
                "<h5>PR Title : {}</h5>",
                current_execution.github_information.title
            ));
            html.push_str(&format!(
                "<h5>PR Hash : {}</h5>",
                current_execution.github_information.hash
            ));
            html.push_str(
                &generate_previous_current_information(
                    &previous_benchmarks,
                    &current_benchmarks,
                    &current_execution.previous_benchmarks,
                    &current_execution.current_benchmarks,
                )
                .map_err(|e| wrap!(e))?,
            );

            // Add collapsible structure for the stats collection
            html.push_str("<ul class=\"collapsible\">");
            let stats_file_path = PathBuf::from(format!(
                "{}/{}.json",
                current_execution.stats_folder.display(),
                current_execution
                    .root_folder
                    .components()
                    .last()
                    .ok_or_else(|| wrap!(
                        "Failed to get the last component of the root folder path.".into()
                    ))?
                    .as_os_str()
                    .to_str()
                    .ok_or_else(|| wrap!(
                        "Failed to get the last component of the root folder path as str.".into()
                    ))?,
            ));
            let stats_file_string =
                &std::fs::read_to_string(&stats_file_path).map_err(|e| wrap!(e.into()))?;

            let stats_collection: crate::types::Collection =
                serde_json::from_str(stats_file_string).map_err(|e| wrap!(e.into()))?;

            for (file_name, stats) in &stats_collection.0 {
                if stats_file_path
                    .to_str()
                    .ok_or_else(|| wrap!("Failed to get the stats file path as str.".into()))?
                    .contains(folder_name)
                {
                    html.push_str("<li>");
                    html.push_str(&format!(
                    "<div class=\"collapsible-header\"><i class=\"material-icons\">insert_chart</i>{file_name}</div>"
                ));
                    html.push_str("<div class=\"collapsible-body\"><table class=\"striped\">");
                    html.push_str(
                    "<thead><tr><th>Metric</th><th>Regression(Red)/Improvement(Green)</th></tr></thead>",
                );
                    html.push_str("<tbody>");

                    let metrics = [
                        ("CPU Usage", stats.cpu_usage.1),
                        ("Memory Usage", stats.memory_usage.1),
                        ("Virtual Memory Usage", stats.virtual_memory_usage.1),
                        ("Disk Total Written Bytes", stats.disk_total_written_bytes.1),
                        ("Disk Written Bytes", stats.disk_written_bytes.1),
                        ("Disk Total Read Bytes", stats.disk_total_read_bytes.1),
                        ("Disk Read Bytes", stats.disk_read_bytes.1),
                        ("Bytecode Size", stats.bytecode_size.1),
                        ("Data Section Size", stats.data_section_size.1),
                        ("Time", stats.time.1),
                    ];

                    for (metric, value) in &metrics {
                        let color = if *value > 0.0 {
                            "red"
                        } else if *value < 0.0 {
                            "green"
                        } else {
                            "white"
                        };
                        html.push_str(&format!(
                        "<tr><td>{metric}</td><td style=\"color: {color}\">{value:.2}%</td></tr>"
                    ));
                    }

                    html.push_str("<table class=\"striped\">");
                    html.push_str("<thead><tr><th>Metric</th><th>Previous Benchmark</th><th>Current Benchmark</th></tr></thead>");
                    html.push_str("<tbody>");

                    html.push_str(&format!(
                        "<tr><td>Bytecode Size</td><td>{} bytes</td><td>{} bytes</td></tr>",
                        previous_benchmarks
                            .benchmarks
                            .iter()
                            .find(|b| file_name.contains(&b.name))
                            .ok_or_else(|| wrap!("Failed to find the benchmark in the previous benchmarks.".into()))?
                            .asm_information
                            .clone()
                            .ok_or_else(|| wrap!("Failed to get the asm information for bytecode size from previous benchmakrs.".into()))?["bytecode_size"],
                        current_benchmarks
                            .benchmarks
                            .iter()
                            .find(|b| file_name.contains(&b.name))
                            .ok_or_else(|| wrap!("Failed to find the benchmark in the current benchmarks.".into()))?
                            .asm_information
                            .clone()
                            .ok_or_else(|| wrap!("Failed to get the asm information for bytecode size from current benchmars.".into()))?["bytecode_size"]
                    ));

                    html.push_str(&format!(
                    "<tr><td>Data Section</td><td>Size : {} - Used : {}</td><td>Size : {} - Used : {}</td></tr>",
                    previous_benchmarks
                        .benchmarks
                        .iter()
                        .find(|b| file_name.contains(&b.name))
                        .ok_or_else(|| wrap!("Failed to find the benchmark in the previous benchmarks.".into()))?
                        .asm_information
                        .clone()
                        .ok_or_else(|| wrap!("Failed to get the asm information for data section size from the previous benchmarks.".into()))?["data_section"]["size"],
                    previous_benchmarks
                        .benchmarks
                        .iter()
                        .find(|b| file_name.contains(&b.name))
                        .ok_or_else(|| wrap!("Failed to find the benchmark in the previous benchmarks.".into()))?
                        .asm_information
                        .clone()
                        .ok_or_else(|| wrap!("Failed to get the asm information for data section used from the previous benchmarks.".into()))?["data_section"]["used"],
                    current_benchmarks
                        .benchmarks
                        .iter()
                        .find(|b| file_name.contains(&b.name))
                        .ok_or_else(|| wrap!("Failed to find the benchmark in the current benchmarks.".into()))?
                        .asm_information
                        .clone()
                        .ok_or_else(|| wrap!("Failed to get the asm information for data section size from the current benchmarks.".into()))?["data_section"]["size"],
                    current_benchmarks
                        .benchmarks
                        .iter()
                        .find(|b| file_name.contains(&b.name))
                        .ok_or_else(|| wrap!("Failed to find the benchmark in the current benchmarks.".into()))?
                        .asm_information
                        .clone()
                        .ok_or_else(|| wrap!("Failed to get the asm information for data section used from the current benchmarks.".into()))?["data_section"]["used"]
                ));
                    html.push_str("</tbody></table>");

                    html.push_str("<h3>Flamegraphs</h3>");
                    html.push_str(&generate_flamegraphs(
                        current_benchmarks
                            .benchmarks
                            .iter()
                            .find(|b| file_name.contains(&b.name))
                            .ok_or_else(|| {
                                wrap!("Failed to find the benchmark in the current benchmarks."
                                    .into())
                            })?
                            .name
                            .as_str(),
                        &current_execution.flamegraphs_folder,
                    )?);

                    html.push_str("<h3>Plots</h3>");

                    html.push_str(
                        &generate_plots(&current_execution.plots_folder, file_name)
                            .map_err(|e| wrap!(e))?,
                    );

                    html.push_str("</div>");
                    html.push_str("</li>");
                }
            }

            html.push_str("</ul>"); // Close stats collection collapsible

            html.push_str("</div>"); // Close container-fluid
            html.push_str("</div>"); // Close collapsible-body
            html.push_str("</li>");
        }
    }

    html.push_str("</ul>"); // Close main collapsible

    html.push_str("</div>");
    html.push_str("<script src=\"https://cdnjs.cloudflare.com/ajax/libs/materialize/1.0.0/js/materialize.min.js\"></script>");
    html.push_str("<script>");
    html.push_str("document.addEventListener('DOMContentLoaded', function() {");
    html.push_str("var elems = document.querySelectorAll('.collapsible');");
    html.push_str("var instances = M.Collapsible.init(elems);");
    html.push_str("});");
    html.push_str("</script>");
    html.push_str("</body></html>");

    Ok(html)
}

fn generate_header() -> String {
    let mut header = String::new();
    header.push_str("<html><head>");
    header.push_str("<link href=\"https://cdnjs.cloudflare.com/ajax/libs/materialize/1.0.0/css/materialize.min.css\" rel=\"stylesheet\">");
    header.push_str("<link href=\"https://fonts.googleapis.com/icon?family=Material+Icons\" rel=\"stylesheet\">");
    header.push_str("<title>System Specifications</title>");
    header.push_str("<style>");
    header.push_str("body { background-color: #121212; color: #ffffff; }");
    header.push_str(".container-fluid { background-color: #1e1e1e; padding: 20px; border-radius: 8px; width: 100%; }");
    header.push_str("table.striped > tbody > tr:nth-child(odd) { background-color: #2c2c2c; }");
    header.push_str("table.striped > tbody > tr:nth-child(even) { background-color: #1e1e1e; }");
    header.push_str(".collapsible-header { background-color: #333333; color: #ffffff; }");
    header.push_str(".collapsible-body { background-color: #1e1e1e; color: #ffffff; }");
    header.push_str(".collapsible-header.active { background-color: #333333 !important; }");
    header.push_str("</style>");
    header.push_str("</head><body>");
    header
}

fn generate_system_specs(previous_benchmarks: &Benchmarks) -> String {
    let mut html = String::new();

    html.push_str("<ul class=\"collapsible\">");
    html.push_str("<li>");
    html.push_str("<div class=\"collapsible-header\"><i class=\"material-icons\">settings</i>System Specifications</div>");
    html.push_str("<div class=\"collapsible-body\"><table class=\"striped\">");
    html.push_str("<thead><tr><th>Specification</th><th>Value</th></tr></thead>");
    html.push_str("<tbody>");

    let specs = [
        (
            "Physical Core Count",
            previous_benchmarks
                .system_specs
                .physical_core_count
                .to_string(),
        ),
        (
            "Total Memory",
            previous_benchmarks.system_specs.total_memory.to_string(),
        ),
        (
            "Free Memory",
            previous_benchmarks.system_specs.free_memory.to_string(),
        ),
        (
            "Available Memory",
            previous_benchmarks
                .system_specs
                .available_memory
                .to_string(),
        ),
        (
            "Used Memory",
            previous_benchmarks.system_specs.used_memory.to_string(),
        ),
        (
            "Total Swap",
            previous_benchmarks.system_specs.total_swap.to_string(),
        ),
        (
            "Free Swap",
            previous_benchmarks.system_specs.free_swap.to_string(),
        ),
        (
            "Used Swap",
            previous_benchmarks.system_specs.used_swap.to_string(),
        ),
        (
            "Uptime",
            previous_benchmarks.system_specs.uptime.to_string(),
        ),
        (
            "Boot Time",
            previous_benchmarks.system_specs.boot_time.to_string(),
        ),
        (
            "Load Average",
            format!("{:?}", previous_benchmarks.system_specs.load_average),
        ),
        ("Name", previous_benchmarks.system_specs.name.clone()),
        (
            "Kernel Version",
            previous_benchmarks.system_specs.kernel_version.clone(),
        ),
        (
            "OS Version",
            previous_benchmarks.system_specs.os_version.clone(),
        ),
        (
            "Long OS Version",
            previous_benchmarks.system_specs.long_os_version.clone(),
        ),
        (
            "Distribution ID",
            previous_benchmarks.system_specs.distribution_id.clone(),
        ),
        (
            "Host Name",
            previous_benchmarks.system_specs.host_name.clone(),
        ),
    ];

    for (spec, value) in &specs {
        html.push_str(&format!("<tr><td>{spec}</td><td>{value}</td></tr>"));
    }

    html.push_str("</tbody></table>");

    // Add nested collapsible for CPU information
    html.push_str("<ul class=\"collapsible\">");
    html.push_str("<li>");
    html.push_str("<div class=\"collapsible-header\"><i class=\"material-icons\">memory</i>CPU Information</div>");
    html.push_str("<div class=\"collapsible-body\"><table class=\"striped\">");
    html.push_str(
        "<thead><tr><th>CPU</th><th>Vendor ID</th><th>Brand</th><th>Frequency</th></tr></thead>",
    );
    html.push_str("<tbody>");

    for cpu in &previous_benchmarks.system_specs.cpus {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{} MHz</td></tr>",
            cpu.name, cpu.vendor_id, cpu.brand, cpu.frequency
        ));
    }

    html.push_str("</tbody></table></div>");
    html.push_str("</li>");
    html.push_str("</ul>");

    html.push_str("</div>");
    html.push_str("</li>");
    html.push_str("</ul>");

    html
}

fn generate_previous_current_information(
    previous_benchmarks: &Benchmarks,
    current_benchmarks: &Benchmarks,
    previous_file_name: &Path,
    current_file_name: &Path,
) -> Result<String> {
    let mut html = String::new();

    // Add grid structure for the benchmarks
    html.push_str("<div class=\"row\">");

    let benchmarks_info = [
        (
            "Previous Benchmarks Collection",
            previous_file_name,
            previous_benchmarks,
        ),
        (
            "Current Benchmark Collection",
            current_file_name,
            current_benchmarks,
        ),
    ];

    for (title, file_name, benchmarks) in &benchmarks_info {
        let file_path = Path::new(file_name);
        html.push_str("<div class=\"col s12 m6\">");
        html.push_str(&format!("<h5>{title}</h5>"));
        html.push_str(&format!(
            "<p><strong>File Name:</strong> {}</p>",
            file_path
                .file_name()
                .ok_or_else(|| wrap!("Failed to get the file name from the file path.".into()))?
                .to_str()
                .ok_or_else(|| wrap!("Failed to get the file name as str.".into()))?
        ));
        html.push_str(&format!(
            "<p><strong>Forc Version:</strong> {}</p>",
            benchmarks.forc_version
        ));
        html.push_str(&format!(
            "<p><strong>Compiler File Hash:</strong> {}</p>",
            benchmarks.compiler_hash
        ));
        html.push_str(&format!(
            "<p><strong>Date and Time of Benchmarks:</strong> {}</p>",
            benchmarks.benchmarks_datetime
        ));
        html.push_str("</div>");
    }

    html.push_str("</div>"); // Close row

    // Add top-level metrics section
    html.push_str("<div class=\"row\">");
    html.push_str("<div class=\"col s12\">");
    html.push_str("<h5>Metrics</h5>");
    html.push_str("<table class=\"striped\">");
    html.push_str("<thead><tr><th>Metric</th><th>Previous Benchmark</th><th>Current Benchmark</th></tr></thead>");
    html.push_str("<tbody>");
    html.push_str(&format!(
        "<tr><td>Total Number of Files Processed</td><td>{}</td><td>{}</td></tr>",
        previous_benchmarks.benchmarks.len(),
        current_benchmarks.benchmarks.len()
    ));
    html.push_str(&format!(
        "<tr><td>Total Time of Execution</td><td>{} ms</td><td>{} ms</td></tr>",
        previous_benchmarks.total_time.as_millis(),
        current_benchmarks.total_time.as_millis()
    ));
    html.push_str("</tbody></table>");
    html.push_str("</div>");
    html.push_str("</div>");

    Ok(html)
}

fn generate_flamegraphs(benchmark_name: &str, flamegraph_file_path: &Path) -> Result<String> {
    let mut html = String::new();
    let flamegraphs_paths = get_all_files(flamegraph_file_path).map_err(|e| wrap!(e))?;

    let file1 = remove_first_component(&flamegraphs_paths[0]);
    let file1 = file1
        .to_str()
        .ok_or_else(|| wrap!("Failed to get the file path as str.".into()))?;
    let file2 = remove_first_component(&flamegraphs_paths[1]);
    let file2 = file2
        .to_str()
        .ok_or_else(|| wrap!("Failed to get the file path as str.".into()))?;

    let (previous_flamegraph, current_flamegraph) = sort_files_by_timestamp(file1, file2)?;

    // Start flexbox container
    html.push_str("<div style=\"display: flex; justify-content: space-between;\">");
    html.push_str(&format!(
        "<div><i class=\"material-icons\">whatshot</i> <a href=\"{previous_flamegraph}\" target=\"_blank\">Previous Flamegraph : {benchmark_name}</a></div>",
    ));
    html.push_str(&format!(
        "<div style=\"margin-left: auto; margin-right: auto;\"><i class=\"material-icons\">whatshot</i> <a href=\"{current_flamegraph}\" target=\"_blank\">Current Flamegraph : {benchmark_name}</a></div>",
            ));
    // End flexbox container
    html.push_str("</div>");

    Ok(html)
}

fn generate_plots(current_folder: &Path, name: &str) -> Result<String> {
    let mut html = String::new();
    let name = name
        .rsplit('/')
        .next()
        .ok_or_else(|| wrap!("Failed to get the name.".into()))?;

    let current_folder = remove_first_component(current_folder);

    let plot_sections = [
        ("CPU Usage", "cpu_usage"),
        ("Memory Usage", "memory_usage"),
        ("Virtual Memory Usage", "virtual_memory_usage"),
        ("Disk Total Written Bytes", "disk_total_written_bytes"),
        ("Disk Written Bytes", "disk_written_bytes"),
        ("Disk Total Read Bytes", "disk_total_read_bytes"),
        ("Disk Read Bytes", "disk_read_bytes"),
    ];

    for (title, suffix) in &plot_sections {
        html.push_str("<div class=\"row\">");
        // Previous benchmark plot
        html.push_str("<div class=\"col s12 m6\">");
        html.push_str(&format!("<h5>Previous Benchmark {title}</h5>"));
        html.push_str(&format!(
            "<img src=\"{}/previous_{name}_{suffix}.png\" alt=\"Previous Benchmark {title}\" class=\"responsive-img\">", current_folder.display()
        ));
        html.push_str("</div>");

        // Current benchmark plot
        html.push_str("<div class=\"col s12 m6\">");
        html.push_str(&format!("<h5>Current Benchmark {title}</h5>"));
        html.push_str(&format!(
            "<img src=\"{}/current_{name}_{suffix}.png\" alt=\"Current Benchmark {title}\" class=\"responsive-img\">", current_folder.display()
        ));
        html.push_str("</div>");

        html.push_str("</div>"); // Close row
    }

    Ok(html)
}

pub fn generate_error_page() -> String {
    let mut html = String::new();

    html.push_str("<html><head>");
    html.push_str("<link href=\"https://cdnjs.cloudflare.com/ajax/libs/materialize/1.0.0/css/materialize.min.css\" rel=\"stylesheet\">");
    html.push_str("<link href=\"https://fonts.googleapis.com/icon?family=Material+Icons\" rel=\"stylesheet\">");
    html.push_str("<title>404 Error Not Found</title>");
    html.push_str("<style>");
    html.push_str("body { background-color: #121212; color: #ffffff; display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0; }");
    html.push_str(".container { text-align: center; }");
    html.push_str("</style>");
    html.push_str("</head><body>");
    html.push_str("<div class=\"container\">");
    html.push_str("<h1>404 Error Not Found</h1>");
    html.push_str("</div>");
    html.push_str("</body></html>");

    html
}

fn remove_first_component(path: &Path) -> PathBuf {
    let mut components = path.components();
    components.next();
    components.as_path().to_path_buf()
}

fn parse_timestamp(file_name: &str) -> Result<chrono::NaiveDateTime> {
    let file = if file_name.contains("/") {
        // Get the part before the last / in the file name 
        let file_name_parts = file_name.split('/').collect::<Vec<_>>();
        
        file_name_parts[file_name_parts.len() - 2]
    }else {
        file_name
    };
    
    // Find the index of the second last underscore
    if let Some(second_last_underscore_index) = find_second_last_underscore(file) {
        // Extract the timestamp part
        let timestamp_str = &file[second_last_underscore_index + 1..];
        let timestamp = chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d_%H:%M:%S")
            .map_err(|e| wrap!(e.into()))?;
        Ok(timestamp)
    } else {
        Err("Invalid file name".into())
    }
}

fn sort_files_by_timestamp(file1: &str, file2: &str) -> Result<(String, String)> {
    let timestamp1 = parse_timestamp(file1)?;
    let timestamp2 = parse_timestamp(file2)?;

    if timestamp1 < timestamp2 {
        Ok((file1.to_string(), file2.to_string()))
    } else {
        Ok((file2.to_string(), file1.to_string()))
    }
}

fn find_second_last_underscore(s: &str) -> Option<usize> {
    let underscores: Vec<_> = s.match_indices('_').collect();
    if underscores.len() >= 2 {
        Some(underscores[underscores.len() - 2].0)
    } else {
        None
    }
}

fn get_all_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                files.extend(get_all_files(&path)?);
            } else {
                files.push(path);
            }
        }
    }
    Ok(files)
}
