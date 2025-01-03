use crate::{
    app::{App, InputMode},
    utils::{
        centered_rect, create_block, get_notification_title, pad_vertical, MIN_HEIGHT, MIN_WIDTH,
    },
};
use ratatui::{
    prelude::*,
    widgets::{Clear, Paragraph},
};

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        let text = vec![
            Line::from("Terminal size too small:"),
            Line::from(format!("Width = {} Height = {}", area.width, area.height)),
            Line::from(""),
            Line::from("Needed to display properly:"),
            Line::from(format!("Width = {} Height = {}", MIN_WIDTH, MIN_HEIGHT)),
        ];

        let padded_text = pad_vertical(text, area.height);
        let warning = Paragraph::new(padded_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::LightCyan));

        frame.render_widget(warning, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)].as_ref())
        .split(area);

    draw_main_block(frame, app, chunks[0]);
    draw_help_block(frame, chunks[1]);
    draw_popups(frame, app, area);
}

fn draw_main_block(frame: &mut Frame, app: &App, area: Rect) {
    let title = get_notification_title(&app.error_message, app.copy_notification_time);
    let main_block = create_block(&title);
    let entries: Vec<Line> = app
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let style = if i == app.selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            let (code, remaining) = entry
                .generate_totp_with_time()
                .unwrap_or_else(|_| ("Invalid".to_string(), 0));

            Line::styled(
                format!("{:<30} {:>6} ({:>1}s)", entry.name, code, remaining),
                style,
            )
        })
        .collect();

    let main_widget = Paragraph::new(entries)
        .block(main_block)
        .alignment(Alignment::Left);

    frame.render_widget(main_widget, area);
}

fn draw_help_block(frame: &mut Frame, area: Rect) {
    let help_block = create_block(" Bindings ");
    let help_text = vec![Line::from(
        "a: add  E: edit  d: del  i: import  e: export  ↑/k: up  ↓/j: down  enter: copy  q: quit",
    )];

    let help_widget = Paragraph::new(help_text)
        .block(help_block)
        .alignment(Alignment::Center);

    frame.render_widget(help_widget, area);
}

fn draw_popups(frame: &mut Frame, app: &App, area: Rect) {
    match app.input_mode {
        InputMode::Adding => {
            let popup_block = create_block(" Add Entry ");
            let area = centered_rect(60, 20, area);
            let popup = Paragraph::new(vec![
                Line::from("Name:"),
                Line::from(format!(
                    "{}{}",
                    app.new_entry_name.as_str(),
                    if app.input_field == 0 { "|" } else { "" }
                )),
                Line::from(""),
                Line::from("Secret:"),
                Line::from(format!(
                    "{}{}",
                    app.new_entry_secret.as_str(),
                    if app.input_field == 1 { "|" } else { "" }
                )),
            ])
            .block(popup_block);

            frame.render_widget(Clear, area);
            frame.render_widget(popup, area);
        }
        InputMode::Importing | InputMode::Exporting => {
            let title = match app.input_mode {
                InputMode::Importing => " Import ",
                InputMode::Exporting => " Export ",
                _ => unreachable!(),
            };

            let popup_block = create_block(title);
            let area = centered_rect(60, 20, area);
            let popup = Paragraph::new(vec![
                Line::from("Path:"),
                Line::from(format!("{}|", app.path_input.as_str())),
            ])
            .block(popup_block);

            frame.render_widget(Clear, area);
            frame.render_widget(popup, area);
        }
        InputMode::Editing => {
            let popup_block = create_block(" Edit Entry ");
            let area = centered_rect(60, 20, area);
            let popup = Paragraph::new(vec![
                Line::from("Name:"),
                Line::from(format!(
                    "{}{}",
                    app.edit_entry_name.as_str(),
                    if app.input_field == 0 { "|" } else { "" }
                )),
                Line::from(""),
                Line::from("Secret:"),
                Line::from(format!(
                    "{}{}",
                    app.edit_entry_secret.as_str(),
                    if app.input_field == 1 { "|" } else { "" }
                )),
            ])
            .block(popup_block);

            frame.render_widget(Clear, area);
            frame.render_widget(popup, area);
        }
        _ => {}
    }
}
