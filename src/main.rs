use std::{env, io};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use xfercat::{
    app::{Action, App, Screen},
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
            "usage: xfercat [--snapshot connections|profile-add|profile-edit|workspace|rename|review]",
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
        if let Some(action) = action_for(app.screen, key.code)
            && app.update(action)
        {
            return Ok(());
        }
    }
}

fn action_for(screen: Screen, code: KeyCode) -> Option<Action> {
    if screen == Screen::ProfileEditor {
        return match code {
            KeyCode::Enter => Some(Action::Activate),
            KeyCode::Esc => Some(Action::Back),
            KeyCode::Tab | KeyCode::Down => Some(Action::NextProfileField),
            KeyCode::BackTab | KeyCode::Up => Some(Action::PreviousProfileField),
            KeyCode::Left | KeyCode::Right => Some(Action::ToggleProfileAuthentication),
            KeyCode::Backspace => Some(Action::BackspaceProfile),
            KeyCode::Char(character) => Some(Action::InputProfileChar(character)),
            _ => None,
        };
    }

    if screen == Screen::Rename {
        return match code {
            KeyCode::Enter => Some(Action::Activate),
            KeyCode::Esc => Some(Action::Back),
            KeyCode::Backspace => Some(Action::BackspaceRename),
            KeyCode::Char(character) => Some(Action::InputRenameChar(character)),
            _ => None,
        };
    }

    match code {
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Up | KeyCode::Char('k') => Some(Action::Up),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::Down),
        KeyCode::Enter => Some(Action::Activate),
        KeyCode::Esc => Some(Action::Back),
        KeyCode::Tab => Some(Action::NextFocus),
        KeyCode::Char('a' | 'A') => Some(Action::AddProfile),
        KeyCode::Char('e' | 'E') => Some(Action::EditProfile),
        KeyCode::Char(' ') => Some(Action::AddToPlan),
        KeyCode::Char('d') => Some(Action::RemovePlanItem),
        KeyCode::Char('p') => Some(Action::CycleConflictPolicy),
        KeyCode::Char('n' | 'N') => Some(Action::BeginRename),
        KeyCode::Char('K') => Some(Action::MovePlanUp),
        KeyCode::Char('J') => Some(Action::MovePlanDown),
        KeyCode::Char('r') => Some(Action::ReviewPlan),
        _ => None,
    }
}
