use crate::domain::{
    Authentication, BrowserEntry, ConflictPolicy, ConnectionProfile, Endpoint, EntryKind, Protocol,
    TransferDirection, TransferPlanItem, TransferState,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Screen {
    Connections,
    ProfileEditor,
    Workspace,
    Rename,
    Review,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProfileField {
    Label,
    User,
    Host,
    Authentication,
    KeyReference,
}

impl ProfileField {
    pub fn next(self, authentication: ProfileAuthentication) -> Self {
        match self {
            Self::Label => Self::User,
            Self::User => Self::Host,
            Self::Host => Self::Authentication,
            Self::Authentication if authentication == ProfileAuthentication::KeyReference => {
                Self::KeyReference
            }
            Self::Authentication | Self::KeyReference => Self::Label,
        }
    }

    pub fn previous(self, authentication: ProfileAuthentication) -> Self {
        match self {
            Self::Label if authentication == ProfileAuthentication::KeyReference => {
                Self::KeyReference
            }
            Self::Label => Self::Authentication,
            Self::User => Self::Label,
            Self::Host => Self::User,
            Self::Authentication => Self::Host,
            Self::KeyReference => Self::Authentication,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProfileAuthentication {
    SshAgent,
    KeyReference,
}

impl ProfileAuthentication {
    pub const fn toggled(self) -> Self {
        match self {
            Self::SshAgent => Self::KeyReference,
            Self::KeyReference => Self::SshAgent,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::SshAgent => "SSH Agent",
            Self::KeyReference => "Key reference",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileEditor {
    pub profile_id: Option<String>,
    pub field: ProfileField,
    pub label: String,
    pub user: String,
    pub host: String,
    pub authentication: ProfileAuthentication,
    pub key_reference: String,
}

impl ProfileEditor {
    fn create() -> Self {
        Self {
            profile_id: None,
            field: ProfileField::Label,
            label: String::new(),
            user: String::new(),
            host: String::new(),
            authentication: ProfileAuthentication::SshAgent,
            key_reference: String::new(),
        }
    }

    fn edit(profile: &ConnectionProfile) -> Self {
        let (authentication, key_reference) = match &profile.authentication {
            Authentication::SshAgent => (ProfileAuthentication::SshAgent, String::new()),
            Authentication::KeyReference(reference) => {
                (ProfileAuthentication::KeyReference, reference.clone())
            }
        };
        Self {
            profile_id: Some(profile.id.clone()),
            field: ProfileField::Label,
            label: profile.label.clone(),
            user: profile.user.clone(),
            host: profile.host.clone(),
            authentication,
            key_reference,
        }
    }

    pub const fn is_create(&self) -> bool {
        self.profile_id.is_none()
    }
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
    AddProfile,
    EditProfile,
    NextProfileField,
    PreviousProfileField,
    ToggleProfileAuthentication,
    InputProfileChar(char),
    BackspaceProfile,
    NextFocus,
    AddToPlan,
    RemovePlanItem,
    CycleConflictPolicy,
    BeginRename,
    InputRenameChar(char),
    BackspaceRename,
    MovePlanUp,
    MovePlanDown,
    ReviewPlan,
    Quit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct App {
    pub screen: Screen,
    pub focus: Focus,
    pub profiles: Vec<ConnectionProfile>,
    pub selected_profile: usize,
    pub profile_editor: Option<ProfileEditor>,
    pub connected_profile_id: Option<String>,
    pub local_entries: Vec<BrowserEntry>,
    pub remote_entries: Vec<BrowserEntry>,
    pub local_selection: usize,
    pub remote_selection: usize,
    pub plan: Vec<TransferPlanItem>,
    pub plan_selection: usize,
    pub rename_buffer: String,
    pub status: String,
    next_profile_id: u64,
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
            profile_editor: None,
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
            rename_buffer: String::new(),
            status: "Choose a complete profile; editing is a separate action.".into(),
            next_profile_id: 1,
            next_plan_id: 1,
        }
    }

    pub fn update(&mut self, action: Action) -> bool {
        if action == Action::Quit {
            return true;
        }

        match self.screen {
            Screen::Connections => self.update_connections(action),
            Screen::ProfileEditor => self.update_profile_editor(action),
            Screen::Workspace => self.update_workspace(action),
            Screen::Rename => self.update_rename(action),
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
                if let Some(profile) = self.profiles.get(self.selected_profile) {
                    self.connected_profile_id = Some(profile.id.clone());
                    self.screen = Screen::Workspace;
                    self.status = format!("Connected to synthetic profile {}.", profile.label);
                }
            }
            Action::AddProfile => {
                self.profile_editor = Some(ProfileEditor::create());
                self.screen = Screen::ProfileEditor;
                self.status = "Create a process-lifetime synthetic profile.".into();
            }
            Action::EditProfile => {
                if let Some(profile) = self.profiles.get(self.selected_profile) {
                    self.profile_editor = Some(ProfileEditor::edit(profile));
                    self.screen = Screen::ProfileEditor;
                    self.status = "Edit is isolated from the Connect action.".into();
                }
            }
            _ => {}
        }
    }

    fn update_profile_editor(&mut self, action: Action) {
        let Some(editor) = self.profile_editor.as_mut() else {
            self.screen = Screen::Connections;
            self.status = "Profile editor state was unavailable.".into();
            return;
        };

        match action {
            Action::NextProfileField => {
                editor.field = editor.field.next(editor.authentication);
            }
            Action::PreviousProfileField => {
                editor.field = editor.field.previous(editor.authentication);
            }
            Action::ToggleProfileAuthentication if editor.field == ProfileField::Authentication => {
                editor.authentication = editor.authentication.toggled();
            }
            Action::InputProfileChar(character) if !character.is_control() => match editor.field {
                ProfileField::Label => editor.label.push(character),
                ProfileField::User => editor.user.push(character),
                ProfileField::Host => editor.host.push(character),
                ProfileField::Authentication => {}
                ProfileField::KeyReference => editor.key_reference.push(character),
            },
            Action::BackspaceProfile => match editor.field {
                ProfileField::Label => {
                    editor.label.pop();
                }
                ProfileField::User => {
                    editor.user.pop();
                }
                ProfileField::Host => {
                    editor.host.pop();
                }
                ProfileField::Authentication => {}
                ProfileField::KeyReference => {
                    editor.key_reference.pop();
                }
            },
            Action::Activate => self.save_profile(),
            Action::Back => {
                self.profile_editor = None;
                self.screen = Screen::Connections;
                self.status = "Profile edit cancelled; catalog unchanged.".into();
            }
            _ => {}
        }
    }

    fn save_profile(&mut self) {
        let Some(editor) = self.profile_editor.clone() else {
            return;
        };
        let profile = match profile_from_editor(&editor, &self.profiles, self.next_profile_id) {
            Ok(profile) => profile,
            Err(message) => {
                self.status = message.into();
                return;
            }
        };
        let label = profile.label.clone();

        if let Some(profile_id) = editor.profile_id {
            let Some(index) = self
                .profiles
                .iter()
                .position(|existing| existing.id == profile_id)
            else {
                self.status = "Profile being edited no longer exists.".into();
                return;
            };
            self.profiles[index] = profile;
            self.selected_profile = index;
            self.status = format!("Updated profile {label}; press Enter to connect.");
        } else {
            self.profiles.push(profile);
            self.selected_profile = self.profiles.len() - 1;
            self.next_profile_id += 1;
            self.status = format!("Added profile {label}; press Enter to connect.");
        }

        self.profile_editor = None;
        self.screen = Screen::Connections;
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
            Action::BeginRename if self.focus == Focus::Waybill => {
                if let Some(item) = self.plan.get(self.plan_selection) {
                    self.rename_buffer = destination_file_name(&item.destination.path)
                        .unwrap_or_default()
                        .to_owned();
                    self.screen = Screen::Rename;
                    self.status = format!("Renaming Waybill item #{}.", item.id);
                }
            }
            Action::MovePlanUp if self.focus == Focus::Waybill => {
                self.move_plan_item(false);
            }
            Action::MovePlanDown if self.focus == Focus::Waybill => {
                self.move_plan_item(true);
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

    fn update_rename(&mut self, action: Action) {
        match action {
            Action::InputRenameChar(character) if !character.is_control() => {
                self.rename_buffer.push(character);
            }
            Action::BackspaceRename => {
                self.rename_buffer.pop();
            }
            Action::Activate => {
                let Some(item) = self.plan.get_mut(self.plan_selection) else {
                    self.screen = Screen::Workspace;
                    self.status = "Waybill item no longer exists.".into();
                    return;
                };
                match renamed_destination(&item.destination.path, &self.rename_buffer) {
                    Ok(path) => {
                        item.destination.path = path;
                        let item_id = item.id;
                        self.screen = Screen::Workspace;
                        self.status = format!("Renamed destination for Waybill item #{item_id}.");
                    }
                    Err(message) => {
                        self.status = message.into();
                    }
                }
            }
            Action::Back => {
                self.screen = Screen::Workspace;
                self.status = "Rename cancelled; destination unchanged.".into();
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

    fn move_plan_item(&mut self, forward: bool) {
        if self.plan.is_empty() {
            return;
        }
        let target = if forward {
            self.plan_selection.checked_add(1)
        } else {
            self.plan_selection.checked_sub(1)
        };
        let Some(target) = target.filter(|target| *target < self.plan.len()) else {
            self.status = if forward {
                "Waybill item is already last.".into()
            } else {
                "Waybill item is already first.".into()
            };
            return;
        };

        let item_id = self.plan[self.plan_selection].id;
        self.plan.swap(self.plan_selection, target);
        self.plan_selection = target;
        self.status = format!(
            "Moved Waybill item #{item_id} {}.",
            if forward { "down" } else { "up" }
        );
    }

    pub fn connected_profile(&self) -> Option<&ConnectionProfile> {
        let id = self.connected_profile_id.as_deref()?;
        self.profiles.iter().find(|profile| profile.id == id)
    }
}

fn profile_from_editor(
    editor: &ProfileEditor,
    profiles: &[ConnectionProfile],
    next_profile_id: u64,
) -> Result<ConnectionProfile, &'static str> {
    let label = editor.label.trim();
    let user = editor.user.trim();
    let host = editor.host.trim();

    if label.is_empty() {
        return Err("Profile label cannot be empty.");
    }
    if label.chars().any(char::is_control) {
        return Err("Profile label cannot contain control characters.");
    }
    if profiles.iter().any(|profile| {
        profile.id != editor.profile_id.as_deref().unwrap_or_default()
            && profile.label.eq_ignore_ascii_case(label)
    }) {
        return Err("Profile label must be unique.");
    }
    if user.is_empty() {
        return Err("User cannot be empty.");
    }
    if user.chars().any(char::is_whitespace) {
        return Err("User cannot contain whitespace.");
    }
    if host.is_empty() {
        return Err("Host cannot be empty.");
    }
    if host.chars().any(char::is_whitespace) {
        return Err("Host cannot contain whitespace.");
    }

    let authentication = match editor.authentication {
        ProfileAuthentication::SshAgent => Authentication::SshAgent,
        ProfileAuthentication::KeyReference => {
            let reference = editor.key_reference.trim();
            if reference.is_empty() {
                return Err("Key reference cannot be empty.");
            }
            if reference.chars().any(char::is_whitespace) {
                return Err("Key reference cannot contain whitespace.");
            }
            Authentication::KeyReference(reference.into())
        }
    };

    Ok(ConnectionProfile {
        id: editor
            .profile_id
            .clone()
            .unwrap_or_else(|| format!("custom-{next_profile_id}")),
        label: label.into(),
        protocol: Protocol::Sftp,
        user: user.into(),
        host: host.into(),
        authentication,
    })
}

fn destination_file_name(path: &str) -> Option<&str> {
    path.rsplit_once('/')
        .map(|(_, file_name)| file_name)
        .filter(|file_name| !file_name.is_empty())
}

fn renamed_destination(path: &str, file_name: &str) -> Result<String, &'static str> {
    if file_name.is_empty() {
        return Err("Filename cannot be empty.");
    }
    if matches!(file_name, "." | "..") {
        return Err("Filename cannot be . or ..");
    }
    if file_name.contains('/') || file_name.contains('\\') {
        return Err("Filename cannot contain a path separator.");
    }
    if file_name.chars().any(char::is_control) {
        return Err("Filename cannot contain control characters.");
    }
    let Some((parent, _)) = path.rsplit_once('/') else {
        return Err("Destination has no parent path.");
    };
    Ok(format!("{parent}/{file_name}"))
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
    use super::{Action, App, Focus, ProfileField, Screen};
    use crate::domain::{Authentication, ConflictPolicy};

    #[test]
    fn cancelled_profile_edit_is_isolated_from_connect() {
        let mut app = App::demo();
        let original = app.profiles.clone();

        app.update(Action::Down);
        app.update(Action::EditProfile);
        app.update(Action::InputProfileChar('x'));

        assert_eq!(app.screen, Screen::ProfileEditor);
        assert_eq!(app.profiles, original);

        app.update(Action::Back);
        app.update(Action::Activate);

        assert_eq!(app.profiles, original);
        assert_eq!(app.connected_profile_id.as_deref(), Some("archive"));
    }

    #[test]
    fn creating_profile_adds_a_unique_stable_catalog_item() {
        let mut app = App::demo();

        app.update(Action::AddProfile);
        input_profile_text(&mut app, "lab box");
        app.update(Action::NextProfileField);
        input_profile_text(&mut app, "builder");
        app.update(Action::NextProfileField);
        input_profile_text(&mut app, "lab.example");
        app.update(Action::NextProfileField);
        app.update(Action::ToggleProfileAuthentication);
        app.update(Action::NextProfileField);
        input_profile_text(&mut app, "lab-key");
        app.update(Action::Activate);

        assert_eq!(app.screen, Screen::Connections);
        assert_eq!(app.selected_profile, 2);
        assert_eq!(app.profiles.len(), 3);
        assert_eq!(app.profiles[2].id, "custom-1");
        assert_eq!(app.profiles[2].label, "lab box");
        assert_eq!(app.profiles[2].user, "builder");
        assert_eq!(app.profiles[2].host, "lab.example");
        assert_eq!(
            app.profiles[2].authentication,
            Authentication::KeyReference("lab-key".into())
        );
    }

    #[test]
    fn editing_profile_preserves_identity_and_other_catalog_items() {
        let mut app = App::demo();
        let untouched = app.profiles[0].clone();

        app.update(Action::Down);
        let stable_id = app.profiles[1].id.clone();
        app.update(Action::EditProfile);
        clear_profile_field(&mut app);
        input_profile_text(&mut app, "cold archive");
        app.update(Action::NextProfileField);
        clear_profile_field(&mut app);
        input_profile_text(&mut app, "backup");
        app.update(Action::Activate);

        assert_eq!(app.screen, Screen::Connections);
        assert_eq!(app.profiles[0], untouched);
        assert_eq!(app.profiles[1].id, stable_id);
        assert_eq!(app.profiles[1].label, "cold archive");
        assert_eq!(app.profiles[1].user, "backup");
        assert_eq!(
            app.profiles[1].authentication,
            Authentication::KeyReference("archive-key".into())
        );
    }

    #[test]
    fn invalid_or_duplicate_profile_never_mutates_catalog() {
        let mut app = App::demo();
        let original = app.profiles.clone();

        app.update(Action::AddProfile);
        input_profile_text(&mut app, "DEV-BOX");
        app.update(Action::Activate);

        assert_eq!(app.screen, Screen::ProfileEditor);
        assert_eq!(app.profiles, original);
        assert_eq!(app.status, "Profile label must be unique.");

        clear_profile_field(&mut app);
        input_profile_text(&mut app, "new-box");
        app.update(Action::NextProfileField);
        input_profile_text(&mut app, "builder");
        app.update(Action::NextProfileField);
        input_profile_text(&mut app, "new.example");
        app.update(Action::NextProfileField);
        app.update(Action::ToggleProfileAuthentication);
        app.update(Action::Activate);

        assert_eq!(app.screen, Screen::ProfileEditor);
        assert_eq!(app.profiles, original);
        assert_eq!(app.status, "Key reference cannot be empty.");

        app.update(Action::Back);
        assert_eq!(app.screen, Screen::Connections);
        assert_eq!(app.profiles, original);
    }

    #[test]
    fn authentication_changes_only_from_its_selected_field() {
        let mut app = App::demo();
        app.update(Action::AddProfile);

        app.update(Action::ToggleProfileAuthentication);
        assert_eq!(
            app.profile_editor
                .as_ref()
                .expect("profile editor is active")
                .authentication,
            super::ProfileAuthentication::SshAgent
        );

        app.update(Action::NextProfileField);
        app.update(Action::NextProfileField);
        app.update(Action::NextProfileField);
        assert_eq!(
            app.profile_editor
                .as_ref()
                .expect("profile editor is active")
                .field,
            ProfileField::Authentication
        );

        app.update(Action::InputProfileChar('x'));
        assert_eq!(
            app.profile_editor
                .as_ref()
                .expect("profile editor is active")
                .authentication,
            super::ProfileAuthentication::SshAgent
        );
        app.update(Action::ToggleProfileAuthentication);
        assert_eq!(
            app.profile_editor
                .as_ref()
                .expect("profile editor is active")
                .authentication,
            super::ProfileAuthentication::KeyReference
        );
    }

    #[test]
    fn editing_connected_profile_does_not_rewrite_staged_endpoints() {
        let mut app = App::demo();
        app.update(Action::Activate);
        app.update(Action::AddToPlan);
        let staged = app.plan[0].clone();

        app.update(Action::Back);
        app.update(Action::EditProfile);
        clear_profile_field(&mut app);
        input_profile_text(&mut app, "renamed dev");
        app.update(Action::Activate);

        assert_eq!(app.profiles[0].id, "dev-box");
        assert_eq!(app.plan[0], staged);
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

    #[test]
    fn rename_changes_only_the_destination_leaf() {
        let mut app = staged_app();
        let original = app.plan[1].clone();

        app.update(Action::BeginRename);
        clear_rename_buffer(&mut app);
        for character in "service-copy.log".chars() {
            app.update(Action::InputRenameChar(character));
        }
        app.update(Action::Activate);

        assert_eq!(app.screen, Screen::Workspace);
        assert_eq!(app.plan[1].source, original.source);
        assert_eq!(
            app.plan[1].destination.profile_id,
            original.destination.profile_id
        );
        assert_eq!(
            app.plan[1].destination.path,
            "/workspace/incoming/service-copy.log"
        );
    }

    #[test]
    fn invalid_rename_preserves_the_plan_and_stays_in_rename_mode() {
        let mut app = staged_app();
        let original = app.plan.clone();

        app.update(Action::BeginRename);
        for invalid_name in ["", ".", "..", "nested/name", r"nested\name"] {
            clear_rename_buffer(&mut app);
            for character in invalid_name.chars() {
                app.update(Action::InputRenameChar(character));
            }
            app.update(Action::Activate);

            assert_eq!(app.screen, Screen::Rename);
            assert_eq!(app.plan, original);
        }
        assert_eq!(app.status, "Filename cannot contain a path separator.");
    }

    #[test]
    fn reorder_moves_the_selected_stable_item_without_changing_payload() {
        let mut app = staged_app();
        let selected = app.plan[1].clone();

        app.update(Action::MovePlanUp);

        assert_eq!(app.plan_selection, 0);
        assert_eq!(app.plan[0], selected);

        app.update(Action::MovePlanUp);
        assert_eq!(app.plan[0], selected);
        assert_eq!(app.status, "Waybill item is already first.");

        app.update(Action::MovePlanDown);
        assert_eq!(app.plan_selection, 1);
        assert_eq!(app.plan[1], selected);
    }

    #[test]
    fn cancelling_rename_discards_the_edit_buffer() {
        let mut app = staged_app();
        let original = app.plan.clone();

        app.update(Action::BeginRename);
        app.update(Action::InputRenameChar('x'));
        app.update(Action::Back);

        assert_eq!(app.screen, Screen::Workspace);
        assert_eq!(app.plan, original);
        assert_eq!(app.status, "Rename cancelled; destination unchanged.");
    }

    fn staged_app() -> App {
        let mut app = App::demo();
        app.update(Action::Activate);
        app.update(Action::AddToPlan);
        app.update(Action::NextFocus);
        app.update(Action::AddToPlan);
        app.update(Action::NextFocus);
        app
    }

    fn clear_rename_buffer(app: &mut App) {
        while !app.rename_buffer.is_empty() {
            app.update(Action::BackspaceRename);
        }
    }

    fn input_profile_text(app: &mut App, value: &str) {
        for character in value.chars() {
            app.update(Action::InputProfileChar(character));
        }
    }

    fn clear_profile_field(app: &mut App) {
        let field = app
            .profile_editor
            .as_ref()
            .map(|editor| editor.field)
            .expect("profile editor is active");
        assert!(matches!(
            field,
            ProfileField::Label
                | ProfileField::User
                | ProfileField::Host
                | ProfileField::KeyReference
        ));
        loop {
            let is_empty = {
                let editor = app
                    .profile_editor
                    .as_ref()
                    .expect("profile editor is active");
                match field {
                    ProfileField::Label => editor.label.is_empty(),
                    ProfileField::User => editor.user.is_empty(),
                    ProfileField::Host => editor.host.is_empty(),
                    ProfileField::KeyReference => editor.key_reference.is_empty(),
                    ProfileField::Authentication => true,
                }
            };
            if is_empty {
                break;
            }
            app.update(Action::BackspaceProfile);
        }
    }
}
