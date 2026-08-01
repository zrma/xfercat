use std::{env, io, path::PathBuf};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use xfercat::{
    app::{Action, App, Focus, Screen},
    domain::ConnectionProfile,
    localfs, openssh,
    sftp::{ConnectionOptions, RemoteDirectory, SftpFailure, SftpSession},
    ui,
};

fn main() -> io::Result<()> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    if arguments.first().map(String::as_str) == Some("--snapshot") {
        let kind = arguments.get(1).map(String::as_str).unwrap_or("workspace");
        print!("{}", ui::snapshot(kind)?);
        return Ok(());
    }
    let connection_options = match arguments.as_slice() {
        [] => ConnectionOptions::default(),
        [flag, path] if flag == "--ssh-config" => ConnectionOptions {
            config_file: Some(PathBuf::from(path)),
            ..ConnectionOptions::default()
        },
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "usage: xfercat [--ssh-config <path>] | [--snapshot connections|openssh|openssh-empty|profile-add|profile-edit|workspace|rename|review|results]",
            ));
        }
    };
    let discovery = discover_profiles(&connection_options);
    let discovery_status = discovery.status();
    let mut app = App::runtime(discovery.profiles(), discovery_status.clone());
    match env::current_dir().and_then(localfs::read_directory) {
        Ok(current) => {
            app.replace_local_directory(current);
            app.status = discovery_status;
        }
        Err(_) => {
            app.local_directory = "<unavailable>".into();
            app.local_entries.clear();
            app.local_selection = 0;
            app.status = format!("{discovery_status} Local directory unavailable.");
        }
    }
    ratatui::run(move |terminal| run_interactive(terminal, app, connection_options))
}

fn run_interactive(
    terminal: &mut ratatui::DefaultTerminal,
    mut app: App,
    connection_options: ConnectionOptions,
) -> io::Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let mut session = None;
    loop {
        terminal.draw(|frame| ui::render(frame, &mut app))?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        if let Some(action) = action_for(app.screen, key.code) {
            if action == Action::Quit {
                close_session(&runtime, &mut session);
                return Ok(());
            } else if action == Action::RefreshOpenSshProfiles {
                let discovery = discover_profiles(&connection_options);
                app.refresh_open_ssh_profiles(discovery.profiles(), discovery.status());
            } else if action == Action::Activate && app.screen == Screen::Connections {
                let Some(profile) = app.profiles.get(app.selected_profile).cloned() else {
                    app.status = "No profile selected; refresh OpenSSH or add one manually.".into();
                    continue;
                };
                app.status = "Connecting with strict SSH host verification...".into();
                terminal.draw(|frame| ui::render(frame, &mut app))?;
                match runtime.block_on(connect_and_read_home(&profile, &connection_options)) {
                    Ok((connected, directory)) => {
                        close_session(&runtime, &mut session);
                        app.enter_remote_workspace(&profile.id, directory);
                        session = Some(connected);
                    }
                    Err(error) => app.status = error.status().into(),
                }
            } else if matches!(action, Action::Activate | Action::NavigateParent)
                && app.screen == Screen::Workspace
            {
                let parent = action == Action::NavigateParent;
                match app.focus {
                    Focus::Local => {
                        if let Some(target) = app.local_navigation_target(parent) {
                            match localfs::read_directory(target) {
                                Ok(directory) => app.replace_local_directory(directory),
                                Err(_) => {
                                    app.status = "Local directory could not be opened.".into()
                                }
                            }
                        } else if !parent {
                            app.status = "Selected local file; press Space to stage it.".into();
                        }
                    }
                    Focus::Remote => {
                        if let Some(target) = app.remote_navigation_target(parent) {
                            let Some(connected) = session.as_ref() else {
                                app.status = "Remote session is not available.".into();
                                continue;
                            };
                            match runtime.block_on(connected.read_directory(target)) {
                                Ok(directory) => app.replace_remote_directory(directory),
                                Err(error) => app.status = error.status().into(),
                            }
                        } else if !parent {
                            app.status = "Selected remote file; press Space to stage it.".into();
                        }
                    }
                    Focus::Waybill => {
                        app.update(action);
                    }
                }
            } else {
                if action == Action::Back && app.screen == Screen::Workspace {
                    close_session(&runtime, &mut session);
                }
                app.update(action);
            }
        }
    }
}

fn discover_profiles(options: &ConnectionOptions) -> openssh::OpenSshDiscovery {
    let Some(path) = options.config_file.as_deref() else {
        return openssh::discover_home();
    };
    let parent = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let home = if parent.file_name().is_some_and(|name| name == ".ssh") {
        parent.parent().unwrap_or(parent)
    } else {
        parent
    };
    openssh::discover_from_path(path, home)
}

async fn connect_and_read_home(
    profile: &ConnectionProfile,
    options: &ConnectionOptions,
) -> Result<(SftpSession, RemoteDirectory), SftpFailure> {
    let session = SftpSession::connect_with_options(profile, options).await?;
    match session.home_directory().await {
        Ok(directory) => Ok((session, directory)),
        Err(error) => {
            let _ = session.close().await;
            Err(error)
        }
    }
}

fn close_session(runtime: &tokio::runtime::Runtime, session: &mut Option<SftpSession>) {
    if let Some(session) = session.take() {
        let _ = runtime.block_on(session.close());
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
        KeyCode::Backspace if screen == Screen::Workspace => Some(Action::NavigateParent),
        KeyCode::Char('i' | 'I') if screen == Screen::Connections => {
            Some(Action::RefreshOpenSshProfiles)
        }
        KeyCode::Char('a' | 'A') => Some(Action::AddProfile),
        KeyCode::Char('e' | 'E') => Some(Action::EditProfile),
        KeyCode::Char('d' | 'D') if screen == Screen::Connections => Some(Action::DeleteProfile),
        KeyCode::Char(' ') => Some(Action::AddToPlan),
        KeyCode::Char('d' | 'D') => Some(Action::RemovePlanItem),
        KeyCode::Char('p') => Some(Action::CycleConflictPolicy),
        KeyCode::Char('n' | 'N') => Some(Action::BeginRename),
        KeyCode::Char('K') => Some(Action::MovePlanUp),
        KeyCode::Char('J') => Some(Action::MovePlanDown),
        KeyCode::Char('r') => Some(Action::ReviewPlan),
        _ => None,
    }
}
