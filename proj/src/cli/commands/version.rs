use anyhow::Result;
use colored::Colorize;

pub fn run() -> Result<()> {
    println!("{}", "proj".cyan().bold());
    println!("  Version: {}", env!("CARGO_PKG_VERSION").yellow());
    println!("  Build:   {}", "2025-03-27 (TUI Edition)".dimmed());
    println!("  Path:    {}", std::env::current_exe()?.display());
    Ok(())
}
