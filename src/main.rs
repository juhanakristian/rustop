use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, TableState},
    DefaultTerminal, Frame,
};
use sysinfo::System;

pub struct App {
    sys: System,
    table_state: TableState,
}

impl App {
    fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        Self {
            sys,
            table_state: TableState::default().with_selected(0),
        }
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(app)?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut app = App::new();
    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            render(frame, area, &app.sys, &mut app.table_state);
        })?;
        if crossterm::event::read()?.is_key_press() {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame, area: Rect, sys: &System, table_state: &mut TableState) {
    let header = Row::new(vec!["PID", "Name", "CPU%", "Mem (KB)"])
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = sys
        .processes()
        .values()
        .map(|p| {
            Row::new(vec![
                p.pid().to_string(),
                p.name().to_string_lossy().to_string(),
                format!("{:.1}", p.cpu_usage()),
                format!("{}", p.memory() / 1024),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(8),
        Constraint::Min(20),
        Constraint::Length(8),
        Constraint::Length(12),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().title("Processes").borders(Borders::ALL))
        .row_highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol(">> ");

    // let mut state = TableState::default();
    frame.render_stateful_widget(table, area, table_state);
    // frame.render_widget(table, area);
}
