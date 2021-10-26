use crate::cui::*;
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

pub fn draw<B: Backend>(m: &Model, f: &mut Frame<B>) {
    let global_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(f.size().height - 1), Constraint::Min(1)])
        .split(f.size());
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(34), Constraint::Percentage(100)])
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
                    flux.online_time.to_string(),
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
                        Span::styled(u.login_time.to_string(), Style::default().fg(Color::Green)),
                    ]),
                    Spans::from(vec![
                        Span::styled("流量     ", subtitle_style),
                        Span::styled(
                            u.flux.to_string(),
                            Style::default().fg(get_flux_color(u.flux.0, true)),
                        ),
                    ]),
                    Spans::from({
                        let mut spans = vec![
                            Span::styled("MAC 地址 ", subtitle_style),
                            Span::styled(
                                u.mac_address.map(|a| a.to_string()).unwrap_or_default(),
                                Style::default().fg(Color::LightCyan),
                            ),
                        ];
                        let is_self = m
                            .mac_addrs
                            .iter()
                            .any(|it| Some(it) == u.mac_address.as_ref());
                        if is_self {
                            spans.push(Span::styled(" 本机", Style::default().fg(Color::Magenta)));
                        }
                        spans
                    }),
                    Spans::default(),
                ]))
            })
            .collect::<Vec<_>>(),
    )
    .block(Block::default().title("连接详情").borders(Borders::all()));
    f.render_widget(table, title_chunks[1]);

    let max_flux = (m.max_flux.to_gb() * 1.1).ceil().max(1.0) as u64;

    let dataset = Dataset::default()
        .marker(tui::symbols::Marker::Dot)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(get_flux_color(m.max_flux.0, true)))
        .data(&m.details);

    let chart = Chart::new(vec![dataset])
        .x_axis(
            Axis::default()
                .title(Span::from(format!("日期/{}月", m.now.month())))
                .bounds([1.0, m.now.day() as f64])
                .labels(
                    (1..=m.now.day())
                        .into_iter()
                        .map(|d| Span::from(d.to_string()))
                        .collect(),
                ),
        )
        .y_axis(
            Axis::default()
                .title(Span::from("流量/GB"))
                .bounds([0.0, max_flux as f64])
                .labels((0..=max_flux).map(|f| Span::from(f.to_string())).collect()),
        )
        .block(Block::default().title("流量详情").borders(Borders::all()));
    f.render_widget(chart, chunks[1]);

    let key_style = Style::default().bg(Color::Black).fg(Color::White);

    let mut spans = vec![
        Span::styled("F1", key_style),
        Span::raw("登录    "),
        Span::styled("F2", key_style),
        Span::raw("注销    "),
        Span::styled("F3", key_style),
        Span::raw("刷新流量"),
        Span::styled("F4", key_style),
        Span::raw("刷新在线"),
        Span::styled("F5", key_style),
        Span::raw("刷新图表"),
        Span::styled("F6", key_style),
        Span::raw("退出    "),
    ];

    if let Some(log) = &m.log {
        spans.push(Span::styled("  ", key_style));
        spans.push(Span::raw(log.as_ref()));
    }
    if m.online {
        spans.push(Span::styled("  ", key_style));
        spans.push(Span::raw("正在刷新在线"));
    }
    if m.detail {
        spans.push(Span::styled("  ", key_style));
        spans.push(Span::raw("正在刷新图表"));
    }

    let status = Paragraph::new(Spans::from(spans))
        .block(Block::default().style(Style::default().bg(Color::LightCyan).fg(Color::Black)));
    f.render_widget(status, global_chunks[1]);
}
