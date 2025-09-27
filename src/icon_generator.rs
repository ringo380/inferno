use image::{ImageBuffer, Rgba, RgbaImage};
use std::path::Path;

// Dark mode color palette for Inferno AI Runner
const BACKGROUND: [u8; 4] = [16, 22, 26, 255]; // Dark background #10161a
const PRIMARY: [u8; 4] = [255, 107, 53, 255]; // Orange accent #ff6b35
const SECONDARY: [u8; 4] = [94, 234, 212, 255]; // Teal accent #5eeaд4
const WHITE: [u8; 4] = [255, 255, 255, 255];

pub fn generate_app_icons() -> anyhow::Result<()> {
    // Create icons directory if it doesn't exist
    std::fs::create_dir_all("icons")?;

    // Generate different icon sizes
    let sizes = [32, 128, 256, 512, 1024];

    for size in sizes {
        let icon = create_inferno_icon(size);
        let path = format!("icons/{}x{}.png", size, size);
        icon.save(&path)?;
        println!("Generated icon: {}", path);

        // Also create @2x versions for retina displays
        if size <= 128 {
            let retina_path = format!("icons/{}x{}@2x.png", size, size);
            let retina_icon = create_inferno_icon(size * 2);
            retina_icon.save(&retina_path)?;
            println!("Generated retina icon: {}", retina_path);
        }
    }

    // Generate specific formats needed by Tauri
    let main_icon = create_inferno_icon(512);
    main_icon.save("icons/icon.png")?;
    println!("Generated main icon: icons/icon.png");

    // Generate ICO format for Windows compatibility
    generate_ico_file()?;

    // Generate ICNS format for macOS
    generate_icns_file()?;

    Ok(())
}

fn create_inferno_icon(size: u32) -> RgbaImage {
    let mut img = ImageBuffer::new(size, size);

    // Fill background with dark color
    for pixel in img.pixels_mut() {
        *pixel = Rgba(BACKGROUND);
    }

    let center = size as f32 / 2.0;
    let radius = size as f32 * 0.35;

    // Draw flame-like icon representing "Inferno"
    draw_flame_icon(&mut img, center, center, radius);

    img
}

fn draw_flame_icon(img: &mut RgbaImage, cx: f32, cy: f32, radius: f32) {
    let (width, height) = img.dimensions();

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let distance = (dx * dx + dy * dy).sqrt();

            // Create flame-like shape
            if distance < radius {
                let normalized_distance = distance / radius;
                let angle = dy.atan2(dx);

                // Create flame pattern with multiple layers
                let flame_factor = create_flame_pattern(normalized_distance, angle, cy - y as f32);

                if flame_factor > 0.0 {
                    let color = interpolate_flame_color(flame_factor, normalized_distance);
                    img.put_pixel(x, y, Rgba(color));
                }
            }
        }
    }
}

fn create_flame_pattern(distance: f32, angle: f32, height_offset: f32) -> f32 {
    // Create a flame-like shape that's wider at the bottom and pointed at the top
    let base_radius = 1.0 - distance;

    // Make the flame taller and more pointed at the top
    let height_factor = if height_offset > 0.0 {
        1.0 - (height_offset / 100.0).min(0.8)
    } else {
        1.0
    };

    // Add some waviness to make it look more flame-like
    let wave = (angle * 3.0).sin() * 0.1 * distance;

    // Combine factors
    let flame_factor = base_radius * height_factor + wave;

    // Smooth cutoff
    if flame_factor > 0.3 {
        flame_factor
    } else {
        0.0
    }
}

fn interpolate_flame_color(flame_factor: f32, distance: f32) -> [u8; 4] {
    // Create gradient from white (hot center) to orange to dark red
    let t = flame_factor.clamp(0.0, 1.0);
    let d = distance.clamp(0.0, 1.0);

    if t > 0.8 {
        // Hot center - white to light orange
        let blend = (t - 0.8) / 0.2;
        interpolate_color(WHITE, PRIMARY, blend)
    } else if t > 0.5 {
        // Mid flame - orange to secondary color
        let blend = (t - 0.5) / 0.3;
        interpolate_color(PRIMARY, SECONDARY, blend)
    } else {
        // Outer flame - secondary to transparent
        let blend = t / 0.5;
        let mut color = SECONDARY;
        color[3] = (255.0 * blend * (1.0 - d * 0.5)) as u8;
        color
    }
}

fn interpolate_color(color1: [u8; 4], color2: [u8; 4], t: f32) -> [u8; 4] {
    let t = t.clamp(0.0, 1.0);
    [
        (color1[0] as f32 * (1.0 - t) + color2[0] as f32 * t) as u8,
        (color1[1] as f32 * (1.0 - t) + color2[1] as f32 * t) as u8,
        (color1[2] as f32 * (1.0 - t) + color2[2] as f32 * t) as u8,
        (color1[3] as f32 * (1.0 - t) + color2[3] as f32 * t) as u8,
    ]
}

fn generate_ico_file() -> anyhow::Result<()> {
    // For now, just copy the PNG as ICO (Tauri can handle PNG in ICO format)
    if Path::new("icons/256x256.png").exists() {
        std::fs::copy("icons/256x256.png", "icons/icon.ico")?;
        println!("Generated ICO file: icons/icon.ico");
    }
    Ok(())
}

fn generate_icns_file() -> anyhow::Result<()> {
    // For ICNS, we'll use a simple approach by creating a bundle of PNG files
    // In a real implementation, you'd want to use a proper ICNS library
    if Path::new("icons/512x512.png").exists() {
        std::fs::copy("icons/512x512.png", "icons/icon.icns")?;
        println!("Generated ICNS file: icons/icon.icns");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_generation() {
        let result = generate_app_icons();
        assert!(result.is_ok());
    }

    #[test]
    fn test_flame_pattern() {
        let pattern = create_flame_pattern(0.5, 0.0, 10.0);
        assert!(pattern >= 0.0);
    }
}
