#![forbid(unsafe_code)]

use anyhow::Result;
use clap::Parser;
use compio::runtime::RuntimeBuilder;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::*,
};
use futures_util::FutureExt;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::*,
    text::*,
    widgets::*,
    Terminal,
};
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
    run(opt.host)
}

pub fn run(state: Option<NetState>) -> Result<()> {
    let mut reader = SettingsReader::new()?;
    let (u, p) = reader.read_ask_full()?;

    enable_raw_mode()?;
    execute!(std::io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.clear()?;

    let res = RuntimeBuilder::new()
        .build()?
        .block_on(main_loop(&mut terminal, &u, &p, state));

    let res = if let Ok(()) = res {
        reader.save(&u, &p).map_err(anyhow::Error::from)
    } else {
        res
    };

    execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    disable_raw_mode()?;
    res
}

async fn main_loop<B: Backend>(
    terminal: &mut Terminal<B>,
    u: &str,
    p: &str,
    state: Option<NetState>,
) -> Result<()> {
    let mut event = Event::new()?;
    event
        .model
        .queue(Action::Credential(u.to_string(), p.to_string()));
    event.model.queue(Action::State(state));

    let mut interval = compio::time::interval(std::time::Duration::from_micros(100));
    event.start();

    loop {
        futures_util::select! {
            _ = interval.tick().fuse() => {
                terminal.draw(|f| view::draw(&event.model, f))?;
            }
            e = event.next_event().fuse() => {
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
