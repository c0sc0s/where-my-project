use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "proj",
    version,
    about = "Manage multiple clones of the same git repository"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Watch(WatchArgs),
    List(ListArgs),
    Scan(ScanArgs),
    Alias(AliasArgs),
    Status(StatusArgs),
    Cd(CdArgs),
    Work(WorkArgs),
    Init,
    Version,
}

#[derive(Debug, Args)]
pub struct WatchArgs {
    pub repo_name: Option<String>,

    #[arg(long)]
    pub list: bool,

    #[arg(long)]
    pub remove: Option<String>,
}

#[derive(Debug, Args)]
pub struct ListArgs {
    #[arg(long)]
    pub selection_file: Option<String>,
}

#[derive(Debug, Args)]
pub struct ScanArgs {
    #[arg(long, value_delimiter = ',')]
    pub paths: Option<Vec<String>>,

    #[arg(long, default_value = "true")]
    pub auto_alias: bool,

    #[arg(long)]
    pub tui: bool,
}

#[derive(Debug, Args)]
pub struct AliasArgs {
    pub target: Option<String>,
    pub alias_name: Option<String>,

    #[arg(long)]
    pub list: bool,
}

#[derive(Debug, Args)]
pub struct StatusArgs {
    pub target: Option<String>,
}

#[derive(Debug, Args)]
pub struct CdArgs {
    pub target: String,

    #[arg(long)]
    pub raw: bool,
}

#[derive(Debug, Args)]
pub struct WorkArgs {
    /// Alias or index of the project to work in
    pub target: String,
}
