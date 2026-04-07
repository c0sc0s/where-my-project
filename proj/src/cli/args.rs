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
    List(ListArgs),
    Scan(ScanArgs),
    Status(StatusArgs),
    Cd(CdArgs),
    Work(WorkArgs),
    Init,
    Version,
}

#[derive(Debug, Args)]
pub struct ListArgs {
    #[arg(long)]
    pub selection_file: Option<String>,
}

#[derive(Debug, Args)]
pub struct ScanArgs {
    #[arg(value_name = "PATH", value_delimiter = ',')]
    pub paths: Vec<String>,

    #[arg(long)]
    pub tui: bool,
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
    /// Repository name, index, or full path of the project to work in
    pub target: String,
}
