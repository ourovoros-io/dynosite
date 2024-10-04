use crate::types::Benchmarks;

use crate::error::Result;
use crate::wrap;

pub fn generate(
    previous_benchmarks: &Benchmarks,
    current_benchmarks: &Benchmarks,
    previous_file_name: &str,
    current_file_name: &str,
    stats_collection: &crate::types::Collection,
    options: &crate::cli::Options,
) -> Result<String> {
    let mut html = generate_header();

    html.push_str("<div class=\"container-fluid\">");
    html.push_str("<h1>Forc Performance Profiler</h1>");
    html.push_str(&format!("<p>Version: {}</p>", crate::constants::VERSION));
    html.push_str(&generate_system_specs(previous_benchmarks));

    // Add collapsible structure for each folder in the data directory
    let data_dir = std::path::Path::new(crate::constants::SITE_FOLDER)
        .join(crate::constants::SITE_DATA_FOLDER);
    let folders = std::fs::read_dir(data_dir).map_err(|e| wrap!(e.into()))?;

    html.push_str("<h5>Run Collection</h5>");
    html.push_str("<ul class=\"collapsible\">");

    for folder in folders {
        let folder = folder.map_err(|e| wrap!(e.into()))?;
        let folder_path = folder.path();
        if folder_path.is_dir() {
            let folder_name = folder_path.file_name().unwrap().to_str().unwrap();

            html.push_str("<li>");
            html.push_str(&format!(
                "<div class=\"collapsible-header\"><i class=\"material-icons\">folder</i>{}_{}_{}</div>", folder_name, options.pr_hash, options.pr_title
            ));
            html.push_str("<div class=\"collapsible-body\">");
            html.push_str("<div class=\"container-fluid\">");
            html.push_str(&format!("<h5>PR Link : {}</h5>", options.pr_link));
            html.push_str(&generate_previous_current_information(
                previous_benchmarks,
                current_benchmarks,
                previous_file_name,
                current_file_name,
            ));

            // Add collapsible structure for the stats collection
            html.push_str("<ul class=\"collapsible\">");

            for (file_name, stats) in &stats_collection.0 {
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

                html.push_str(&generate_flamegraphs(
                    previous_file_name,
                    current_file_name,
                    options.benchmarks_folder.display().to_string().as_str(),
                    folder_name,
                    current_benchmarks
                        .benchmarks
                        .iter()
                        .find(|b| file_name.contains(&b.name))
                        .unwrap()
                        .name
                        .as_str(),
                )?);

                html.push_str("</tbody></table>");
                let current_plot_folder = std::path::Path::new(crate::constants::SITE_DATA_FOLDER)
                    .join(folder_name)
                    .join(crate::constants::SITE_PLOTS_FOLDER);
                html.push_str(&generate_plots(
                    &current_plot_folder.display().to_string(),
                    file_name,
                ));

                html.push_str("</div>");
                html.push_str("</li>");
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
    previous_file_name: &str,
    current_file_name: &str,
) -> String {
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
        html.push_str("<div class=\"col s12 m6\">");
        html.push_str(&format!("<h5>{title}</h5>"));
        html.push_str(&format!("<p><strong>File Name:</strong> {file_name}</p>"));
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

    html
}

fn generate_flamegraphs(
    previous_file_name: &str,
    current_file_name: &str,
    path: &str,
    current_folder: &str,
    benchmark_name: &str,
) -> Result<String> {
    let previous_file_name = previous_file_name
        .strip_suffix(".json")
        .ok_or_else(|| wrap!("Failed to remove suffix from previous filename".into()))?;

    let previous_file_name = previous_file_name
        .split("runs/")
        .nth(1)
        .ok_or_else(|| wrap!("Failed to split previous filename".into()))?;

    let current_file_name = current_file_name
        .strip_suffix(".json")
        .ok_or_else(|| wrap!("Failed to remove suffix from current filename".into()))?;

    let current_file_name = current_file_name
        .split("runs/")
        .nth(1)
        .ok_or_else(|| wrap!("Failed to split current filename".into()))?;

    let benchmarks_flamegraphs_folder =
        std::path::Path::new(path).join(crate::constants::BENCHMARKS_FLAMEGRAPHS_FOLDER);

    let site_flamegraphs_folder = std::path::Path::new(crate::constants::SITE_FOLDER)
        .join(crate::constants::SITE_DATA_FOLDER)
        .join(current_folder)
        .join(crate::constants::SITE_FLAMEGRAPHS_FOLDER);

    copy_flamegraph_folder(&benchmarks_flamegraphs_folder, &site_flamegraphs_folder)
        .map_err(|e| wrap!(e))?;

    let mut html = String::new();

    for file_name in &[previous_file_name, current_file_name] {
        let flamegraph_path = site_flamegraphs_folder.join(file_name);
        let flamegraph_files = std::fs::read_dir(&flamegraph_path).map_err(|e| wrap!(e.into()))?;

        for file in flamegraph_files {
            let file = file.map_err(|e| wrap!(e.into()))?;
            let file_name = file.file_name().into_string().unwrap();

            // Check if the file is an SVG
            if file_name.ends_with(".svg") {
                let f_name = file_name.strip_suffix(".svg").unwrap();

                if f_name == benchmark_name {
                    let flamegraph_file_path =
                        flamegraph_path.display().to_string().replace("site/", "");

                    html.push_str(&format!(
                    "<p><i class=\"material-icons\">whatshot</i> <a href=\"{flamegraph_file_path}/{file_name}\" target=\"_blank\">Flamegraph : {file_name}</a></p>",
                ));
                }
            }
        }
    }

    Ok(html)
}

fn generate_plots(current_folder: &str, file_name: &str) -> String {
    let mut html = String::new();
    let name = file_name.rsplit('/').next().unwrap();

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
            "<img src=\"{current_folder}/previous_{name}_{suffix}.png\" alt=\"Previous Benchmark {title}\" style=\"width: 100%;\">"
        ));
        html.push_str("</div>");

        // Current benchmark plot
        html.push_str("<div class=\"col s12 m6\">");
        html.push_str(&format!("<h5>Current Benchmark {title}</h5>"));
        html.push_str(&format!(
            "<img src=\"{current_folder}/current_{name}_{suffix}.png\" alt=\"Current Benchmark {title}\" style=\"width: 100%;\">"
        ));
        html.push_str("</div>");

        html.push_str("</div>"); // Close row
    }

    html
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

fn copy_flamegraph_folder(src: &std::path::Path, dest: &std::path::Path) -> Result<()> {
    if !dest.exists() {
        std::fs::create_dir_all(dest)?;
    }

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            copy_flamegraph_folder(&entry.path(), &dest.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dest.join(entry.file_name()))?;
        }
    }

    Ok(())
}
