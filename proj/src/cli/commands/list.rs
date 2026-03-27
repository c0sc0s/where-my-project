use std::{fs, path::Path};

use anyhow::{Context, Result};

use crate::{cli::args::ListArgs, tui};

pub fn run(args: ListArgs) -> Result<()> {
    if let Some(path) = tui::list::run()? {
        if let Some(selection_file) = args.selection_file {
            fs::write(Path::new(&selection_file), path.trim()).with_context(|| {
                format!("failed to write selection file at {}", selection_file)
            })?;
        } else {
            print!("{}", path.trim());
            std::io::Write::flush(&mut std::io::stdout())?;
        }
    }
    Ok(())
}
