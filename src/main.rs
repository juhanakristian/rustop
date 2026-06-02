use crossterm::event::{self, KeyCode};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, TableState},
    DefaultTerminal, Frame,
};
use std::time::Duration;
use sysinfo::{ProcessesToUpdate, System};

pub struct App {
    sys: System,
    table_state: TableState,
}

impl App {
    fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);

        sys.refresh_all();

        Self {
            sys,
            table_state: TableState::default().with_selected(0),
        }
    }

    fn select_next(&mut self) {
        self.table_state.select_next();
    }

    fn select_previous(&mut self) {
        self.table_state.select_previous();
    }

    fn refresh(&mut self) {
        self.sys.refresh_processes(ProcessesToUpdate::All, true);
        self.sys.refresh_cpu_all();
        self.sys.refresh_memory();
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

        if event::poll(Duration::from_millis(100))? && let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Char('j') | KeyCode::Down => app.select_next(),
                KeyCode::Char('k') | KeyCode::Up => app.select_previous(),
                _ => {}
            }
        }

        app.refresh();
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
