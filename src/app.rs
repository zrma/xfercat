use crate::domain::{
    Authentication, BrowserEntry, ConflictPolicy, ConnectionProfile, Endpoint, EntryKind, Protocol,
    TransferDirection, TransferPlanItem, TransferState,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Screen {
    Connections,
    ProfileDetails,
    Workspace,
    Review,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Focus {
    Local,
    Remote,
    Waybill,
}

impl Focus {
    pub const fn next(self) -> Self {
        match self {
            Self::Local => Self::Remote,
            Self::Remote => Self::Waybill,
            Self::Waybill => Self::Local,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Up,
    Down,
    Activate,
    Back,
    EditProfile,
    NextFocus,
    AddToPlan,
    RemovePlanItem,
    CycleConflictPolicy,
    ReviewPlan,
    Quit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct App {
    pub screen: Screen,
    pub focus: Focus,
    pub profiles: Vec<ConnectionProfile>,
    pub selected_profile: usize,
    pub connected_profile_id: Option<String>,
    pub local_entries: Vec<BrowserEntry>,
    pub remote_entries: Vec<BrowserEntry>,
    pub local_selection: usize,
    pub remote_selection: usize,
    pub plan: Vec<TransferPlanItem>,
    pub plan_selection: usize,
    pub status: String,
    next_plan_id: u64,
}

impl App {
    pub fn demo() -> Self {
        Self {
            screen: Screen::Connections,
            focus: Focus::Local,
            profiles: vec![
                ConnectionProfile {
                    id: "dev-box".into(),
                    label: "dev-box".into(),
                    protocol: Protocol::Sftp,
                    user: "deploy".into(),
                    host: "dev.example".into(),
                    authentication: Authentication::SshAgent,
                },
                ConnectionProfile {
                    id: "archive".into(),
                    label: "archive".into(),
                    protocol: Protocol::Sftp,
                    user: "operator".into(),
                    host: "archive.example".into(),
                    authentication: Authentication::KeyReference("archive-key".into()),
                },
            ],
            selected_profile: 0,
            connected_profile_id: None,
            local_entries: vec![
                BrowserEntry {
                    name: "releases".into(),
                    path: "/workspace/outgoing/releases".into(),
                    kind: EntryKind::Directory,
                    size: None,
                },
                BrowserEntry {
                    name: "app.tar.gz".into(),
                    path: "/workspace/outgoing/app.tar.gz".into(),
                    kind: EntryKind::File,
                    size: Some(438 * 1024 * 1024),
                },
                BrowserEntry {
                    name: "config.yaml".into(),
                    path: "/workspace/outgoing/config.yaml".into(),
                    kind: EntryKind::File,
                    size: Some(2048),
                },
            ],
            remote_entries: vec![
                BrowserEntry {
                    name: "releases".into(),
                    path: "/srv/xfercat/releases".into(),
                    kind: EntryKind::Directory,
                    size: None,
                },
                BrowserEntry {
                    name: "service.log".into(),
                    path: "/srv/xfercat/service.log".into(),
                    kind: EntryKind::File,
                    size: Some(128 * 1024),
                },
            ],
            local_selection: 1,
            remote_selection: 1,
            plan: Vec::new(),
            plan_selection: 0,
            status: "Choose a complete profile; editing is a separate action.".into(),
            next_plan_id: 1,
        }
    }

    pub fn update(&mut self, action: Action) -> bool {
        if action == Action::Quit {
            return true;
        }

        match self.screen {
            Screen::Connections => self.update_connections(action),
            Screen::ProfileDetails => {
                if action == Action::Back {
                    self.screen = Screen::Connections;
                    self.status = "Profile unchanged.".into();
                }
            }
            Screen::Workspace => self.update_workspace(action),
            Screen::Review => {
                if matches!(action, Action::Back | Action::Activate) {
                    self.screen = Screen::Workspace;
                    self.status = "Dry-run reviewed; no files were transferred by this PoC.".into();
                }
            }
        }

        false
    }

    fn update_connections(&mut self, action: Action) {
        match action {
            Action::Up => {
                self.selected_profile = previous_index(self.selected_profile, self.profiles.len());
            }
            Action::Down => {
                self.selected_profile = next_index(self.selected_profile, self.profiles.len());
            }
            Action::Activate => {
                let profile = &self.profiles[self.selected_profile];
                self.connected_profile_id = Some(profile.id.clone());
                self.screen = Screen::Workspace;
                self.status = format!("Connected to synthetic profile {}.", profile.label);
            }
            Action::EditProfile => {
                self.screen = Screen::ProfileDetails;
                self.status = "Profile details are isolated from connection selection.".into();
            }
            _ => {}
        }
    }

    fn update_workspace(&mut self, action: Action) {
        match action {
            Action::Up => self.move_selection(false),
            Action::Down => self.move_selection(true),
            Action::NextFocus => {
                self.focus = self.focus.next();
                self.status = format!("Focus: {:?}", self.focus);
            }
            Action::AddToPlan => self.add_selected_to_plan(),
            Action::RemovePlanItem if self.focus == Focus::Waybill => {
                if !self.plan.is_empty() {
                    self.plan.remove(self.plan_selection);
                    self.plan_selection =
                        self.plan_selection.min(self.plan.len().saturating_sub(1));
                    self.status = "Removed one Waybill item.".into();
                }
            }
            Action::CycleConflictPolicy if self.focus == Focus::Waybill => {
                if let Some(item) = self.plan.get_mut(self.plan_selection) {
                    item.conflict_policy = item.conflict_policy.next();
                    self.status = format!("Conflict policy: {}.", item.conflict_policy);
                }
            }
            Action::ReviewPlan | Action::Activate
                if self.focus == Focus::Waybill && !self.plan.is_empty() =>
            {
                self.screen = Screen::Review;
                self.status = "Review exact endpoints before execution.".into();
            }
            Action::Back => {
                self.screen = Screen::Connections;
                self.status = "Connection picker; profiles remain unchanged.".into();
            }
            _ => {}
        }
    }

    fn move_selection(&mut self, forward: bool) {
        match self.focus {
            Focus::Local => {
                self.local_selection =
                    move_index(self.local_selection, self.local_entries.len(), forward);
            }
            Focus::Remote => {
                self.remote_selection =
                    move_index(self.remote_selection, self.remote_entries.len(), forward);
            }
            Focus::Waybill => {
                self.plan_selection = move_index(self.plan_selection, self.plan.len(), forward);
            }
        }
    }

    fn add_selected_to_plan(&mut self) {
        let Some(profile) = self.connected_profile() else {
            self.status = "No active profile.".into();
            return;
        };

        let item = match self.focus {
            Focus::Local => {
                let entry = &self.local_entries[self.local_selection];
                if entry.kind == EntryKind::Directory {
                    self.status = "Directory staging is deferred in this PoC.".into();
                    return;
                }
                TransferPlanItem {
                    id: self.next_plan_id,
                    source: Endpoint::local(entry.path.clone()),
                    destination: Endpoint::remote(
                        profile.id.clone(),
                        profile.label.clone(),
                        format!("/srv/xfercat/releases/{}", entry.name),
                    ),
                    direction: TransferDirection::Upload,
                    conflict_policy: ConflictPolicy::Ask,
                    state: TransferState::Staged,
                }
            }
            Focus::Remote => {
                let entry = &self.remote_entries[self.remote_selection];
                if entry.kind == EntryKind::Directory {
                    self.status = "Directory staging is deferred in this PoC.".into();
                    return;
                }
                TransferPlanItem {
                    id: self.next_plan_id,
                    source: Endpoint::remote(
                        profile.id.clone(),
                        profile.label.clone(),
                        entry.path.clone(),
                    ),
                    destination: Endpoint::local(format!("/workspace/incoming/{}", entry.name)),
                    direction: TransferDirection::Download,
                    conflict_policy: ConflictPolicy::Ask,
                    state: TransferState::Staged,
                }
            }
            Focus::Waybill => {
                self.status = "Select LOCAL or REMOTE before adding an item.".into();
                return;
            }
        };

        self.next_plan_id += 1;
        self.plan.push(item);
        self.plan_selection = self.plan.len() - 1;
        self.status = format!("Waybill contains {} item(s).", self.plan.len());
    }

    pub fn connected_profile(&self) -> Option<&ConnectionProfile> {
        let id = self.connected_profile_id.as_deref()?;
        self.profiles.iter().find(|profile| profile.id == id)
    }
}

fn move_index(current: usize, length: usize, forward: bool) -> usize {
    if forward {
        next_index(current, length)
    } else {
        previous_index(current, length)
    }
}

fn next_index(current: usize, length: usize) -> usize {
    if length == 0 {
        0
    } else {
        (current + 1) % length
    }
}

fn previous_index(current: usize, length: usize) -> usize {
    if length == 0 {
        0
    } else {
        (current + length - 1) % length
    }
}

#[cfg(test)]
mod tests {
    use super::{Action, App, Focus, Screen};
    use crate::domain::ConflictPolicy;

    #[test]
    fn selecting_and_opening_details_never_mutates_profiles() {
        let mut app = App::demo();
        let original = app.profiles.clone();

        app.update(Action::Down);
        app.update(Action::EditProfile);

        assert_eq!(app.screen, Screen::ProfileDetails);
        assert_eq!(app.profiles, original);

        app.update(Action::Back);
        app.update(Action::Activate);

        assert_eq!(app.profiles, original);
        assert_eq!(app.connected_profile_id.as_deref(), Some("archive"));
    }

    #[test]
    fn staged_item_keeps_exact_endpoints_after_browser_state_changes() {
        let mut app = App::demo();
        app.update(Action::Activate);
        app.update(Action::AddToPlan);

        let staged = app.plan[0].clone();

        app.update(Action::Down);
        app.update(Action::NextFocus);
        app.update(Action::Down);

        assert_eq!(staged.source.path, "/workspace/outgoing/app.tar.gz");
        assert_eq!(staged.destination.path, "/srv/xfercat/releases/app.tar.gz");
        assert_eq!(app.plan[0], staged);
    }

    #[test]
    fn waybill_items_can_be_reviewed_edited_and_removed_independently() {
        let mut app = App::demo();
        app.update(Action::Activate);
        app.update(Action::AddToPlan);
        app.update(Action::NextFocus);
        app.update(Action::AddToPlan);
        app.update(Action::NextFocus);

        assert_eq!(app.focus, Focus::Waybill);
        assert_eq!(app.plan.len(), 2);

        app.update(Action::CycleConflictPolicy);
        assert_eq!(
            app.plan[app.plan_selection].conflict_policy,
            ConflictPolicy::Overwrite
        );

        app.update(Action::ReviewPlan);
        assert_eq!(app.screen, Screen::Review);
        app.update(Action::Back);
        app.update(Action::RemovePlanItem);

        assert_eq!(app.plan.len(), 1);
    }
}
