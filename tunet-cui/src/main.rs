#![forbid(unsafe_code)]

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::*,
};
use futures_util::TryStreamExt;
use tokio::runtime::Builder as RuntimeBuilder;
use tui::{backend::CrosstermBackend, layout::*, text::*, widgets::*, Terminal};
use tunet_helper::*;
use tunet_model::Action;
use tunet_settings::*;

mod event;
mod view;

use event::*;

#[derive(Debug, Parser)]
#[clap(about, version, author)]
struct Opt {
    #[clap(long, short = 's')]
    /// 连接方式
    host: Option<NetState>,
}

fn main() -> Result<()> {
    let opt = Opt::parse();
    RuntimeBuilder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(run(opt.host))
}

pub async fn run(state: Option<NetState>) -> Result<()> {
    let mut event = Event::new()?;

    event.model.queue(Action::Credential(read_cred()?));
    event.model.queue(Action::State(state));

    enable_raw_mode()?;
    execute!(std::io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    let res = main_loop(&mut event).await;

    let res = if let Ok(()) = res {
        save_cred(event.model.cred.clone())
            .await
            .map_err(anyhow::Error::from)
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
    event.start();

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
