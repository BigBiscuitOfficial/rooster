mod app;
mod editor;
mod requests;
mod tui;
mod ui;

use app::{AppState, EditorMode, MainScreenElement};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut terminal = tui::init()?;
    let mut app = AppState::new();

    let res = run_app(&mut terminal, &mut app).await;

    tui::restore()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

async fn run_app(terminal: &mut tui::Tui, app: &mut AppState) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| ui::ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(16))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            // Signal int, idk if needs proper sig handle
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                return Ok(());
            }

            match app.mode {
                EditorMode::View => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Up => app.up(),
                    KeyCode::Down => app.down(),
                    KeyCode::Left => app.left(),
                    KeyCode::Right => app.right(),
                    KeyCode::Enter => app.toggle_edit(),
                    _ => {}
                },
                EditorMode::Edit => {
                    // Special Header Actions (BROKEN)
                    if app.selected_section == MainScreenElement::Headers {
                        if key.code == KeyCode::Tab {
                            app.add_new_header();
                            continue;
                        }
                        if key.code == KeyCode::Char('d')
                            && key.modifiers.contains(KeyModifiers::CONTROL)
                        {
                            app.remove_current_header();
                            continue;
                        }
                    }

                    match key.code {
                        KeyCode::Esc => app.mode = EditorMode::View,
                        KeyCode::Enter => app.capture_char(KeyCode::Enter), // Could be better

                        // Editor nav, note to self: Add vim binds
                        KeyCode::Left => app.cursor_left(),
                        KeyCode::Right => app.cursor_right(),
                        KeyCode::Backspace => app.delete_char(),
                        _ => app.capture_char(key.code),
                    }
                }
            }
        }
    }
}
