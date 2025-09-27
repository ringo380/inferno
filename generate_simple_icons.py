#!/usr/bin/env python3
"""
Simple icon generator for Inferno AI Runner - Dark Mode
Creates flame-themed icons in required sizes for Tauri
"""

import os
from PIL import Image, ImageDraw
import math

# Dark mode color palette
BACKGROUND = (16, 22, 26, 255)  # Dark background #10161a
PRIMARY = (255, 107, 53, 255)   # Orange accent #ff6b35
SECONDARY = (94, 234, 212, 255) # Teal accent #5eeaد4
WHITE = (255, 255, 255, 255)

def create_flame_icon(size):
    """Create a flame-themed icon for the given size"""
    img = Image.new('RGBA', (size, size), BACKGROUND)
    draw = ImageDraw.Draw(img)

    center = size // 2
    radius = size // 3

    # Draw multiple flame layers for depth
    flame_points = []
    num_points = 8

    for i in range(num_points):
        angle = (2 * math.pi * i) / num_points
        if i % 2 == 0:  # Outer points (flame tips)
            r = radius + (radius * 0.3 * (1 - i/num_points))  # Taper toward top
            x = center + r * math.cos(angle - math.pi/2)
            y = center + r * math.sin(angle - math.pi/2) * 0.8  # Make it more flame-like
        else:  # Inner points
            r = radius * 0.7
            x = center + r * math.cos(angle - math.pi/2)
            y = center + r * math.sin(angle - math.pi/2) * 0.8

        flame_points.extend([x, y])

    # Draw base flame shape with gradient effect
    draw.polygon(flame_points, fill=PRIMARY)

    # Add inner highlight
    inner_points = []
    for i in range(0, len(flame_points), 2):
        x = flame_points[i]
        y = flame_points[i+1]
        # Scale toward center
        x = center + (x - center) * 0.6
        y = center + (y - center) * 0.6
        inner_points.extend([x, y])

    draw.polygon(inner_points, fill=SECONDARY)

    # Add hot center
    core_radius = radius // 4
    draw.ellipse([
        center - core_radius, center - core_radius,
        center + core_radius, center + core_radius
    ], fill=WHITE)

    return img

def generate_icons():
    """Generate all required icon sizes"""
    os.makedirs('icons', exist_ok=True)

    sizes = [16, 32, 64, 128, 256, 512, 1024]

    for size in sizes:
        icon = create_flame_icon(size)

        # Save standard size
        icon.save(f'icons/{size}x{size}.png')
        print(f"Generated: icons/{size}x{size}.png")

        # Save @2x version for retina displays (for smaller sizes)
        if size <= 128:
            retina_icon = create_flame_icon(size * 2)
            retina_icon.save(f'icons/{size}x{size}@2x.png')
            print(f"Generated: icons/{size}x{size}@2x.png")

    # Create main icon files
    main_icon = create_flame_icon(512)
    main_icon.save('icons/icon.png')
    print("Generated: icons/icon.png")

    # Create ICO (just copy PNG for now)
    main_icon.save('icons/icon.ico')
    print("Generated: icons/icon.ico")

    # Create ICNS (macOS format - simplified)
    main_icon.save('icons/icon.icns')
    print("Generated: icons/icon.icns")

    print("\nIcon generation complete!")
    print("Dark mode flame-themed icons created for Inferno AI Runner")

if __name__ == "__main__":
    generate_icons()