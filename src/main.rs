mod app;
mod ui;
pub mod explorer;
mod viewer;
mod config;
mod utils;

use anyhow::{Result, Context};
use std::env;

fn main() -> Result<()> {
    // Check for command-line arguments
    let args: Vec<String> = env::args().collect();
    
    // Process any command-line arguments
    if args.len() > 1 {
        match args[1].as_str() {
            "--generate-config" | "-g" => {
                // Generate default config
                let config_path = config::Config::create_default_config()
                    .context("Failed to create default configuration file")?;
                
                println!("Default configuration created at: {}", config_path.display());
                println!("Edit this file to customize Packrat's behavior.");
                return Ok(());
            },
            "--help" | "-h" => {
                // Show help
                println!("Packrat - Interactive text file chunker");
                println!("");
                println!("USAGE:");
                println!("  packrat [OPTIONS]");
                println!("");
                println!("OPTIONS:");
                println!("  -g, --generate-config  Generate a default configuration file");
                println!("  -h, --help             Show this help message");
                println!("");
                println!("CONFIGURATION:");
                println!("  Packrat searches for configuration in the following locations:");
                println!("  1. ./packrat.toml (current directory)");
                println!("  2. User config directory (platform-specific)");
                println!("");
                println!("  Run 'packrat --generate-config' to create a default config file");
                println!("  with comments explaining all available options.");
                return Ok(());
            },
            _ => {
                println!("Unknown option: {}", args[1]);
                println!("Run 'packrat --help' for usage information");
                return Ok(());
            }
        }
    }
    
    // Initialize the application
    let mut app = app::App::new()?;
    
    // Run the application
    app.run()?;
    
    Ok(())
}
