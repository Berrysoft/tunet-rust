use crate::*;
use ratatui::{style::*, Frame};
use tunet_model::*;

fn get_flux_color(flux: u64, total: bool) -> Color {
    if flux == 0 {
        Color::LightCyan
    } else if flux < if total { 20_000_000_000 } else { 2_000_000_000 } {
        Color::LightYellow
    } else {
        Color::LightMagenta
    }
}

pub fn draw(m: &Model, f: &mut Frame) {
    let global_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(f.area().height - 1), Constraint::Min(1)])
        .split(f.area());
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(51), Constraint::Percentage(100)])
        .split(global_chunks[0]);
    let title_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Percentage(100)])
        .split(chunks[0]);

    let subtitle_style = Style::default().fg(Color::Cyan);

    let graph = {
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled("用户 ", subtitle_style),
                Span::styled(
                    &m.flux.username,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("流量 ", subtitle_style),
                Span::styled(
                    m.flux.flux.to_string(),
                    Style::default()
                        .fg(get_flux_color(m.flux.flux.0, true))
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("时长 ", subtitle_style),
                Span::styled(
                    m.flux.online_time.to_string(),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::styled("余额 ", subtitle_style),
                Span::styled(
                    m.flux.balance.to_string(),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
        ])
    }
    .block(Block::default().title("基础信息").borders(Borders::all()));
    f.render_widget(graph, title_chunks[0]);

    let table = List::new(
        m.users
            .iter()
            .map(|u| {
                ListItem::new(Text::from(vec![
                    Line::from(vec![
                        Span::styled("IP地址   ", subtitle_style),
                        Span::styled(
                            u.address.to_string(),
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("IPv6地址 ", subtitle_style),
                        Span::styled(
                            u.address_v6.to_string(),
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("登录时间 ", subtitle_style),
                        Span::styled(u.login_time.to_string(), Style::default().fg(Color::Green)),
                    ]),
                    Line::from(vec![
                        Span::styled("流量     ", subtitle_style),
                        Span::styled(
                            u.flux.to_string(),
                            Style::default().fg(get_flux_color(u.flux.0, true)),
                        ),
                    ]),
                    Line::from({
                        let mut spans = vec![
                            Span::styled("MAC地址  ", subtitle_style),
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
                    Line::default(),
                ]))
            })
            .collect::<Vec<_>>(),
    )
    .block(Block::default().title("连接详情").borders(Borders::all()));
    f.render_widget(table, title_chunks[1]);

    let DetailDaily {
        details,
        now,
        max_flux: max,
    } = DetailDaily::new(&m.details);
    let details = details
        .into_iter()
        .map(|(d, f)| (d.day() as f64, f.to_gb()))
        .collect::<Vec<_>>();

    let max_flux = (max.to_gb() * 1.1).ceil().max(1.0) as u64;

    let dataset = Dataset::default()
        .marker(ratatui::symbols::Marker::Dot)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(get_flux_color(max.0, true)))
        .data(&details);

    let chart = Chart::new(vec![dataset])
        .x_axis(
            Axis::default()
                .title(Span::from(format!("日期/{}月", now.month())))
                .bounds([1.0, now.day() as f64])
                .labels((1..=now.day()).map(|d| Span::from(d.to_string()))),
        )
        .y_axis(
            Axis::default()
                .title(Span::from("流量/GB"))
                .bounds([0.0, max_flux as f64])
                .labels((0..=max_flux).map(|f| Span::from(f.to_string()))),
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

    let log = &m.log;
    if !log.is_empty() {
        spans.push(Span::styled("  ", key_style));
        spans.push(Span::raw(log.as_ref()));
    }
    if m.online_busy() {
        spans.push(Span::styled("  ", key_style));
        spans.push(Span::raw("正在刷新在线"));
    }
    if m.detail_busy() {
        spans.push(Span::styled("  ", key_style));
        spans.push(Span::raw("正在刷新图表"));
    }

    let status = Paragraph::new(Line::from(spans))
        .block(Block::default().style(Style::default().bg(Color::LightCyan).fg(Color::Black)));
    f.render_widget(status, global_chunks[1]);
}
