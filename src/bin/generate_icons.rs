use inferno::icon_generator::generate_app_icons;

fn main() -> anyhow::Result<()> {
    println!("Generating Inferno AI Runner app icons...");
    generate_app_icons()?;
    println!("Icon generation complete!");
    Ok(())
}
