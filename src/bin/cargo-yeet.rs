use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = clap::Command::new("cargo-yeet")
        .bin_name("cargo")
        .subcommand_required(true)
        .subcommand(
            clap::command!("yeet")
                .arg(
                    clap::arg!(--"manifest-path" <PATH>)
                        .short('m')
                        .value_parser(clap::value_parser!(std::path::PathBuf)),
                )
                .arg(
                    clap::arg!(--"path-override" <PATH>)
                        .short('p')
                        .value_parser(clap::value_parser!(std::path::PathBuf)),
                )
                .arg(
                    clap::arg!(--"recursive" <PATH>)
                        .short('r')
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    clap::arg!(--"execute" <PATH>)
                        .short('x')
                        .action(clap::ArgAction::SetTrue),
                ),
        );
    let matches = cmd.get_matches();
    let matches = match matches.subcommand() {
        Some(("yeet", matches)) => matches,
        _ => unreachable!("clap should ensure we don't get here"),
    };

    let mut paths = Vec::new();

    if let Some(manifest_path) = matches.get_one::<std::path::PathBuf>("manifest-path") {
        paths.push(manifest_path.clone())
    }

    let root_dir = if let Some(path) = matches.get_one::<std::path::PathBuf>("path-override") {
        path.clone()
    } else {
        std::env::current_dir()?
    };
    if matches.get_flag("recursive") {
        walk_dirs(&root_dir, &mut paths, 0);
    } else {
        paths.push(root_dir);
    }

    for dirs in paths.iter().filter_map(|path| std::fs::read_dir(path).ok()) {
        for path in dirs
            .filter_map(|try_dir| try_dir.ok())
            .map(|entry| entry.path())
            .filter(is_cache)
        {
            println!("{:#?}", path);
            if matches.get_flag("execute") {
                std::fs::remove_dir_all(path)?;
            }
        }
    }
    Ok(())
}

fn walk_dirs(path: &std::path::PathBuf, paths: &mut Vec<std::path::PathBuf>, mut depth: usize) {
    depth += 1;
    if depth > 16 {
        return;
    }

    let Ok(dirs) = std::fs::read_dir(path) else {
        return;
    };

    let mut contains_cache = false;
    for sub_dir in dirs
        .filter_map(|try_dir| try_dir.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
    {
        let is_cache = is_cache(&sub_dir);
        if !is_cache {
            walk_dirs(&sub_dir, paths, depth);
        }
        contains_cache |= is_cache;
    }

    if contains_cache {
        paths.push(path.clone());
    }
}

fn is_cache(path: &PathBuf) -> bool {
    path.ends_with("target")
        && std::fs::read_dir(path)
            .map(|dirs| {
                dirs.filter_map(|p| p.ok())
                    .any(|p| p.file_name() == "CACHEDIR.TAG")
            })
            .unwrap_or_default()
}
