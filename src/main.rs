use anyhow::Result;

slint::include_modules!();

fn main() -> Result<()> {
    let ui = AppWindow::new()?;

    ui.on_gerald(|| {
        println!("test");
    });

    ui.run()?;

    Ok(())
}
