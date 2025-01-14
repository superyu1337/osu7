#[cfg(target_os = "windows")]
fn main() -> std::io::Result<()> {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        winresource::WindowsResource::new()
            // This path can be absolute, or relative to your crate root.
            .set_icon("../assets/icon.ico")
            .compile()?;
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn main() -> std::io::Result<()> {
    Ok(())
}
