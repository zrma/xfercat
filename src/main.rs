use std::{env, io};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use xfercat::{
    app::{Action, App, Screen},
    openssh, ui,
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
            "usage: xfercat [--snapshot connections|openssh|openssh-empty|profile-add|profile-edit|workspace|rename|review]",
        ));
    }

    let discovery = openssh::discover_home();
    let app = App::runtime(discovery.profiles(), discovery.status());
    ratatui::run(move |terminal| run_interactive(terminal, app))
}

fn run_interactive(terminal: &mut ratatui::DefaultTerminal, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, &mut app))?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        if let Some(action) = action_for(app.screen, key.code) {
            if action == Action::RefreshOpenSshProfiles {
                let discovery = openssh::discover_home();
                app.refresh_open_ssh_profiles(discovery.profiles(), discovery.status());
            } else if app.update(action) {
                return Ok(());
            }
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
        KeyCode::Char('i' | 'I') if screen == Screen::Connections => {
            Some(Action::RefreshOpenSshProfiles)
        }
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
