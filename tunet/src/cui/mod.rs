use crate::{settings::*, strfmt::*};
use anyhow::*;
use crossterm::{execute, terminal::*};
use futures_util::TryStreamExt;
use tui::{backend::CrosstermBackend, layout::*, text::*, widgets::*, Terminal};
use tunet_rust::{usereg::UseregHelper, *};

mod event;
mod model;
mod view;

use event::*;
use model::*;

pub async fn run() -> Result<()> {
    let cred = read_cred()?;

    enable_raw_mode()?;
    execute!(std::io::stdout(), EnterAlternateScreen)?;

    let res = main_loop(cred).await;

    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    res
}

async fn main_loop(cred: NetCredential) -> Result<()> {
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.clear()?;

    let client = create_http_client()?;
    let usereg = UseregHelper::new(cred.clone(), client.clone());
    let client = TUNetConnect::new(NetState::Auto, cred, client).await?;

    let mut event = Event::new(client, usereg);
    let mut model = Model::default();

    loop {
        terminal.draw(|f| view::draw(&model, f))?;

        if let Some(m) = event.try_next().await? {
            if !model.handle(m) {
                break;
            }
        }
    }

    Ok(())
}
