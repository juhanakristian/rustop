use ratatui::{DefaultTerminal, Frame, layout::{Rect, Constraint}, widgets::{Block, Borders, Row, Table}, style::{Color, Modifier, Style}};
use sysinfo::{System};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    ratatui::run(app)?;
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut sys = System::new_all();
    sys.refresh_all();

    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            render(frame, area, &sys);
        })?;
        if crossterm::event::read()?.is_key_press() {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame, area: Rect, sys: &System) {
    let header = Row::new(vec!["PID", "Name", "CPU%", "Mem (KB)"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = sys.processes().values().map(|p| {
        Row::new(vec![
            p.pid().to_string(),
            p.name().to_string_lossy().to_string(),
            format!("{:.1}", p.cpu_usage()),
            format!("{}", p.memory() / 1024),
        ])
    }).collect();

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
    // frame.render_stateful_widget(table, area, &mut state);
    frame.render_widget(table, area);
}
