use crate::{settings::*, strfmt::*};
use anyhow::*;
use crossterm::{terminal::*, ExecutableCommand};
use std::sync::Arc;
use tui::{backend::CrosstermBackend, layout::*, text::*, widgets::*, Terminal};
use tunet_rust::*;

mod event;
mod model;
mod view;

use event::*;
use model::*;

pub async fn run() -> Result<()> {
    let cred = read_cred()?;

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    stdout.execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.clear()?;

    let client = create_http_client()?;
    let client = Arc::new(TUNetConnect::new(NetState::Auto, cred, client).await?);

    let mut event = Event::new(client.clone());
    let mut model = Model::default();

    let mut res = Ok(());

    loop {
        terminal.draw(|f| view::draw(&model, f))?;

        if let Some(m) = event.next().await {
            match m {
                Ok(m) => {
                    if !model.handle(m) {
                        break;
                    }
                }
                Err(e) => {
                    res = Err(e);
                }
            }
        }
    }

    let mut stdout = std::io::stdout();
    stdout.execute(LeaveAlternateScreen)?;

    res?;

    save_cred(client.cred())?;

    Ok(())
}
