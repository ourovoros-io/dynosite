use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(name = "Dynosite Generator")]
#[clap(
    author = "Georgios Delkos <georgios@tenbeo.io>, Camden Smallwood <camden-smallwood@gmail.com>"
)]
#[clap(version = "1.0")]
#[clap(about = "Fuel Dynosite Profiler Site Generator", long_about = None)]
pub struct Options {
    #[clap(short, long)]
    /// The target folder containing the benchmarks
    pub benchmarks_folder: PathBuf,

    #[clap(short, long)]
    /// Data only mode
    pub data_only: bool,

    #[clap(short, long)]
    /// The site name (Optional)
    pub site_name: Option<String>,

    #[clap(short, long)]
    /// The PR hash (Optional)
    pub pr_hash: Option<String>,

    #[clap(short = 't', long)]
    /// The PR title (Optional)
    pub pr_title: Option<String>,

    #[clap(short = 'l', long)]
    /// The PR link (Optional)
    pub pr_link: Option<String>,
}
