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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_execute_generates_man_page() {
        let temp_dir = TempDir::new().unwrap();

        let args = GenerateManArgs {
            output: temp_dir.path().to_path_buf(),
        };

        let result = execute(args).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), exit_codes::SUCCESS);

        // Verify man page was created
        let man_path = temp_dir.path().join("repolens.1");
        assert!(man_path.exists());

        // Verify content is valid roff format (starts with .TH)
        let content = fs::read_to_string(&man_path).unwrap();
        assert!(
            content.contains(".TH"),
            "Man page should contain .TH header"
        );
        assert!(
            content.to_lowercase().contains("repolens"),
            "Man page should mention repolens"
        );
    }

    #[tokio::test]
    async fn test_execute_to_invalid_path() {
        let args = GenerateManArgs {
            output: PathBuf::from("/nonexistent/deeply/nested/path"),
        };

        let result = execute(args).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_command_factory() {
        // Verify Cli can be used to create a Command
        let cmd = Cli::command();
        assert_eq!(cmd.get_name(), "repolens");
    }
}
