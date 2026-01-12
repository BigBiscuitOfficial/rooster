mod app;
mod requests;
mod tui;
use app::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Constructor later
    let terminal = tui::init()?;
    let mut app = AppState::new(terminal);

    let res = app.run().await;
    tui::restore()?;
    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}
