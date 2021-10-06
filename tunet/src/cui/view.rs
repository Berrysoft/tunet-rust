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
    let global_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(f.size().height - 1), Constraint::Min(1)])
        .split(f.size());
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(32), Constraint::Percentage(100)])
        .split(global_chunks[0]);
    let title_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Percentage(100)])
        .split(chunks[0]);

    let subtitle_style = Style::default().fg(Color::Cyan);

    let graph = if let Some(flux) = &m.flux {
        Paragraph::new(vec![
            Spans::from(vec![
                Span::styled("用户 ", subtitle_style),
                Span::styled(
                    &flux.username,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Spans::from(vec![
                Span::styled("流量 ", subtitle_style),
                Span::styled(
                    flux.flux.to_string(),
                    Style::default()
                        .fg(get_flux_color(flux.flux.0, true))
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Spans::from(vec![
                Span::styled("时长 ", subtitle_style),
                Span::styled(
                    FmtDuration(flux.online_time).to_string(),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Spans::from(vec![
                Span::styled("余额 ", subtitle_style),
                Span::styled(flux.balance.to_string(), Style::default().fg(Color::Yellow)),
            ]),
        ])
    } else {
        Paragraph::new("")
    }
    .block(Block::default().title("基础信息").borders(Borders::all()));
    f.render_widget(graph, title_chunks[0]);

    let table = List::new(
        m.users
            .iter()
            .map(|u| {
                ListItem::new(Text::from(vec![
                    Spans::from(vec![
                        Span::styled("IP 地址  ", subtitle_style),
                        Span::styled(
                            u.address.to_string(),
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Spans::from(vec![
                        Span::styled("登录时间 ", subtitle_style),
                        Span::styled(
                            FmtDateTime(u.login_time).to_string(),
                            Style::default().fg(Color::Green),
                        ),
                    ]),
                    Spans::from(vec![
                        Span::styled("MAC 地址 ", subtitle_style),
                        Span::styled(
                            u.mac_address.map(|a| a.to_string()).unwrap_or_default(),
                            Style::default().fg(Color::LightCyan),
                        ),
                    ]),
                    Spans::from(vec![]),
                ]))
            })
            .collect::<Vec<_>>(),
    )
    .block(Block::default().title("连接详情").borders(Borders::all()));
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
    for d in 1u32..=max_day {
        if let Some(f) = details_group.get(&d) {
            flux += *f;
        }
        details.push((d as f64, flux as f64 / GIGABYTES));
    }
    let flux_g = flux as f64 / GIGABYTES;

    let max_flux = (m
        .flux
        .as_ref()
        .map(|f| f.flux.0 as f64 / GIGABYTES)
        .unwrap_or_default()
        .max(flux_g)
        .max(1.0)
        * 1.1)
        .ceil() as usize;

    let dataset = Dataset::default()
        .marker(tui::symbols::Marker::Dot)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(get_flux_color(flux, true)))
        .data(&details);

    let chart = Chart::new(vec![dataset])
        .x_axis(
            Axis::default()
                .title(Span::from("日期"))
                .bounds([1.0, max_day as f64])
                .labels(
                    (1..=max_day)
                        .into_iter()
                        .map(|d| Span::from(d.to_string()))
                        .collect(),
                ),
        )
        .y_axis(
            Axis::default()
                .title(Span::from("流量"))
                .bounds([0.0, max_flux as f64])
                .labels((0..=max_flux).map(|f| Span::from(f.to_string())).collect()),
        )
        .block(Block::default().title("流量详情").borders(Borders::all()));
    f.render_widget(chart, chunks[1]);

    let key_style = Style::default().bg(Color::Black).fg(Color::White);

    let status = Paragraph::new(Spans::from(vec![
        Span::styled("F1", key_style),
        Span::raw("登录"),
        Span::styled("F2", key_style),
        Span::raw("注销"),
        Span::styled("F3", key_style),
        Span::raw("刷新流量"),
        Span::styled("F4", key_style),
        Span::raw("刷新在线"),
        Span::styled("F5", key_style),
        Span::raw("刷新图表"),
        Span::styled("    ", key_style),
        Span::raw(m.log.as_ref().map(|s| s.as_ref()).unwrap_or("")),
        Span::raw(if m.online { "|正在刷新在线" } else { "" }),
        Span::raw(if m.detail { "|正在刷新图表" } else { "" }),
    ]))
    .block(Block::default().style(Style::default().bg(Color::LightCyan).fg(Color::Black)));
    f.render_widget(status, global_chunks[1]);
}
