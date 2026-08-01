use std::{cmp::Ordering, fs, io, path::Path};

use crate::domain::{BrowserEntry, EntryKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalDirectory {
    pub path: String,
    pub entries: Vec<BrowserEntry>,
    pub skipped_entries: usize,
}

impl LocalDirectory {
    pub fn status(&self) -> String {
        if self.skipped_entries == 0 {
            format!("Local directory loaded: {} item(s).", self.entries.len())
        } else {
            format!(
                "Local directory loaded: {} item(s); {} unsafe or unreadable entry(s) skipped.",
                self.entries.len(),
                self.skipped_entries
            )
        }
    }
}

pub fn read_directory(path: impl AsRef<Path>) -> io::Result<LocalDirectory> {
    let canonical = fs::canonicalize(path)?;
    if !canonical.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "local browser path is not a directory",
        ));
    }
    let directory = canonical
        .to_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "directory path is not UTF-8"))?
        .to_owned();

    let mut entries = Vec::new();
    let mut skipped_entries = 0;
    for candidate in fs::read_dir(&canonical)? {
        let candidate = match candidate {
            Ok(candidate) => candidate,
            Err(_) => {
                skipped_entries += 1;
                continue;
            }
        };
        let file_type = match candidate.file_type() {
            Ok(file_type) if !file_type.is_symlink() => file_type,
            Ok(_) | Err(_) => {
                skipped_entries += 1;
                continue;
            }
        };
        let kind = if file_type.is_dir() {
            EntryKind::Directory
        } else if file_type.is_file() {
            EntryKind::File
        } else {
            skipped_entries += 1;
            continue;
        };
        let name = match candidate.file_name().into_string() {
            Ok(name) if !name.chars().any(char::is_control) => name,
            Ok(_) | Err(_) => {
                skipped_entries += 1;
                continue;
            }
        };
        let path = canonical.join(&name);
        let Some(path) = path.to_str().map(str::to_owned) else {
            skipped_entries += 1;
            continue;
        };
        let size = if kind == EntryKind::File {
            match candidate.metadata() {
                Ok(metadata) => Some(metadata.len()),
                Err(_) => {
                    skipped_entries += 1;
                    continue;
                }
            }
        } else {
            None
        };
        entries.push(BrowserEntry {
            name,
            path,
            kind,
            size,
        });
    }
    entries.sort_by(compare_entries);

    Ok(LocalDirectory {
        path: directory,
        entries,
        skipped_entries,
    })
}

fn compare_entries(left: &BrowserEntry, right: &BrowserEntry) -> Ordering {
    match (left.kind, right.kind) {
        (EntryKind::Directory, EntryKind::File) => Ordering::Less,
        (EntryKind::File, EntryKind::Directory) => Ordering::Greater,
        _ => left
            .name
            .to_ascii_lowercase()
            .cmp(&right.name.to_ascii_lowercase())
            .then_with(|| left.name.cmp(&right.name)),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::read_directory;
    use crate::domain::EntryKind;

    #[test]
    fn reads_real_entries_with_directory_first_and_exact_file_size() {
        let root = tempdir().expect("temporary root");
        fs::create_dir(root.path().join("packages")).expect("create directory");
        fs::write(root.path().join("beta.txt"), b"beta").expect("write beta");
        fs::write(root.path().join("Alpha.txt"), b"alpha").expect("write alpha");

        let listing = read_directory(root.path()).expect("read local directory");

        assert_eq!(
            listing.path,
            fs::canonicalize(root.path())
                .expect("canonical temporary root")
                .to_string_lossy()
        );
        assert_eq!(listing.skipped_entries, 0);
        assert_eq!(listing.entries[0].name, "packages");
        assert_eq!(listing.entries[0].kind, EntryKind::Directory);
        assert_eq!(listing.entries[1].name, "Alpha.txt");
        assert_eq!(listing.entries[1].size, Some(5));
        assert_eq!(listing.entries[2].name, "beta.txt");
        assert_eq!(listing.entries[2].size, Some(4));
    }

    #[cfg(unix)]
    #[test]
    fn excludes_symlinks_instead_of_following_them() {
        use std::os::unix::fs::symlink;

        let root = tempdir().expect("temporary root");
        fs::write(root.path().join("payload.bin"), b"payload").expect("write payload");
        symlink(
            root.path().join("payload.bin"),
            root.path().join("payload-link"),
        )
        .expect("create symlink");
        let listing = read_directory(root.path()).expect("read local directory");

        assert_eq!(listing.entries.len(), 1);
        assert_eq!(listing.entries[0].name, "payload.bin");
        assert_eq!(listing.skipped_entries, 1);
        assert!(listing.status().contains("unsafe or unreadable"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn excludes_non_unicode_names_instead_of_guessing_paths() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt};

        let root = tempdir().expect("temporary root");
        fs::write(
            root.path()
                .join(OsString::from_vec(b"invalid-\xff".to_vec())),
            b"x",
        )
        .expect("write non-Unicode entry");

        let listing = read_directory(root.path()).expect("read local directory");

        assert!(listing.entries.is_empty());
        assert_eq!(listing.skipped_entries, 1);
    }

    #[test]
    fn rejects_file_path_as_browser_directory() {
        let root = tempdir().expect("temporary root");
        let file = root.path().join("payload.bin");
        fs::write(&file, b"payload").expect("write payload");

        let error = read_directory(file).expect_err("file cannot be browsed as directory");

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    }
}
