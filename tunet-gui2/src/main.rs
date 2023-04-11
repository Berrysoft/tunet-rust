use tunet_helper::Result;

slint::include_modules!();

#[tokio::main]
async fn main() -> Result<()> {
    let app = App::new()?;
    app.run()?;
    Ok(())
}
