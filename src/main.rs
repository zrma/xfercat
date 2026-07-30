use std::{env, io};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use xfercat::{
    app::{Action, App},
    ui,
};

fn main() -> io::Result<()> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    if arguments.first().map(String::as_str) == Some("--snapshot") {
        let kind = arguments.get(1).map(String::as_str).unwrap_or("workspace");
        print!("{}", ui::snapshot(kind)?);
        return Ok(());
    }
    if !arguments.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: xfercat [--snapshot connections|workspace|review]",
        ));
    }

    ratatui::run(run_interactive)
}

fn run_interactive(terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
    let mut app = App::demo();

    loop {
        terminal.draw(|frame| ui::render(frame, &mut app))?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        if let Some(action) = action_for(key.code)
            && app.update(action)
        {
            return Ok(());
        }
    }
}

fn action_for(code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Up | KeyCode::Char('k') => Some(Action::Up),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::Down),
        KeyCode::Enter => Some(Action::Activate),
        KeyCode::Esc => Some(Action::Back),
        KeyCode::Tab => Some(Action::NextFocus),
        KeyCode::Char('e') => Some(Action::EditProfile),
        KeyCode::Char(' ') => Some(Action::AddToPlan),
        KeyCode::Char('d') => Some(Action::RemovePlanItem),
        KeyCode::Char('p') => Some(Action::CycleConflictPolicy),
        KeyCode::Char('r') => Some(Action::ReviewPlan),
        _ => None,
    }
}
