// Simple test to understand ort API
use ort;

fn main() {
    println!("Testing ort API...");

    // Try different ways to create a session
    match ort::Session::builder() {
        Ok(builder) => {
            println!("Session::builder() exists");
            // Try to build with a file
            // builder.commit_from_file()
        }
        Err(e) => {
            println!("Session::builder() failed: {}", e);
        }
    }

    // Check what methods are available
    println!("ort version and available methods:");
    // This will show us compilation errors that reveal the API
}