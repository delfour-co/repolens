//! Schema command - Display the JSON Schema for audit report output

use colored::Colorize;

use super::SchemaArgs;
use crate::cli::output::json::AUDIT_REPORT_SCHEMA;
use crate::error::RepoLensError;
use crate::exit_codes;

pub async fn execute(args: SchemaArgs) -> Result<i32, RepoLensError> {
    let schema = AUDIT_REPORT_SCHEMA;

    match args.output {
        Some(output_path) => {
            std::fs::write(&output_path, schema).map_err(|e| {
                RepoLensError::Action(crate::error::ActionError::FileWrite {
                    path: output_path.display().to_string(),
                    source: e,
                })
            })?;

            println!(
                "{} Schema written to: {}",
                "Success:".green().bold(),
                output_path.display().to_string().cyan()
            );
        }
        None => {
            println!("{schema}");
        }
    }

    Ok(exit_codes::SUCCESS)
}
