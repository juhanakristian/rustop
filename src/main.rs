use crossterm::event::{self, KeyCode};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    DefaultTerminal, Frame,
};
use std::cmp::Ordering::Equal;
use std::time::Duration;
use sysinfo::{Process, ProcessesToUpdate, System};

enum SortBy {
    Cpu,
    Mem,
    Pid,
    Name,
}

pub struct App {
    sys: System,
    table_state: TableState,
    sort: SortBy,
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
            sort: SortBy::Cpu,
        }
    }

    fn select_next(&mut self) {
        self.table_state.select_next();
    }

    fn select_previous(&mut self) {
        self.table_state.select_previous();
    }

    fn next_sort_type(&mut self) {
        match self.sort {
            SortBy::Cpu => self.sort = SortBy::Mem,
            SortBy::Mem => self.sort = SortBy::Pid,
            SortBy::Pid => self.sort = SortBy::Name,
            SortBy::Name => self.sort = SortBy::Cpu,
        }
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
            render(frame, area, &app.sys, &mut app.table_state, &app.sort);
        })?;

        if event::poll(Duration::from_millis(200))? && let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Char('j') | KeyCode::Down => app.select_next(),
                KeyCode::Char('k') | KeyCode::Down => app.select_previous(),
                KeyCode::Char('s') | KeyCode::Down => app.next_sort_type(),
                _ => {}
            }
        }

        app.refresh();
    }
}

fn render(
    frame: &mut Frame,
    area: Rect,
    sys: &System,
    table_state: &mut TableState,
    sort_by: &SortBy,
) {
    let header_cells = ["PID", "Name", "CPU%", "Mem (KB)"].map(|h| {
        let label = match (sort_by, h) {
            (SortBy::Cpu, "CPU%") => format!("CPU% ▼"),
            (SortBy::Mem, "Mem (KB)") => format!("Mem (KB) ▼"),
            (SortBy::Pid, "PID") => format!("PID ▼"),
            (SortBy::Name, "Name") => format!("Name ▼"),
            _ => h.to_string(),
        };

        Cell::from(label).style(Style::default().bold())
    });

    let header = Row::new(header_cells)
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let mut processes: Vec<&Process> = sys.processes().values().collect();
    match sort_by {
        SortBy::Cpu => {
            processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap_or(Equal))
        }
        SortBy::Mem => processes.sort_by(|a, b| b.memory().cmp(&a.memory())),
        SortBy::Pid => processes.sort_by(|a, b| a.pid().cmp(&b.pid())),
        SortBy::Name => processes.sort_by(|a, b| a.name().cmp(b.name())),
    }

    let rows: Vec<Row> = processes
        .into_iter()
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

    frame.render_stateful_widget(table, area, table_state);
}
