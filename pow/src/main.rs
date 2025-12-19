use std::error::Error;

use pow::start_node;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Set Wayland backend before anything else
    unsafe { std::env::set_var("WINIT_UNIX_BACKEND", "wayland") };
    
    start_node()
}