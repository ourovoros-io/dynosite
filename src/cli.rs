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
    /// The PR hash
    pub pr_hash: String,

    /// The PR title
    #[clap(short = 't', long)]
    pub pr_title: String,

    /// The PR link
    #[clap(short = 'l', long)]
    pub pr_link: String,
}
