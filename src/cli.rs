use clap::Parser;

/// Search directories for Git checkouts.
#[derive(Parser, Debug)]
#[command(version)] // Reads version from Cargo.toml
pub struct CLI {
    /// Max number of subdirectories to seach in.
    #[arg(short, long, default_value_t = 1)]
    pub depth: usize,

    /// The directory where to start the search.
    #[arg(short, long, default_value = ".")]
    pub start_dir: String,

    /// The regexp pattern to filter the paths of git repositories or their git-status strings,
    /// which contain the name of the checkout branch.
    pub regex_pattern: Option<String>,
}
