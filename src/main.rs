mod app;
mod ui;
mod explorer;
mod viewer;
mod config;
mod utils;

use anyhow::Result;

fn main() -> Result<()> {
    // Initialize the application
    let mut app = app::App::new()?;
    
    // Run the application
    app.run()?;
    
    Ok(())
}
