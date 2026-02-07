//! Generate man page command

use crate::cli::Cli;
use crate::error::{ActionError, ConfigError, RepoLensError};
use crate::exit_codes;
use clap::CommandFactory;
use std::fs;

use super::GenerateManArgs;

/// Execute the generate-man command
pub async fn execute(args: GenerateManArgs) -> Result<i32, RepoLensError> {
    let cmd = Cli::command();
    let man = clap_mangen::Man::new(cmd);

    let output_path = args.output.join("repolens.1");

    let mut buffer: Vec<u8> = Vec::new();
    man.render(&mut buffer).map_err(|e| {
        RepoLensError::Config(ConfigError::Serialize {
            message: format!("Failed to generate man page: {}", e),
        })
    })?;

    fs::write(&output_path, buffer).map_err(|e| {
        RepoLensError::Action(ActionError::FileWrite {
            path: output_path.display().to_string(),
            source: e,
        })
    })?;

    println!("Man page generated: {}", output_path.display());

    Ok(exit_codes::SUCCESS)
}
