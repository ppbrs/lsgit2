use clap::Parser;
use colored::Colorize;
use regex::Regex;
use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process;
use std::process::{Command, Stdio};

mod cli;
mod scan_dir;
use cli::CLI;
use scan_dir::scan_dir;

fn main() {
    let cli: CLI = CLI::parse();

    // Print basic information about the application.
    let _exe_path_buf = env::current_exe().unwrap();
    let exe_path_str = _exe_path_buf.to_str().unwrap();
    let _cwd_path_buf = env::current_dir().unwrap();
    let cwd_path_str = _cwd_path_buf.to_str().unwrap();
    println!("lsgit2 v{}", env!("CARGO_PKG_VERSION"));
    println!("\tStarted from `{}`.", exe_path_str);
    println!("\tCurrent working directory: `{}`.", cwd_path_str);
    println!("\tDepth of search: {}.", cli.depth);

    // Figure out the start directory.
    let current_dir: PathBuf = env::current_dir().unwrap();
    let mut start_dir = current_dir.clone();
    start_dir.push(cli.start_dir);
    start_dir = match start_dir.canonicalize() {
        Ok(value) => value,
        Err(e) => {
            eprintln!("ERROR: '{}': {}.", start_dir.as_path().to_str().unwrap(), e);
            process::exit(1);
        }
    };
    if !start_dir.exists() {
        eprintln!(
            "ERROR: '{}': Directory does not exist.",
            start_dir.as_path().to_str().unwrap()
        );
        process::exit(1);
    }
    if !start_dir.is_dir() {
        eprintln!(
            "ERROR: '{}': Not a directory.",
            start_dir.as_path().to_str().unwrap()
        );
        process::exit(2);
    }
    println!(
        "\tStarting searching from `{}`.",
        start_dir.to_str().unwrap()
    );

    // Figure out the pattern.
    let regex_pattern = match cli.regex_pattern {
        None => Regex::new(".*").unwrap(),
        Some(val) => Regex::new(&val).unwrap(),
    };

    // Start scanning.
    let mut scan_dir_cnt: i32 = 0;
    let mut repo_abs_paths: Vec<std::path::PathBuf> = Vec::new();
    let start_dir_path = start_dir.as_path();
    scan_dir(
        start_dir_path,
        start_dir_path,
        cli.depth,
        &mut scan_dir_cnt,
        &mut repo_abs_paths,
    );
    repo_abs_paths.sort();
    use std::str;

    match scan_dir_cnt {
        0 => println!("\tNo directories were checked."),
        1 => println!("\t1 directory was checked."),
        _ => println!("\t{} directories were checked.", scan_dir_cnt),
    }
    let num_repos = repo_abs_paths.len();
    match num_repos {
        0 => println!("\tNo git repositories were found."),
        1 => println!("\t1 git repository was found."),
        _ => println!("\t{} git repositories were found.", num_repos),
    }

    let start_dir_comp: Vec<&OsStr> = start_dir.iter().collect();

    // Fetch repository updates in the background.
    for repo_path_abs in repo_abs_paths.iter() {
        let _ = Command::new("git")
            .args(["-C", repo_path_abs.to_str().unwrap(), "fetch"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
    }

    // Collect repo statuses.
    let mut repo_snapshots: Vec<String> = Vec::new();
    for repo_abs_path in repo_abs_paths.iter() {
        let status_stdout = Command::new("git")
            .args([
                "-c",
                "color.ui=always",
                "-C",
                repo_abs_path.to_str().unwrap(),
                "status",
                "--branch",
                "--short",
            ])
            .output()
            .unwrap()
            .stdout;
        let status_str = str::from_utf8(&status_stdout).unwrap();
        let status_str: Vec<&str> = status_str.split("\n").collect();
        let status_str = status_str[0]; // For example "## HEAD (no branch)", or "## master...origin/master".

        let last_commit_stdout = Command::new("git").args(
            [
                "-c", "color.ui=always",
                "-C", repo_abs_path.to_str().unwrap(),
                "log", "-1",
                "--pretty=format:%C(yellow)%h%C(reset), %s, %C(blue)%ae%C(reset), %C(magenta)%aI, %ar%C(reset)"
            ]
        ).output().unwrap().stdout;
        let last_commit_str = str::from_utf8(&last_commit_stdout).unwrap();
        let last_commit_str: Vec<&str> = last_commit_str.split("\n").collect();
        let last_commit_str = last_commit_str[0]; // For example "159e31b6 Prepare 2.1.2 release"

        let mut repo_snapshot = String::new();
        repo_snapshot.push_str(status_str);
        repo_snapshot.push_str(", ");
        repo_snapshot.push_str(last_commit_str);
        repo_snapshots.push(repo_snapshot);
    }

    // Collect relative repo paths.
    let mut repo_rel_paths: Vec<std::path::PathBuf> = Vec::new();
    for repo_abs_path in repo_abs_paths.iter() {
        // Remove parent path from the repository path.
        let repo_abs_path_comps: Vec<&OsStr> = repo_abs_path.iter().collect();
        let mut repo_rel_path = std::path::PathBuf::new();
        for n in start_dir_comp.len()..repo_abs_path_comps.len() {
            repo_rel_path.push(repo_abs_path_comps[n]);
        }
        repo_rel_paths.push(repo_rel_path);
    }

    let mut selected: Vec<bool> = Vec::new();
    let mut current_dir_is_repo = false;
    let mut current_dir_repo_name: String = "".to_string();
    for n in 0..repo_rel_paths.len() {
        let repo_rel_path: &std::path::PathBuf = &repo_rel_paths[n];
        let repo_status: &str = &repo_snapshots[n];
        if regex_pattern.is_match(repo_rel_path.to_str().unwrap())
            || regex_pattern.is_match(repo_status)
        {
            selected.push(true);
            let repo_rel_path_comps: Vec<&OsStr> = repo_rel_path.iter().collect();
            if repo_rel_path_comps.len() == 0 {
                current_dir_is_repo = true;
                let repo_abs_path = &repo_abs_paths[n];
                let repo_abs_path_comps: Vec<&OsStr> = repo_abs_path.iter().collect();
                current_dir_repo_name = repo_abs_path_comps[repo_abs_path_comps.len() - 1]
                    .to_str()
                    .unwrap()
                    .to_string();
            }
        } else {
            selected.push(false);
        }
    }

    let num_selected = selected.iter().filter(|&n| *n == true).count();
    match num_selected {
        0 => println!("\tNo git repositories were selected."),
        1 => println!("\t1 git repository was selected."),
        _ => println!("\t{} git repositories were selected.", num_selected),
    }
    if current_dir_is_repo {
        println!(
            "\tCurrent directory ({}) is a repository.",
            current_dir_repo_name
        );
    }
    println!();

    // Sanity check.
    if repo_abs_paths.len() != repo_rel_paths.len()
        || repo_abs_paths.len() != selected.len()
        || repo_abs_paths.len() != repo_snapshots.len()
    {
        panic!("There is a problem with logic.");
    }

    for n in 0..repo_rel_paths.len() {
        if selected[n] {
            let repo_rel_path: &std::path::PathBuf = &repo_rel_paths[n];
            let repo_snapshot: &str = &repo_snapshots[n];

            let repo_rel_path_comps: Vec<&OsStr> = repo_rel_path.iter().collect();
            let mut indent: String = String::from("");
            let repo_dir_str: &str;
            if repo_rel_path_comps.len() > 0 {
                indent += &current_dir_repo_name;
                indent += "/";
                for i in 0..(repo_rel_path_comps.len() - 1) {
                    indent += repo_rel_path_comps[i].to_str().unwrap();
                    indent += "/";
                }
                repo_dir_str = repo_rel_path_comps[repo_rel_path_comps.len() - 1]
                    .to_str()
                    .unwrap();
            } else {
                // The starting directory is a repository itself.
                let repo_abs_path_comps: Vec<&OsStr> = repo_abs_paths[n].iter().collect();
                repo_dir_str = repo_abs_path_comps[repo_abs_path_comps.len() - 1]
                    .to_str()
                    .unwrap();
            }
            println!(
                "{}{}    {}.",
                indent,
                repo_dir_str.white().bold(),
                repo_snapshot
            );
        }
    }
}
