use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use serde::{Deserialize, Serialize};
/// A collection of benchmarks and system specifications.
#[derive(Debug, Serialize, Deserialize)]
pub struct Benchmarks {
    /// Total time taken to run all benchmarks
    pub total_time: Duration,
    /// The system specifications of the machine running the benchmarks.
    pub system_specs: SystemSpecs,
    /// The benchmarks data that was collected.
    pub benchmarks: Vec<Benchmark>,
    /// The forc version
    pub forc_version: String,
    /// The compiler hash
    pub compiler_hash: String,
    /// The time that the benchmarks were run
    pub benchmarks_datetime: String,
}

/// A collection of system hardware specifications.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemSpecs {
    /// The global cpu usage of the system.
    #[serde(skip_serializing, skip_deserializing)]
    pub global_cpu_usage: f64,
    /// The cpus of the system.
    pub cpus: Vec<Cpu>,
    /// The physical core count of the system.
    pub physical_core_count: i64,
    /// The total memory of the system.
    pub total_memory: i64,
    /// The free memory of the system.
    pub free_memory: i64,
    /// The available memory of the system.
    pub available_memory: i64,
    /// The used memory of the system.
    pub used_memory: i64,
    /// The total swap of the system.
    pub total_swap: i64,
    /// The free swap of the system.
    pub free_swap: i64,
    /// The used swap of the system.
    pub used_swap: i64,
    /// The uptime of the system.
    pub uptime: i64,
    /// The boot time of the system.
    pub boot_time: i64,
    /// The load average of the system.
    pub load_average: LoadAverage,
    /// The name of the system.
    pub name: String,
    /// The kernel version of the system.
    pub kernel_version: String,
    /// The os version of the system.
    pub os_version: String,
    /// The long os version of the system.
    pub long_os_version: String,
    /// The distribution id of the system.
    pub distribution_id: String,
    /// The host name of the system.
    pub host_name: String,
}

/// A collection of specifications for a single cpu.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cpu {
    #[serde(skip_serializing, skip_deserializing)]
    /// The usage of the cpu at the time of querying.
    pub cpu_usage: f64,
    /// The name of the cpu.
    pub name: String,
    /// The vendor id of the cpu.
    pub vendor_id: String,
    /// The brand of the cpu.
    pub brand: String,
    /// The frequency of the cpu.
    pub frequency: i64,
}

/// System load average specifications.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadAverage {
    /// The `one` of the load average.
    pub one: f64,
    /// The `five` of the load average.
    pub five: f64,
    /// The `fifteen` of the load average.
    pub fifteen: f64,
}

/// Benchmark metadata and phase-specific performance data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Benchmark {
    /// The name of the benchmark.
    pub name: String,
    /// The path to the benchmark's project folder.
    pub path: PathBuf,
    /// The start time of the benchmark.
    pub start_time: Option<Duration>,
    /// The end time of the benchmark.
    pub end_time: Option<Duration>,
    /// The phases of the benchmark.
    pub phases: Vec<BenchmarkPhase>,
    /// The performance frames collected from the benchmark.
    pub frames: Arc<Mutex<Vec<BenchmarkFrame>>>,
    /// The bytecode information
    pub asm_information: Option<serde_json::Value>,
    /// The hyperfine information
    pub hyperfine: Option<serde_json::Value>,
}

/// A named collection of performance frames representing a single phase of a benchmark.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkPhase {
    /// The name of the benchmark phase.
    pub name: String,
    /// The start time of the benchmark phase.
    pub start_time: Option<Duration>,
    /// The end time of the benchmark phase.
    pub end_time: Option<Duration>,
}

/// A single frame of performance information for a benchmark phase.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkFrame {
    /// The time that the frame was captured.
    pub timestamp: Duration,
    /// The relative to the benchmark timestamp.
    pub relative_timestamp: Duration,
    /// The process-specific CPU usage at the time the frame was captured.
    pub cpu_usage: f32,
    /// The total process-specific memory usage (in bytes) at the time the frame was captured.
    pub memory_usage: u64,
    /// The total process-specific virtual memory usage (in bytes) at the time the frame was captured.
    pub virtual_memory_usage: u64,
    /// The total number of bytes the process has written to disk at the time the frame was captured.
    pub disk_total_written_bytes: u64,
    /// The number of bytes the process has written to disk since the last refresh at the time the frame was captured.
    pub disk_written_bytes: u64,
    /// The total number of bytes the process has read from disk at the time the frame was captured.
    pub disk_total_read_bytes: u64,
    /// The number of bytes the process has read from disk since the last refresh at the time the frame was captured.
    pub disk_read_bytes: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Collection(pub Vec<(String, Stats)>);

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Stats {
    pub cpu_usage: (f64, f64),
    pub memory_usage: (f64, f64),
    pub virtual_memory_usage: (f64, f64),
    pub disk_total_written_bytes: (f64, f64),
    pub disk_written_bytes: (f64, f64),
    pub disk_total_read_bytes: (f64, f64),
    pub disk_read_bytes: (f64, f64),
    pub bytecode_size: (f64, f64),
    pub data_section_size: (f64, f64),
    pub time: (f64, f64),
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct PRInformation {
    pub hash: String,
    pub title: String,
    pub link: String,
}
