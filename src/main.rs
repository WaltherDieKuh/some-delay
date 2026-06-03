mod dsp;
mod audio;
mod ui;

use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Try to initialize audio engine
    let engine = match audio::AudioEngine::new() {
        Ok(engine) => {
            println!("Audio engine initialized successfully");
            Some(Arc::new(engine))
        }
        Err(e) => {
            eprintln!("Failed to initialize audio engine: {}", e);
            eprintln!("Continuing without audio engine (UI only)");
            None
        }
    };

    ui::run(engine)?;
    Ok(())
}
