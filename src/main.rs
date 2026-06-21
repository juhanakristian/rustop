use core::{cmp::Ordering, fmt, ptr::null};
use crossterm::event::{self, KeyCode};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    DefaultTerminal, Frame,
};
use std::cmp::Ordering::Equal;
use std::time::Duration;
use sysinfo::{Process, ProcessesToUpdate, System};

pub struct RustopProcess {
    cpu: f32,
    mem: u64,
    pid: Option<String>,
    name: String,
}

enum SortBy {
    Cpu,
    Mem,
    Pid,
    Name,
}

impl fmt::Display for SortBy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SortBy::Cpu => write!(f, "CPU"),
            SortBy::Mem => write!(f, "Memory"),
            SortBy::Pid => write!(f, "PID"),
            SortBy::Name => write!(f, "Name"),
        }
    }
}

pub struct App {
    sys: System,
    table_state: TableState,
    sort: SortBy,
    grouping: bool,
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
            grouping: false,
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

        if event::poll(Duration::from_millis(500))? && let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Char('j') | KeyCode::Down => app.select_next(),
                KeyCode::Char('k') | KeyCode::Down => app.select_previous(),
                KeyCode::Char('s') | KeyCode::Down => app.next_sort_type(),
                KeyCode::Char('g') | KeyCode::Down => app.grouping = !app.grouping,
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
    use Constraint::{Fill, Length, Min};
    let vertical = Layout::vertical([Length(1), Min(0), Length(3)]);
    let [title_area, main_area, status_area] = vertical.areas(area);

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

    let processes: Vec<&Process> = sys.processes().values().collect();

    let mut rustop_processes = grouped_processes(convert_to_rustopprocess(processes));

    match sort_by {
        SortBy::Cpu => rustop_processes.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(Equal)),
        SortBy::Mem => rustop_processes.sort_by(|a, b| b.mem.cmp(&a.mem)),
        SortBy::Pid => rustop_processes.sort_by(|a, b| a.pid.cmp(&b.pid)),
        SortBy::Name => rustop_processes.sort_by(|a, b| a.name.cmp(&b.name)),
    }

    let rows: Vec<Row> = rustop_processes
        .into_iter()
        .map(|p| {
            Row::new(vec![
                p.pid.map_or_else(|| "-".to_string(), |pid| pid.to_string()),
                p.name.clone(),
                format!("{:.1}", p.cpu),
                format!("{}", p.mem / 1024),
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

    frame.render_widget(Block::default().title("rustop"), title_area);
    frame.render_stateful_widget(table, main_area, table_state);

    let text = format!(" Sort: {} ", sort_by,);

    let footer = Paragraph::new(text).style(Style::default());

    frame.render_widget(footer, status_area);
}

fn convert_to_rustopprocess(processes: Vec<&Process>) -> Vec<RustopProcess> {
    let mut result: Vec<RustopProcess> = vec![];
    for process in processes {
        result.push(RustopProcess {
            name: process.name().to_str().unwrap_or("").to_string(),
            cpu: process.cpu_usage(),
            mem: process.memory(),
            pid: Some(process.pid().to_string()),
        })
    }

    return result;
}

fn grouped_processes(processes: Vec<RustopProcess>) -> Vec<RustopProcess> {
    let mut result: Vec<RustopProcess> = vec![];

    for process in processes {
        if let Some(idx) = result.iter().position(|p| p.name == process.name) {
            result[idx].cpu += process.cpu;
            result[idx].mem += process.mem;
        } else {
            result.push(RustopProcess {
                name: process.name,
                cpu: process.cpu,
                mem: process.mem,
                pid: None,
            })
        }
    }

    return result;
}
