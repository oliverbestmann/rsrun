use log::{info, warn};
use std::collections::HashSet;
use std::os::linux::fs::MetadataExt;
use std::sync::LazyLock;
use std::time::Instant;
use std::{env, fs, thread};

static BINARIES: LazyLock<Vec<String>> = LazyLock::new(list_binaries);

pub fn preload() {
    // access the binaries in a different thread so that the initialization
    // is performed in background.
    thread::spawn(|| &*BINARIES);
}

pub fn candidates(prefix: &str) -> Vec<&'static str> {
    BINARIES
        .iter()
        .filter(|binary| binary.starts_with(prefix))
        .map(|str| str.as_str())
        .collect()
}

fn list_binaries() -> Vec<String> {
    let start = Instant::now();

    let path = env::var_os("PATH").unwrap_or_default();

    let mut files = HashSet::<String>::new();

    for path in env::split_paths(&path) {
        info!("Looking for binaries in {:?}", path);
        let Ok(iter) = fs::read_dir(&path) else {
            warn!("Failed to list binaries in {:?}", path);
            continue;
        };

        for file in iter {
            let Ok(file) = file else {
                continue;
            };

            let name = file.file_name();
            let Some(name) = name.to_str() else { continue };

            if files.contains(name) {
                continue;
            };

            let Ok(meta) = fs::metadata(file.path()) else {
                warn!("Failed to stat: {:?}", file.path());
                continue;
            };

            if meta.is_file() && (meta.st_mode() & 0o400) != 0 {
                // executable file
                files.insert(name.to_owned());
            }
        }
    }

    let mut binaries: Vec<_> = files.into_iter().collect();
    binaries.sort();

    info!(
        "Found {} binaries for autocompletion in {:?}",
        binaries.len(),
        Instant::now().duration_since(start),
    );

    binaries
}
