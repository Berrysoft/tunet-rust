use crate::cui::*;
use itertools::Itertools;
use std::collections::HashMap;
use tui::{backend::Backend, style::*, Frame};

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

    let details_group = m
        .details
        .iter()
        .group_by(|d| d.logout_time.date())
        .into_iter()
        .map(|(key, group)| (key.day(), group.map(|d| d.flux.0).sum::<u64>()))
        .collect::<HashMap<_, _>>();

    let max_day = Local::now().day();

    let mut details = vec![];
    let mut flux = 0.0;
    for d in 1u32..max_day {
        if let Some(f) = details_group.get(&d) {
            flux += *f as f64 / 1_000_000_000.0;
        }
        details.push((d as f64, flux));
    }

    let dataset = Dataset::default()
        .name("Detail")
        .marker(tui::symbols::Marker::Dot)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::White))
        .data(&details);

    let max_flux = m
        .flux
        .as_ref()
        .map(|f| f.flux.0 as f64 / 1_000_000_000.0)
        .unwrap_or_default()
        .max(flux)
        .max(1.0) as usize;

    let chart = Chart::new(vec![dataset])
        .style(Style::default().fg(Color::White))
        .x_axis(
            Axis::default()
                .title(Span::from("日期"))
                .style(Style::default().fg(Color::White))
                .bounds([1.0, max_day as f64])
                .labels(
                    (1..max_day)
                        .into_iter()
                        .map(|d| Span::from(d.to_string()))
                        .collect(),
                ),
        )
        .y_axis(
            Axis::default()
                .title(Span::from("流量"))
                .style(Style::default().fg(Color::White))
                .bounds([0.0, max_flux as f64])
                .labels(
                    (0..3)
                        .map(|f| Span::from((max_flux as f64 / 2.0 * f as f64).to_string()))
                        .collect(),
                ),
        );
    f.render_widget(chart, chunks[1]);
}
