use crate::cui::*;
use itertools::Itertools;
use std::collections::HashMap;
use tui::{backend::Backend, style::*, Frame};

fn get_flux_color(flux: u64, total: bool) -> Color {
    if flux == 0 {
        Color::LightCyan
    } else if flux < if total { 20_000_000_000 } else { 2_000_000_000 } {
        Color::LightYellow
    } else {
        Color::LightMagenta
    }
}

const GIGABYTES: f64 = 1_000_000_000.0;

pub fn draw<B: Backend>(m: &Model, f: &mut Frame<B>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Percentage(100)])
        .split(f.size());
    let title_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);
    let graph = if let Some(flux) = &m.flux {
        Paragraph::new(vec![
            Spans::from(format!("用户 {}", flux.username)),
            Spans::from(format!("流量 {}", flux.flux)),
            Spans::from(format!("时长 {}", FmtDuration(flux.online_time))),
            Spans::from(format!("余额 {}", flux.balance)),
        ])
    } else {
        Paragraph::new("加载中...")
    };
    f.render_widget(graph, title_chunks[0]);

    let table = Table::new(
        m.users
            .iter()
            .map(|u| {
                Row::new(vec![
                    u.address.to_string(),
                    FmtDateTime(u.login_time).to_string(),
                    u.mac_address.map(|a| a.to_string()).unwrap_or_default(),
                ])
            })
            .collect::<Vec<_>>(),
    )
    .widths(&[
        Constraint::Length(15),
        Constraint::Length(20),
        Constraint::Length(14),
    ]);
    f.render_widget(table, title_chunks[1]);

    let details_group = m
        .details
        .iter()
        .group_by(|d| d.logout_time.date())
        .into_iter()
        .map(|(key, group)| (key.day(), group.map(|d| d.flux.0).sum::<u64>()))
        .collect::<HashMap<_, _>>();

    let max_day = Local::now().day();

    let mut details = vec![];
    let mut flux = 0u64;
    for d in 1u32..max_day {
        if let Some(f) = details_group.get(&d) {
            flux += *f;
        }
        details.push((d as f64, flux as f64 / GIGABYTES));
    }
    let flux_g = flux as f64 / GIGABYTES;

    let max_flux = m
        .flux
        .as_ref()
        .map(|f| f.flux.0 as f64 / GIGABYTES)
        .unwrap_or_default()
        .max(flux_g)
        .max(1.0) as usize;

    let dataset = Dataset::default()
        .marker(tui::symbols::Marker::Dot)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(get_flux_color(flux, true)))
        .data(&details);

    let chart = Chart::new(vec![dataset])
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
