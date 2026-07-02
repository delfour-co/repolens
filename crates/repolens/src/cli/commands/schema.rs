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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_execute_to_stdout() {
        let args = SchemaArgs { output: None };
        let result = execute(args).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), exit_codes::SUCCESS);
    }

    #[tokio::test]
    async fn test_execute_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("schema.json");

        let args = SchemaArgs {
            output: Some(output_path.clone()),
        };

        let result = execute(args).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), exit_codes::SUCCESS);

        // Verify file was created
        assert!(output_path.exists());

        // Verify content is valid JSON
        let content = std::fs::read_to_string(&output_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.get("$schema").is_some() || parsed.get("type").is_some());
    }

    #[tokio::test]
    async fn test_execute_to_invalid_path() {
        let args = SchemaArgs {
            output: Some(PathBuf::from("/nonexistent/deeply/nested/path/schema.json")),
        };

        let result = execute(args).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_schema_is_valid_json() {
        let schema = AUDIT_REPORT_SCHEMA;
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(schema);
        assert!(parsed.is_ok(), "AUDIT_REPORT_SCHEMA should be valid JSON");
    }
}
