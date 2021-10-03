use crate::{settings::*, strfmt::*};
use anyhow::*;
use crossterm::{execute, terminal::*};
use futures_util::TryStreamExt;
use std::sync::Arc;
use tui::{backend::CrosstermBackend, layout::*, text::*, widgets::*, Terminal};
use tunet_rust::{usereg::UseregHelper, *};

mod event;
mod model;
mod view;

use event::*;
use model::*;

pub async fn run(state: NetState) -> Result<()> {
    let cred = read_cred()?;

    enable_raw_mode()?;
    execute!(std::io::stdout(), EnterAlternateScreen)?;

    let res = main_loop(state, cred.clone()).await;

    let res = if let Ok(()) = res {
        save_cred(cred).await
    } else {
        res
    };

    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    res
}

async fn main_loop(state: NetState, cred: Arc<NetCredential>) -> Result<()> {
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.clear()?;

    let client = create_http_client()?;
    let usereg = UseregHelper::new(cred.clone(), client.clone());
    let client = TUNetConnect::new(state, cred, client).await?;

    let mut event = Event::new(client, usereg);
    let mut model = Model::default();

    let mut interval = tokio::time::interval(std::time::Duration::from_micros(100));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                terminal.draw(|f| view::draw(&model, f))?;
            }
            m = event.try_next() => {
                if let Some(m) = m? {
                    if !model.handle(m) {
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
