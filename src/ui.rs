use std::io;

use ratatui::{
    Frame, Terminal,
    backend::TestBackend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::{
    app::{Action, App, Focus, ProfileAuthentication, ProfileField, Screen},
    domain::{BrowserEntry, TransferPlanItem},
};

pub fn render(frame: &mut Frame<'_>, app: &mut App) {
    match app.screen {
        Screen::Connections => render_connections(frame, app),
        Screen::ProfileEditor => render_profile_editor(frame, app),
        Screen::Workspace => render_workspace(frame, app),
        Screen::Rename => {
            render_workspace(frame, app);
            render_rename(frame, app);
        }
        Screen::Review => {
            render_workspace(frame, app);
            render_review(frame, app);
        }
    }
}

pub fn snapshot(kind: &str) -> io::Result<String> {
    snapshot_at(kind, 110, 32)
}

pub fn snapshot_at(kind: &str, width: u16, height: u16) -> io::Result<String> {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("TestBackend initialization is infallible");
    let mut app = App::demo();

    match kind {
        "connections" => {}
        "profile-add" => {
            app.update(Action::AddProfile);
        }
        "profile-edit" => {
            app.update(Action::EditProfile);
        }
        "workspace" | "rename" | "review" => {
            app.update(Action::Activate);
            app.update(Action::AddToPlan);
            app.update(Action::NextFocus);
            app.update(Action::AddToPlan);
            app.update(Action::NextFocus);
            if matches!(kind, "rename" | "review") {
                app.update(Action::BeginRename);
                app.rename_buffer = "service-copy.log".into();
            }
            if kind == "review" {
                app.update(Action::Activate);
                app.update(Action::MovePlanUp);
                app.update(Action::ReviewPlan);
            }
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "snapshot must be connections, profile-add, profile-edit, workspace, rename, or review",
            ));
        }
    }

    terminal
        .draw(|frame| render(frame, &mut app))
        .expect("TestBackend drawing is infallible");
    Ok(buffer_text(terminal.backend()))
}

fn render_connections(frame: &mut Frame<'_>, app: &mut App) {
    let [title, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(4),
    ])
    .areas(frame.area());

    frame.render_widget(
        Paragraph::new("xfercat · Connections")
            .alignment(Alignment::Center)
            .block(Block::bordered()),
        title,
    );

    let [list_area, detail_area] =
        Layout::horizontal([Constraint::Percentage(58), Constraint::Percentage(42)]).areas(body);

    let profiles = app
        .profiles
        .iter()
        .map(|profile| {
            ListItem::new(vec![
                Line::from(format!(
                    "{:<12} {:<5} {}",
                    profile.label,
                    profile.protocol,
                    profile.authentication.short_label()
                )),
                Line::styled(
                    format!("  {}@{}", profile.user, profile.host),
                    Style::default().fg(Color::DarkGray),
                ),
            ])
        })
        .collect::<Vec<_>>();
    let mut state = ListState::default();
    state.select(Some(app.selected_profile));
    frame.render_stateful_widget(
        List::new(profiles)
            .block(Block::new().title("[CONNECTIONS]").borders(Borders::ALL))
            .highlight_symbol("▶ ")
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        list_area,
        &mut state,
    );

    let profile = &app.profiles[app.selected_profile];
    let detail = vec![
        Line::from(vec![
            Span::styled("Profile    ", Style::default().fg(Color::DarkGray)),
            Span::raw(&profile.label),
        ]),
        Line::from(vec![
            Span::styled("Protocol   ", Style::default().fg(Color::DarkGray)),
            Span::raw(profile.protocol.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Endpoint   ", Style::default().fg(Color::DarkGray)),
            Span::raw(format!("{}@{}", profile.user, profile.host)),
        ]),
        Line::from(vec![
            Span::styled("Auth       ", Style::default().fg(Color::DarkGray)),
            Span::raw(profile.authentication.to_string()),
        ]),
        Line::from(""),
        Line::styled(
            "Selecting a row never changes its protocol or auth.",
            Style::default().fg(Color::Yellow),
        ),
    ];
    frame.render_widget(
        Paragraph::new(detail)
            .wrap(Wrap { trim: true })
            .block(Block::new().title("[PROFILE]").borders(Borders::ALL)),
        detail_area,
    );

    render_footer(
        frame,
        footer,
        &["↑/↓ Move   Enter Connect   A Add   E Edit   Q Quit"],
        &app.status,
    );
}

fn render_profile_editor(frame: &mut Frame<'_>, app: &App) {
    let Some(editor) = app.profile_editor.as_ref() else {
        return;
    };
    let [title, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(5),
    ])
    .areas(frame.area());

    let operation = if editor.is_create() { "Add" } else { "Edit" };
    frame.render_widget(
        Paragraph::new(format!("xfercat · {operation} profile"))
            .alignment(Alignment::Center)
            .block(Block::bordered()),
        title,
    );

    let authentication = format!("← {} →", editor.authentication.label());
    let mut fields = vec![
        profile_field_line("Label", &editor.label, editor.field == ProfileField::Label),
        profile_field_line("Protocol", "SFTP (fixed)", false),
        profile_field_line("User", &editor.user, editor.field == ProfileField::User),
        profile_field_line("Host", &editor.host, editor.field == ProfileField::Host),
        profile_field_line(
            "Authentication",
            &authentication,
            editor.field == ProfileField::Authentication,
        ),
    ];
    if editor.authentication == ProfileAuthentication::KeyReference {
        fields.push(profile_field_line(
            "Key reference",
            &editor.key_reference,
            editor.field == ProfileField::KeyReference,
        ));
    }
    fields.extend([
        Line::from(""),
        Line::styled(
            "Profile changes are saved only for this process.",
            Style::default().fg(Color::Yellow),
        ),
        Line::from("Credential content and private keys are never stored here."),
    ]);

    frame.render_widget(
        Paragraph::new(fields)
            .wrap(Wrap { trim: false })
            .block(Block::new().title("[PROFILE FORM]").borders(Borders::ALL)),
        body,
    );
    render_footer(
        frame,
        footer,
        &[
            "Tab/Shift+Tab Field   Type to edit",
            "←/→ Authentication   Enter Save   Esc Cancel",
        ],
        &app.status,
    );
}

fn profile_field_line(label: &str, value: &str, active: bool) -> Line<'static> {
    let (marker, label_style, value_style) = if active {
        (
            "▶ ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::UNDERLINED),
        )
    } else {
        ("  ", Style::default().fg(Color::DarkGray), Style::default())
    };
    Line::from(vec![
        Span::styled(format!("{marker}{label:<16}"), label_style),
        Span::styled(value.to_owned(), value_style),
    ])
}

fn render_workspace(frame: &mut Frame<'_>, app: &mut App) {
    let [browsers, waybill, footer] = Layout::vertical([
        Constraint::Percentage(48),
        Constraint::Min(10),
        Constraint::Length(6),
    ])
    .areas(frame.area());
    let [local, remote] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(browsers);

    render_browser(
        frame,
        local,
        "[LOCAL] /workspace/outgoing",
        &app.local_entries,
        app.local_selection,
        app.focus == Focus::Local,
    );
    let remote_title = app
        .connected_profile()
        .map(|profile| format!("[REMOTE] {}:/srv/xfercat", profile.label))
        .unwrap_or_else(|| "[REMOTE] disconnected".into());
    render_browser(
        frame,
        remote,
        &remote_title,
        &app.remote_entries,
        app.remote_selection,
        app.focus == Focus::Remote,
    );
    render_waybill(frame, waybill, app);
    render_footer(
        frame,
        footer,
        &[
            "Tab Focus   ↑/↓ Move   Space Add   D Remove",
            "N Rename   Shift+K/J Reorder   P Policy",
            "R Review   Esc Connections   Q Quit",
        ],
        &app.status,
    );
}

fn render_browser(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    entries: &[BrowserEntry],
    selection: usize,
    active: bool,
) {
    let items = entries
        .iter()
        .map(|entry| {
            let size = entry.size.map(format_size).unwrap_or_else(|| "—".into());
            let name_width = usize::from(area.width.saturating_sub(16)).max(8);
            ListItem::new(format!(
                "{:<name_width$} {:>10}",
                entry.display_name(),
                size
            ))
        })
        .collect::<Vec<_>>();
    let mut state = ListState::default();
    state.select(Some(selection));
    frame.render_stateful_widget(
        List::new(items)
            .block(active_block(title, active))
            .highlight_symbol("▶ ")
            .highlight_style(Style::default().fg(Color::Cyan)),
        area,
        &mut state,
    );
}

fn render_waybill(frame: &mut Frame<'_>, area: Rect, app: &mut App) {
    let title = format!("[WAYBILL] {} item(s) · preview only", app.plan.len());
    let items = if app.plan.is_empty() {
        vec![ListItem::new(vec![
            Line::styled("No staged transfers.", Style::default().fg(Color::DarkGray)),
            Line::from("Focus LOCAL or REMOTE and press Space."),
        ])]
    } else {
        app.plan.iter().map(waybill_item).collect()
    };
    let mut state = ListState::default();
    if !app.plan.is_empty() {
        state.select(Some(app.plan_selection));
    }
    frame.render_stateful_widget(
        List::new(items)
            .block(active_block(&title, app.focus == Focus::Waybill))
            .highlight_symbol("▶ ")
            .highlight_style(Style::default().fg(Color::Cyan)),
        area,
        &mut state,
    );
}

fn waybill_item(item: &TransferPlanItem) -> ListItem<'static> {
    ListItem::new(vec![
        Line::from(format!(
            "#{} {} {}",
            item.id,
            item.direction.symbol(),
            item.source.display()
        )),
        Line::from(vec![
            Span::raw(format!("   → {}  ", item.destination.display())),
            Span::styled(
                format!("[{}] [{}]", item.conflict_policy, item.state),
                Style::default().fg(Color::Yellow),
            ),
        ]),
    ])
}

fn render_review(frame: &mut Frame<'_>, app: &App) {
    let area = centered_rect(88, 76, frame.area());
    frame.render_widget(Clear, area);

    let mut lines = vec![
        Line::styled(
            "DRY-RUN TRANSFER REVIEW",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Line::from(""),
    ];
    for item in &app.plan {
        lines.push(Line::from(format!(
            "#{} {} {}",
            item.id,
            item.direction.symbol(),
            item.source.display()
        )));
        lines.push(Line::from(format!(
            "   → {}  [{}] [{}]",
            item.destination.display(),
            item.conflict_policy,
            item.state
        )));
    }
    lines.extend([
        Line::from(""),
        Line::styled(
            "PoC only: no transport adapter is connected.",
            Style::default().fg(Color::Yellow),
        ),
        Line::from("Enter Accept review   Esc Back"),
    ]);

    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(Block::new().title("[REVIEW]").borders(Borders::ALL)),
        area,
    );
}

fn render_rename(frame: &mut Frame<'_>, app: &App) {
    let area = centered_rect(80, 42, frame.area());
    frame.render_widget(Clear, area);

    let destination = app
        .plan
        .get(app.plan_selection)
        .map(|item| item.destination.display())
        .unwrap_or_else(|| "missing Waybill item".into());
    frame.render_widget(
        Paragraph::new(vec![
            Line::styled(
                "DESTINATION FILENAME",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from(""),
            Line::from(format!("Current  {destination}")),
            Line::from(vec![
                Span::raw("New      "),
                Span::styled(
                    &app.rename_buffer,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::UNDERLINED),
                ),
            ]),
            Line::from(""),
            Line::from("Enter Apply   Esc Cancel"),
        ])
        .wrap(Wrap { trim: false })
        .block(Block::new().title("[RENAME]").borders(Borders::ALL)),
        area,
    );
}

fn active_block<'a>(title: &'a str, active: bool) -> Block<'a> {
    let style = if active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    Block::new()
        .title(title)
        .borders(Borders::ALL)
        .border_style(style)
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, keys: &[&str], status: &str) {
    let mut lines = keys
        .iter()
        .map(|keys| Line::styled(*keys, Style::default().fg(Color::DarkGray)))
        .collect::<Vec<_>>();
    lines.push(Line::from(status));
    frame.render_widget(Paragraph::new(lines).block(Block::bordered()), area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);
    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

fn buffer_text(backend: &TestBackend) -> String {
    let buffer = backend.buffer();
    let area = buffer.area;
    let mut output = String::new();

    for y in area.top()..area.bottom() {
        let mut line = String::new();
        for x in area.left()..area.right() {
            line.push_str(buffer[(x, y)].symbol());
        }
        output.push_str(line.trim_end());
        output.push('\n');
    }

    output
}
