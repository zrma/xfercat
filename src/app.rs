use crate::domain::{
    Authentication, BrowserEntry, ConflictPolicy, ConnectionProfile, ConnectionProfileSource,
    Endpoint, EntryKind, Protocol, TransferDirection, TransferPlanItem, TransferState,
};
use crate::executor;
use crate::localfs::LocalDirectory;
use crate::sftp::RemoteDirectory;

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

    fn edit(profile: &ConnectionProfile) -> Option<Self> {
        let (authentication, key_reference) = match &profile.authentication {
            Authentication::SshAgent => (ProfileAuthentication::SshAgent, String::new()),
            Authentication::KeyReference(reference) => {
                (ProfileAuthentication::KeyReference, reference.clone())
            }
            Authentication::OpenSshConfig => return None,
        };
        Some(Self {
            profile_id: Some(profile.id.clone()),
            field: ProfileField::Label,
            label: profile.label.clone(),
            user: profile.user.clone(),
            host: profile.host.clone(),
            authentication,
            key_reference,
        })
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
    DeleteProfile,
    RefreshOpenSshProfiles,
    NextProfileField,
    PreviousProfileField,
    ToggleProfileAuthentication,
    InputProfileChar(char),
    BackspaceProfile,
    NextFocus,
    NavigateParent,
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
    pub local_directory: String,
    pub remote_directory: String,
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
                    source: ConnectionProfileSource::Synthetic,
                    protocol: Protocol::Sftp,
                    user: "deploy".into(),
                    host: "dev.example".into(),
                    authentication: Authentication::SshAgent,
                },
                ConnectionProfile {
                    id: "archive".into(),
                    label: "archive".into(),
                    source: ConnectionProfileSource::Synthetic,
                    protocol: Protocol::Sftp,
                    user: "operator".into(),
                    host: "archive.example".into(),
                    authentication: Authentication::KeyReference("archive-key".into()),
                },
            ],
            selected_profile: 0,
            profile_editor: None,
            connected_profile_id: None,
            local_directory: "/workspace/outgoing".into(),
            remote_directory: "/srv/xfercat".into(),
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

    pub fn runtime(profiles: Vec<ConnectionProfile>, status: impl Into<String>) -> Self {
        let mut app = Self::demo();
        app.profiles = profiles;
        app.selected_profile = 0;
        app.connected_profile_id = None;
        app.status = status.into();
        app
    }

    pub fn refresh_open_ssh_profiles(
        &mut self,
        imported: Vec<ConnectionProfile>,
        status: impl Into<String>,
    ) {
        let selected_id = self
            .profiles
            .get(self.selected_profile)
            .map(|profile| profile.id.clone());
        self.profiles.retain(|profile| !profile.is_open_ssh());

        let mut conflicts = 0;
        for profile in imported.into_iter().filter(ConnectionProfile::is_open_ssh) {
            if self
                .profiles
                .iter()
                .any(|existing| existing.label.eq_ignore_ascii_case(&profile.label))
            {
                conflicts += 1;
            } else {
                self.profiles.push(profile);
            }
        }
        self.profiles.sort_by(|left, right| {
            left.label
                .to_ascii_lowercase()
                .cmp(&right.label.to_ascii_lowercase())
        });
        self.selected_profile = selected_id
            .as_deref()
            .and_then(|id| self.profiles.iter().position(|profile| profile.id == id))
            .unwrap_or(0);
        if self
            .connected_profile_id
            .as_deref()
            .is_some_and(|id| self.profiles.iter().all(|profile| profile.id != id))
        {
            self.connected_profile_id = None;
        }
        self.status = status.into();
        if conflicts > 0 {
            self.status.push_str(&format!(
                " {conflicts} imported alias(es) conflict with manual profiles."
            ));
        }
    }

    pub fn replace_local_directory(&mut self, directory: LocalDirectory) {
        let status = directory.status();
        self.local_directory = directory.path;
        self.local_entries = directory.entries;
        self.local_selection = 0;
        self.status = status;
    }

    pub fn local_navigation_target(&self, parent: bool) -> Option<String> {
        if self.focus != Focus::Local {
            return None;
        }
        if parent {
            let current = std::path::Path::new(&self.local_directory);
            return current
                .parent()
                .filter(|candidate| *candidate != current)
                .and_then(std::path::Path::to_str)
                .map(str::to_owned);
        }
        self.local_entries
            .get(self.local_selection)
            .filter(|entry| entry.kind == EntryKind::Directory)
            .map(|entry| entry.path.clone())
    }

    pub fn enter_remote_workspace(&mut self, profile_id: &str, directory: RemoteDirectory) {
        let status = directory.status();
        self.connected_profile_id = Some(profile_id.to_owned());
        self.remote_directory = directory.path;
        self.remote_entries = directory.entries;
        self.remote_selection = 0;
        self.focus = Focus::Local;
        self.screen = Screen::Workspace;
        self.status = status;
    }

    pub fn replace_remote_directory(&mut self, directory: RemoteDirectory) {
        let status = directory.status();
        self.remote_directory = directory.path;
        self.remote_entries = directory.entries;
        self.remote_selection = 0;
        self.status = status;
    }

    pub fn remote_navigation_target(&self, parent: bool) -> Option<String> {
        if self.focus != Focus::Remote {
            return None;
        }
        if parent {
            return remote_parent(&self.remote_directory);
        }
        self.remote_entries
            .get(self.remote_selection)
            .filter(|entry| entry.kind == EntryKind::Directory)
            .map(|entry| entry.path.clone())
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
                if action == Action::Back {
                    self.screen = Screen::Workspace;
                    self.status =
                        "Synthetic review closed; no files were transferred by this PoC.".into();
                } else if action == Action::Activate {
                    self.execute_synthetic_plan();
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
                    self.status = format!(
                        "Selected profile {}; the workspace remains synthetic.",
                        profile.label
                    );
                } else {
                    self.status =
                        "No profile selected; refresh OpenSSH or add one manually.".into();
                }
            }
            Action::AddProfile => {
                self.profile_editor = Some(ProfileEditor::create());
                self.screen = Screen::ProfileEditor;
                self.status = "Create a process-lifetime manual profile.".into();
            }
            Action::EditProfile => {
                if let Some(profile) = self.profiles.get(self.selected_profile) {
                    if let Some(editor) = ProfileEditor::edit(profile) {
                        self.profile_editor = Some(editor);
                        self.screen = Screen::ProfileEditor;
                        self.status = "Edit is isolated from the Select action.".into();
                    } else {
                        self.status =
                            "OpenSSH profiles are read-only; edit the source config and press I."
                                .into();
                    }
                }
            }
            Action::DeleteProfile => self.delete_selected_profile(),
            _ => {}
        }
    }

    fn delete_selected_profile(&mut self) {
        let Some(profile) = self.profiles.get(self.selected_profile).cloned() else {
            self.status = "No profile selected; refresh OpenSSH or add one manually.".into();
            return;
        };
        if profile.is_open_ssh() {
            self.status =
                "OpenSSH profiles are source-owned; remove the Host entry and press I.".into();
            return;
        }

        let reference_count = self
            .plan
            .iter()
            .filter(|item| plan_item_references_profile(item, &profile.id))
            .count();
        if reference_count > 0 {
            self.status = format!(
                "Cannot delete {}; remove {reference_count} referencing Waybill item(s) first.",
                profile.label
            );
            return;
        }

        let cleared_active = self.connected_profile_id.as_deref() == Some(profile.id.as_str());
        self.profiles.remove(self.selected_profile);
        self.selected_profile = self
            .selected_profile
            .min(self.profiles.len().saturating_sub(1));
        if cleared_active {
            self.connected_profile_id = None;
            self.status = format!(
                "Deleted profile {}; active synthetic connection cleared.",
                profile.label
            );
        } else {
            self.status = format!("Deleted profile {}.", profile.label);
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
                let Some(entry) = self.local_entries.get(self.local_selection) else {
                    self.status = "No local file selected.".into();
                    return;
                };
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
                        join_logical_path(&self.remote_directory, &entry.name),
                    ),
                    direction: TransferDirection::Upload,
                    entry_kind: entry.kind,
                    expected_size: entry.size,
                    conflict_policy: ConflictPolicy::Ask,
                    state: TransferState::Staged,
                }
            }
            Focus::Remote => {
                let Some(entry) = self.remote_entries.get(self.remote_selection) else {
                    self.status = "No remote file selected.".into();
                    return;
                };
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
                    destination: Endpoint::local(join_logical_path(
                        &self.local_directory,
                        &entry.name,
                    )),
                    direction: TransferDirection::Download,
                    entry_kind: entry.kind,
                    expected_size: entry.size,
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

    fn execute_synthetic_plan(&mut self) {
        let summary = executor::execute_representative(&mut self.plan);
        if summary.total() == 0 {
            self.status = "No staged items; terminal synthetic results were preserved.".into();
        } else {
            self.status = format!(
                "Synthetic only: {} succeeded, {} failed, {} skipped, {} cancelled.",
                summary.succeeded, summary.failed, summary.skipped, summary.cancelled
            );
        }
    }

    pub fn connected_profile(&self) -> Option<&ConnectionProfile> {
        let id = self.connected_profile_id.as_deref()?;
        self.profiles.iter().find(|profile| profile.id == id)
    }
}

fn plan_item_references_profile(item: &TransferPlanItem, profile_id: &str) -> bool {
    item.source.profile_id.as_deref() == Some(profile_id)
        || item.destination.profile_id.as_deref() == Some(profile_id)
}

fn join_logical_path(directory: &str, name: &str) -> String {
    if directory == "/" {
        format!("/{name}")
    } else {
        format!("{}/{name}", directory.trim_end_matches('/'))
    }
}

fn remote_parent(path: &str) -> Option<String> {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    let (parent, _) = trimmed.rsplit_once('/')?;
    Some(if parent.is_empty() { "/" } else { parent }.to_owned())
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
        source: editor
            .profile_id
            .as_deref()
            .and_then(|id| profiles.iter().find(|profile| profile.id == id))
            .map(|profile| profile.source)
            .unwrap_or(ConnectionProfileSource::Manual),
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
    use crate::domain::{
        Authentication, BrowserEntry, ConflictPolicy, ConnectionProfile, EntryKind, TransferState,
    };
    use crate::localfs::LocalDirectory;
    use crate::sftp::RemoteDirectory;

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
    fn runtime_catalog_uses_imported_profiles_and_keeps_them_read_only() {
        let mut app = App::runtime(
            vec![
                ConnectionProfile::open_ssh("build-box"),
                ConnectionProfile::open_ssh("release-box"),
            ],
            "Imported 2 OpenSSH profile(s).",
        );

        assert_eq!(app.profiles.len(), 2);
        assert!(app.profiles.iter().all(ConnectionProfile::is_open_ssh));
        assert!(
            app.profiles
                .iter()
                .all(|profile| profile.label != "dev-box")
        );

        app.update(Action::EditProfile);
        assert_eq!(app.screen, Screen::Connections);
        assert!(app.profile_editor.is_none());
        assert!(app.status.contains("read-only"));

        app.update(Action::Activate);
        assert_eq!(app.screen, Screen::Workspace);
        assert_eq!(
            app.connected_profile_id.as_deref(),
            Some("openssh:build-box")
        );
        assert!(app.status.contains("workspace remains synthetic"));
    }

    #[test]
    fn refresh_replaces_imports_but_preserves_manual_profiles_and_staged_plan() {
        let mut app = App::runtime(
            vec![ConnectionProfile::open_ssh("build-box")],
            "Imported 1 OpenSSH profile(s).",
        );
        app.update(Action::Activate);
        app.update(Action::AddToPlan);
        let staged = app.plan.clone();
        app.update(Action::Back);

        app.update(Action::AddProfile);
        input_profile_text(&mut app, "manual-box");
        app.update(Action::NextProfileField);
        input_profile_text(&mut app, "builder");
        app.update(Action::NextProfileField);
        input_profile_text(&mut app, "manual.example");
        app.update(Action::Activate);
        let manual_id = app.profiles[app.selected_profile].id.clone();

        app.refresh_open_ssh_profiles(
            vec![ConnectionProfile::open_ssh("release-box")],
            "Imported 1 OpenSSH profile(s).",
        );

        assert_eq!(app.plan, staged);
        assert!(app.connected_profile_id.is_none());
        assert!(app.profiles.iter().any(|profile| profile.id == manual_id));
        assert!(
            app.profiles
                .iter()
                .any(|profile| profile.id == "openssh:release-box")
        );
        assert!(
            app.profiles
                .iter()
                .all(|profile| profile.id != "openssh:build-box")
        );
        assert_eq!(app.profiles[app.selected_profile].id, manual_id);
    }

    #[test]
    fn empty_runtime_catalog_is_safe_and_manual_add_remains_available() {
        let mut app = App::runtime(Vec::new(), "No OpenSSH user config found.");

        app.update(Action::Up);
        app.update(Action::Down);
        app.update(Action::Activate);
        app.update(Action::EditProfile);

        assert_eq!(app.screen, Screen::Connections);
        assert!(app.profiles.is_empty());
        assert!(app.status.contains("No profile selected"));

        app.update(Action::AddProfile);
        assert_eq!(app.screen, Screen::ProfileEditor);
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
    fn deleting_unreferenced_profile_preserves_catalog_selection() {
        let mut app = App::demo();
        app.update(Action::Down);

        app.update(Action::DeleteProfile);

        assert_eq!(app.profiles.len(), 1);
        assert_eq!(app.profiles[0].id, "dev-box");
        assert_eq!(app.selected_profile, 0);
        assert_eq!(app.status, "Deleted profile archive.");
    }

    #[test]
    fn deleting_active_unreferenced_profile_clears_connection() {
        let mut app = App::demo();
        app.update(Action::Down);
        app.update(Action::Activate);
        app.update(Action::Back);
        assert_eq!(app.connected_profile_id.as_deref(), Some("archive"));

        app.update(Action::DeleteProfile);

        assert!(app.connected_profile_id.is_none());
        assert!(app.profiles.iter().all(|profile| profile.id != "archive"));
        assert!(app.status.contains("active synthetic connection cleared"));
    }

    #[test]
    fn deleting_profile_referenced_by_waybill_is_non_cascading() {
        let mut app = App::demo();
        app.update(Action::Activate);
        app.update(Action::AddToPlan);
        app.update(Action::Back);
        let profiles = app.profiles.clone();
        let plan = app.plan.clone();

        app.update(Action::DeleteProfile);

        assert_eq!(app.profiles, profiles);
        assert_eq!(app.plan, plan);
        assert_eq!(app.connected_profile_id.as_deref(), Some("dev-box"));
        assert!(
            app.status
                .contains("remove 1 referencing Waybill item(s) first")
        );
    }

    #[test]
    fn deleting_imported_profile_is_deferred_to_source_config() {
        let mut app = App::runtime(
            vec![ConnectionProfile::open_ssh("build-box")],
            "Imported 1 OpenSSH profile(s).",
        );

        app.update(Action::DeleteProfile);

        assert_eq!(app.profiles, vec![ConnectionProfile::open_ssh("build-box")]);
        assert!(app.status.contains("remove the Host entry and press I"));
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
        assert_eq!(staged.destination.path, "/srv/xfercat/app.tar.gz");
        assert_eq!(staged.entry_kind, EntryKind::File);
        assert_eq!(staged.expected_size, Some(438 * 1024 * 1024));
        assert_eq!(app.plan[0], staged);
    }

    #[test]
    fn replacing_local_directory_resets_selection_and_exposes_safe_navigation_targets() {
        let mut app = App::demo();
        app.local_selection = 2;

        app.replace_local_directory(LocalDirectory {
            path: "/actual/local".into(),
            entries: vec![
                BrowserEntry {
                    name: "packages".into(),
                    path: "/actual/local/packages".into(),
                    kind: EntryKind::Directory,
                    size: None,
                },
                BrowserEntry {
                    name: "payload.bin".into(),
                    path: "/actual/local/payload.bin".into(),
                    kind: EntryKind::File,
                    size: Some(7),
                },
            ],
            skipped_entries: 1,
        });

        assert_eq!(app.local_selection, 0);
        assert_eq!(
            app.local_navigation_target(false).as_deref(),
            Some("/actual/local/packages")
        );
        assert_eq!(
            app.local_navigation_target(true).as_deref(),
            Some("/actual")
        );
        assert!(app.status.contains("1 unsafe or unreadable"));
        app.local_selection = 1;
        assert!(app.local_navigation_target(false).is_none());
    }

    #[test]
    fn entering_live_workspace_replaces_remote_fixture_and_exposes_navigation_targets() {
        let mut app = App::runtime(
            vec![ConnectionProfile::open_ssh("fixture-host")],
            "Fixture catalog loaded.",
        );

        app.enter_remote_workspace(
            "openssh:fixture-host",
            RemoteDirectory {
                path: "/remote/home".into(),
                entries: vec![BrowserEntry {
                    name: "packages".into(),
                    path: "/remote/home/packages".into(),
                    kind: EntryKind::Directory,
                    size: None,
                }],
                skipped_entries: 1,
            },
        );

        assert_eq!(app.screen, Screen::Workspace);
        assert_eq!(app.focus, Focus::Local);
        assert_eq!(
            app.connected_profile_id.as_deref(),
            Some("openssh:fixture-host")
        );
        assert_eq!(app.remote_directory, "/remote/home");
        assert!(app.status.contains("1 unsafe or unreadable"));

        app.focus = Focus::Remote;
        assert_eq!(
            app.remote_navigation_target(false).as_deref(),
            Some("/remote/home/packages")
        );
        assert_eq!(
            app.remote_navigation_target(true).as_deref(),
            Some("/remote")
        );
        app.replace_remote_directory(RemoteDirectory {
            path: "/".into(),
            entries: Vec::new(),
            skipped_entries: 0,
        });
        assert!(app.remote_navigation_target(true).is_none());
    }

    #[test]
    fn empty_browser_selection_cannot_be_staged() {
        let mut app = App::demo();
        app.update(Action::Activate);
        app.local_entries.clear();

        app.update(Action::AddToPlan);

        assert!(app.plan.is_empty());
        assert_eq!(app.status, "No local file selected.");
        app.focus = Focus::Remote;
        app.remote_entries.clear();
        app.update(Action::AddToPlan);
        assert!(app.plan.is_empty());
        assert_eq!(app.status, "No remote file selected.");
    }

    #[test]
    fn staging_uses_current_local_and_remote_directories() {
        let mut app = App::demo();
        app.local_directory = "/actual/local".into();
        app.remote_directory = "/actual/remote".into();
        app.update(Action::Activate);

        app.update(Action::AddToPlan);
        app.update(Action::NextFocus);
        app.update(Action::AddToPlan);

        assert_eq!(app.plan[0].source.path, "/workspace/outgoing/app.tar.gz");
        assert_eq!(app.plan[0].destination.path, "/actual/remote/app.tar.gz");
        assert_eq!(app.plan[1].source.path, "/srv/xfercat/service.log");
        assert_eq!(app.plan[1].destination.path, "/actual/local/service.log");
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
    fn review_executes_synthetic_results_and_preserves_them_on_back() {
        let mut app = staged_app();

        app.update(Action::ReviewPlan);
        app.update(Action::Activate);

        assert_eq!(app.screen, Screen::Review);
        assert_eq!(app.plan[0].state, TransferState::Succeeded);
        assert_eq!(app.plan[1].state, TransferState::Failed);
        assert!(app.status.starts_with("Synthetic only:"));

        app.update(Action::Activate);
        assert!(
            app.status
                .contains("terminal synthetic results were preserved")
        );
        app.update(Action::Back);
        assert_eq!(app.screen, Screen::Workspace);
        assert_eq!(app.plan[0].state, TransferState::Succeeded);
        assert_eq!(app.plan[1].state, TransferState::Failed);
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
            "/workspace/outgoing/service-copy.log"
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
