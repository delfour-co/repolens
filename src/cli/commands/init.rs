//! Init command - Initialize a new configuration file

use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{Confirm, Select};
use std::fs;
use std::path::Path;

use super::InitArgs;
use crate::config::{Config, Preset};

const CONFIG_FILENAME: &str = ".repolens.toml";

pub async fn execute(args: InitArgs) -> Result<i32> {
    let config_path = Path::new(CONFIG_FILENAME);

    // Check if config already exists
    if config_path.exists() && !args.force {
        if args.non_interactive {
            eprintln!(
                "{} Configuration file already exists. Use --force to overwrite.",
                "Error:".red().bold()
            );
            return Ok(1);
        }

        let overwrite = Confirm::new()
            .with_prompt("Configuration file already exists. Overwrite?")
            .default(false)
            .interact()?;

        if !overwrite {
            println!("{}", "Aborted.".yellow());
            return Ok(0);
        }
    }

    // Determine preset
    let preset = if let Some(preset_name) = args.preset {
        Preset::from_name(&preset_name).context("Invalid preset name")?
    } else if args.non_interactive {
        Preset::OpenSource
    } else {
        select_preset()?
    };

    // Create configuration
    let config = Config::from_preset(preset);

    // Write configuration file
    let config_content = config.to_toml()?;
    fs::write(config_path, &config_content)
        .context("Failed to write configuration file")?;

    println!(
        "{} Created {} with preset '{}'",
        "Success:".green().bold(),
        CONFIG_FILENAME.cyan(),
        preset.name().yellow()
    );

    println!("\nNext steps:");
    println!("  1. Review and customize {}", CONFIG_FILENAME.cyan());
    println!("  2. Run {} to see planned actions", "repolens plan".cyan());
    println!("  3. Run {} to apply changes", "repolens apply".cyan());

    Ok(0)
}

fn select_preset() -> Result<Preset> {
    let presets = vec![
        ("opensource", "Open Source - Prepare repository for public release"),
        ("enterprise", "Enterprise - Internal company standards"),
        ("strict", "Strict - Maximum security and compliance checks"),
    ];

    let selection = Select::new()
        .with_prompt("Select a preset")
        .items(&presets.iter().map(|(_, desc)| *desc).collect::<Vec<_>>())
        .default(0)
        .interact()?;

    Ok(match selection {
        0 => Preset::OpenSource,
        1 => Preset::Enterprise,
        2 => Preset::Strict,
        _ => Preset::OpenSource,
    })
}
