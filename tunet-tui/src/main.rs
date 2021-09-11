use anyhow::*;
use crossterm::{event::Event as TerminalEvent, event::*, terminal::*, ExecutableCommand};
use tokio::sync::mpsc;
use tui::{backend::CrosstermBackend, layout::*, text::*, widgets::*, Terminal};
use tunet_rust::*;

enum Event {
    TerminalEvent(TerminalEvent),
    Flux(NetFlux),
    Tick,
}

#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    stdout.execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.clear()?;

    let client = create_http_client()?;
    let client = TUNetConnect::new(NetState::Auto, NetCredential::default(), client).await?;

    let (tx, mut rx) = mpsc::channel::<Result<Event>>(10);

    {
        let tx = tx.clone();
        tokio::spawn(async move {
            loop {
                let res = tx
                    .send(
                        read()
                            .map(Event::TerminalEvent)
                            .map_err(anyhow::Error::from),
                    )
                    .await;
                if res.is_err() {
                    break;
                }
            }
        });
    }

    {
        let tx = tx.clone();
        let client = client.clone();
        tokio::spawn(async move {
            let flux = client.flux().await;
            tx.send(flux.map(Event::Flux)).await.ok();
        });
    }

    {
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                if tx.is_closed() {
                    break;
                }
                let tx = tx.clone();
                tokio::spawn(async move {
                    tx.send(Ok(Event::Tick)).await.ok();
                });
            }
        });
    }

    let mut flux: Option<NetFlux> = None;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(6), Constraint::Percentage(100)])
                .split(f.size());
            let graph = if let Some(flux) = &flux {
                Paragraph::new(vec![
                    Spans::from(format!("用户 {}", flux.username)),
                    Spans::from(format!("流量 {}", flux.flux)),
                    Spans::from(format!("时长 {}", flux.online_time)),
                    Spans::from(format!("余额 {}", flux.balance)),
                ])
            } else {
                Paragraph::new("Fetching...")
            };
            f.render_widget(graph, chunks[0]);
        })?;

        if let Some(m) = rx.recv().await {
            match m {
                Ok(m) => match m {
                    Event::TerminalEvent(e) => match e {
                        TerminalEvent::Key(k) => match k.code {
                            KeyCode::Char('q') => {
                                break;
                            }
                            _ => {}
                        },
                        _ => {}
                    },
                    Event::Flux(f) => {
                        flux = Some(f);
                    }
                    Event::Tick => {
                        if let Some(flux) = &mut flux {
                            flux.online_time = flux.online_time + Duration::seconds(1);
                        }
                    }
                },
                Err(_) => {}
            }
        }
    }

    let mut stdout = std::io::stdout();
    stdout.execute(LeaveAlternateScreen)?;
    Ok(())
}
