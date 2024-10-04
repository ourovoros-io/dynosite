use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(name = "Dyno Site Generator")]
#[clap(
    author = "Georgios Delkos <georgios@tenbeo.io>, Camden Smallwood <camden-smallwood@gmail.com>"
)]
#[clap(version = "1.0")]
#[clap(about = "Fuel Dyno Profiler Site Generator", long_about = None)]
pub struct Options {
    #[clap(short, long)]
    /// The folder containing the benchmarks
    pub benchmarks_folder: PathBuf,

    #[clap(short, long)]
    /// The PR hash
    pub pr_hash: String,

    /// The pr title
    #[clap(short = 't', long)]
    pub pr_title: String,

    /// The pr link
    #[clap(short = 'l', long)]
    pub pr_link: String,
}
