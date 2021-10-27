use crate::settings::*;
use anyhow::*;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::*,
};
use futures_util::TryStreamExt;
use tui::{backend::CrosstermBackend, layout::*, text::*, widgets::*, Terminal};
use tunet_model::Action;
use tunet_rust::*;

mod event;
mod view;

use event::*;

pub async fn run(state: NetState) -> Result<()> {
    let mut event = Event::new()?;

    event.model.queue(Action::Credential(read_cred()?));
    event.model.queue(Action::State(Some(state)));
    event.start();

    enable_raw_mode()?;
    execute!(std::io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    let res = main_loop(&mut event).await;

    let res = if let Ok(()) = res {
        save_cred(event.model.cred.clone()).await
    } else {
        res
    };

    execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    disable_raw_mode()?;
    res
}

async fn main_loop(event: &mut Event) -> Result<()> {
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.clear()?;

    let mut interval = tokio::time::interval(std::time::Duration::from_micros(100));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                terminal.draw(|f| view::draw(&event.model, f))?;
            }
            e = event.try_next() => {
                if let Some(e) = e? {
                    if !event.handle(e, terminal.size()?) {
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
