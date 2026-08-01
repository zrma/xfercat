use std::{
    collections::{BTreeMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
};

use glob::glob;

use crate::domain::ConnectionProfile;

const MAX_CONFIG_FILES: usize = 256;
const MAX_CONFIG_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OpenSshDiscovery {
    pub aliases: Vec<String>,
    pub files_read: usize,
    pub warnings: usize,
    pub root_found: bool,
}

impl OpenSshDiscovery {
    pub fn profiles(&self) -> Vec<ConnectionProfile> {
        self.aliases
            .iter()
            .map(ConnectionProfile::open_ssh)
            .collect()
    }

    pub fn status(&self) -> String {
        let mut status = if !self.root_found {
            "No OpenSSH user config found; A adds a manual profile.".into()
        } else if self.aliases.is_empty() {
            "No concrete OpenSSH Host aliases found; A adds a manual profile.".into()
        } else {
            format!(
                "Imported {} OpenSSH profile(s); I refreshes the catalog.",
                self.aliases.len()
            )
        };
        if self.warnings > 0 {
            status.push_str(&format!(" {} config item(s) were skipped.", self.warnings));
        }
        status
    }
}

pub fn discover_home() -> OpenSshDiscovery {
    let Some(home) = env::var_os("HOME").map(PathBuf::from) else {
        return OpenSshDiscovery::default();
    };
    discover_from_path(&home.join(".ssh/config"), &home)
}

pub fn discover_from_path(root: &Path, home: &Path) -> OpenSshDiscovery {
    let mut discovery = DiscoveryBuilder {
        home,
        ssh_directory: home.join(".ssh"),
        aliases: BTreeMap::new(),
        visited: HashSet::new(),
        report: OpenSshDiscovery {
            root_found: root.is_file(),
            ..OpenSshDiscovery::default()
        },
    };
    if discovery.report.root_found {
        discovery.read_file(root);
    }
    discovery.report.aliases = discovery.aliases.into_values().collect();
    discovery.report
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConfigContext {
    Global,
    Host,
    Match,
}

struct DiscoveryBuilder<'a> {
    home: &'a Path,
    ssh_directory: PathBuf,
    aliases: BTreeMap<String, String>,
    visited: HashSet<PathBuf>,
    report: OpenSshDiscovery,
}

impl DiscoveryBuilder<'_> {
    fn read_file(&mut self, path: &Path) {
        if self.report.files_read >= MAX_CONFIG_FILES {
            self.report.warnings += 1;
            return;
        }
        let identity = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        if !self.visited.insert(identity) {
            return;
        }
        let Ok(metadata) = fs::metadata(path) else {
            self.report.warnings += 1;
            return;
        };
        if !metadata.is_file() || metadata.len() > MAX_CONFIG_BYTES {
            self.report.warnings += 1;
            return;
        }
        let Ok(contents) = fs::read_to_string(path) else {
            self.report.warnings += 1;
            return;
        };
        self.report.files_read += 1;

        let mut context = ConfigContext::Global;
        for line in contents.lines() {
            let Some((keyword, arguments)) = parse_directive(line, &mut self.report.warnings)
            else {
                continue;
            };
            match keyword.as_str() {
                "host" => {
                    context = ConfigContext::Host;
                    for alias in arguments {
                        if is_concrete_alias(&alias) {
                            self.aliases
                                .entry(alias.to_ascii_lowercase())
                                .or_insert(alias);
                        }
                    }
                }
                "match" => context = ConfigContext::Match,
                "include" if context == ConfigContext::Global => {
                    for include in arguments {
                        self.read_include(&include);
                    }
                }
                "include" => self.report.warnings += 1,
                _ => {}
            }
        }
    }

    fn read_include(&mut self, include: &str) {
        let Some(expanded) = expand_include(include, self.home) else {
            self.report.warnings += 1;
            return;
        };
        let pattern = if expanded.is_absolute() {
            expanded
        } else {
            self.ssh_directory.join(expanded)
        };
        let Some(pattern) = pattern.to_str() else {
            self.report.warnings += 1;
            return;
        };
        let Ok(matches) = glob(pattern) else {
            self.report.warnings += 1;
            return;
        };
        let mut paths = matches
            .filter_map(|entry| match entry {
                Ok(path) => Some(path),
                Err(_) => {
                    self.report.warnings += 1;
                    None
                }
            })
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            self.read_file(&path);
        }
    }
}

fn is_concrete_alias(alias: &str) -> bool {
    !alias.is_empty()
        && alias.len() <= 255
        && !alias.starts_with('!')
        && !alias.contains('*')
        && !alias.contains('?')
        && !alias.chars().any(char::is_whitespace)
        && !alias.chars().any(char::is_control)
}

fn parse_directive(line: &str, warnings: &mut usize) -> Option<(String, Vec<String>)> {
    let words = match split_words(line) {
        Ok(words) => words,
        Err(()) => {
            *warnings += 1;
            return None;
        }
    };
    let (first, rest) = words.split_first()?;
    let (keyword, mut arguments) = if let Some((keyword, value)) = first.split_once('=') {
        let mut arguments = Vec::new();
        if !value.is_empty() {
            arguments.push(value.to_owned());
        }
        arguments.extend(rest.iter().cloned());
        (keyword.to_ascii_lowercase(), arguments)
    } else {
        (first.to_ascii_lowercase(), rest.to_vec())
    };
    if let Some(first) = arguments.first_mut()
        && let Some(value) = first.strip_prefix('=')
    {
        if value.is_empty() {
            arguments.remove(0);
        } else {
            *first = value.to_owned();
        }
    }
    Some((keyword, arguments))
}

fn split_words(line: &str) -> Result<Vec<String>, ()> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quoted = false;
    let mut escaped = false;

    for character in line.chars() {
        if escaped {
            word.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if character == '"' {
            quoted = !quoted;
            continue;
        }
        if character == '#' && !quoted {
            break;
        }
        if character.is_whitespace() && !quoted {
            if !word.is_empty() {
                words.push(std::mem::take(&mut word));
            }
            continue;
        }
        word.push(character);
    }
    if quoted || escaped {
        return Err(());
    }
    if !word.is_empty() {
        words.push(word);
    }
    Ok(words)
}

fn expand_include(value: &str, home: &Path) -> Option<PathBuf> {
    let home = home.to_str()?;
    let mut expanded = value.replace("%%", "\u{0}").replace("%d", home);
    if expanded.contains('%') {
        return None;
    }
    expanded = expanded.replace('\u{0}', "%");
    expanded = expand_environment(&expanded)?;
    if expanded == "~" {
        return Some(PathBuf::from(home));
    }
    if let Some(relative) = expanded.strip_prefix("~/") {
        return Some(Path::new(home).join(relative));
    }
    if expanded.starts_with('~') {
        return None;
    }
    Some(PathBuf::from(expanded))
}

fn expand_environment(value: &str) -> Option<String> {
    expand_environment_with(value, |name| env::var_os(name)?.into_string().ok())
}

fn expand_environment_with(
    value: &str,
    mut lookup: impl FnMut(&str) -> Option<String>,
) -> Option<String> {
    let mut output = String::new();
    let mut characters = value.chars().peekable();
    while let Some(character) = characters.next() {
        if character != '$' {
            output.push(character);
            continue;
        }
        let name = if characters.peek() == Some(&'{') {
            characters.next();
            let mut name = String::new();
            loop {
                let next = characters.next()?;
                if next == '}' {
                    break;
                }
                name.push(next);
            }
            name
        } else {
            let mut name = String::new();
            while characters
                .peek()
                .is_some_and(|next| next.is_ascii_alphanumeric() || *next == '_')
            {
                name.push(characters.next().expect("peeked character exists"));
            }
            name
        };
        if name.is_empty() {
            return None;
        }
        output.push_str(&lookup(&name)?);
    }
    Some(output)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        discover_from_path, expand_environment_with, expand_include, is_concrete_alias,
        parse_directive, split_words,
    };

    #[test]
    fn discovers_only_concrete_unique_aliases_in_stable_order() {
        let fixture = Fixture::new();
        fixture.write(
            ".ssh/config",
            r#"
                Host * !blocked *.example ?ingle
                Host build-box archive-box
                Host=BUILD-BOX
                Host "release-box" # quoted alias
            "#,
        );

        let discovery = discover_from_path(&fixture.path(".ssh/config"), &fixture.root);

        assert_eq!(
            discovery.aliases,
            ["archive-box", "build-box", "release-box"]
        );
        assert_eq!(discovery.files_read, 1);
        assert_eq!(discovery.warnings, 0);
    }

    #[test]
    fn follows_global_glob_includes_once_and_skips_conditional_include() {
        let fixture = Fixture::new();
        fixture.write(
            ".ssh/config",
            r#"
                Include ~/.ssh/config.d/*
                Host main-box
                    Include ~/.ssh/conditional.conf
            "#,
        );
        fixture.write(
            ".ssh/config.d/10-build.conf",
            "Include ~/.ssh/config\nHost build-box\n",
        );
        fixture.write(".ssh/config.d/20-release.conf", "Host release-box\n");
        fixture.write(".ssh/conditional.conf", "Host hidden-box\n");

        let discovery = discover_from_path(&fixture.path(".ssh/config"), &fixture.root);

        assert_eq!(discovery.aliases, ["build-box", "main-box", "release-box"]);
        assert_eq!(discovery.files_read, 3);
        assert_eq!(discovery.warnings, 1);
    }

    #[test]
    fn missing_config_is_an_empty_non_error_state() {
        let fixture = Fixture::new();

        let discovery = discover_from_path(&fixture.path(".ssh/config"), &fixture.root);

        assert!(!discovery.root_found);
        assert!(discovery.aliases.is_empty());
        assert_eq!(discovery.warnings, 0);
        assert!(discovery.status().contains("No OpenSSH user config found"));
    }

    #[test]
    fn lexer_preserves_quoted_values_and_strips_comments() {
        assert_eq!(
            split_words(r#"Include "config.d/team hosts/*" # comment"#),
            Ok(vec!["Include".into(), "config.d/team hosts/*".into()])
        );
        assert!(split_words(r#"Host "unterminated"#).is_err());
    }

    #[test]
    fn directive_and_include_expansion_cover_separator_and_path_forms() {
        let mut warnings = 0;
        assert_eq!(
            parse_directive("Host =build-box", &mut warnings),
            Some(("host".into(), vec!["build-box".into()]))
        );
        assert_eq!(warnings, 0);

        let home = Path::new("/synthetic-home");
        assert_eq!(
            expand_include("%d/.ssh/config.d/*", home),
            Some(PathBuf::from("/synthetic-home/.ssh/config.d/*"))
        );
        assert_eq!(
            expand_include("~/.ssh/config", home),
            Some(PathBuf::from("/synthetic-home/.ssh/config"))
        );
        assert!(expand_include("%h/.ssh/config", home).is_none());
        assert!(expand_include("~other-user/.ssh/config", home).is_none());
        assert_eq!(
            expand_environment_with("$CONFIG_ROOT/${TEAM}/config", |name| match name {
                "CONFIG_ROOT" => Some("/synthetic-root".into()),
                "TEAM" => Some("build".into()),
                _ => None,
            }),
            Some("/synthetic-root/build/config".into())
        );
        assert!(!is_concrete_alias("bad\u{1b}alias"));
        assert!(!is_concrete_alias(&"x".repeat(256)));
    }

    struct Fixture {
        root: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock is after Unix epoch")
                .as_nanos();
            let root = std::env::temp_dir()
                .join(format!("xfercat-openssh-{}-{unique}", std::process::id()));
            fs::create_dir_all(&root).expect("create fixture root");
            Self { root }
        }

        fn path(&self, relative: impl AsRef<Path>) -> PathBuf {
            self.root.join(relative)
        }

        fn write(&self, relative: impl AsRef<Path>, contents: &str) {
            let path = self.path(relative);
            fs::create_dir_all(path.parent().expect("fixture file has parent"))
                .expect("create fixture directory");
            fs::write(path, contents).expect("write fixture");
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
