use crate::cui::*;
use tui::{backend::Backend, Frame};

pub fn draw<B: Backend>(m: &Model, f: &mut Frame<B>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Percentage(100)])
        .split(f.size());
    let graph = if let Some(flux) = &m.flux {
        Paragraph::new(vec![
            Spans::from(format!("用户 {}", flux.username)),
            Spans::from(format!("流量 {}", flux.flux)),
            Spans::from(format!("时长 {}", FmtDuration(flux.online_time))),
            Spans::from(format!("余额 {}", flux.balance)),
        ])
    } else {
        Paragraph::new("Fetching...")
    };
    f.render_widget(graph, chunks[0]);

    let table = Table::new(
        m.details
            .iter()
            .map(|d| {
                Row::new(vec![
                    FmtDateTime(d.login_time).to_string(),
                    FmtDateTime(d.logout_time).to_string(),
                    d.flux.to_string(),
                ])
            })
            .collect::<Vec<_>>(),
    )
    .header(Row::new(vec!["登录时间", "注销时间", "流量"]))
    .widths(&[
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ]);
    f.render_widget(table, chunks[1]);
}
