mod app;
mod editor;
mod requests;
mod tui;
use app::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Constructor later
    let mut terminal = tui::init()?;
    let mut app = AppState::new();

    let res = app.run(&mut terminal).await;
    tui::restore()?;
    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}
