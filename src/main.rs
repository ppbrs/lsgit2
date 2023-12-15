use colored::Colorize;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path;
use std::process::{Command, Stdio};
use regex::Regex;
use std::str::FromStr;

fn check_dir(dir_path: &std::path::Path, check_dir_cnt: &mut i32, git_dir_vec: &mut Vec<std::path::PathBuf>) {
    // An attribute of a git repository is a .git directory or a .git file.
    *check_dir_cnt = *check_dir_cnt + 1;

    let entries = dir_path.read_dir().unwrap();
    for entry in entries {
    //     let entry = entry.unwrap();
        let path = entry.unwrap().path();
        if path.file_name().unwrap() == ".git" {
            // println!("GIT: {:?}", path.parent());
            git_dir_vec.push(std::path::PathBuf::from_str( path.parent().unwrap().to_str().unwrap() ).unwrap());
        } else if path.is_dir() {
            check_dir(path.as_path(), check_dir_cnt, git_dir_vec);
        }
    }
}

fn main() {
    // Print basic information about the application.
    let _exe_path_buf = env::current_exe().unwrap();
    let exe_path_str = _exe_path_buf.to_str().unwrap();
    let _cwd_path_buf = env::current_dir().unwrap();
    let cwd_path_str = _cwd_path_buf.to_str().unwrap();
    println!("lsgit2");
    println!("\tStarted from `{}`.", exe_path_str);
    println!("\tCurrent working directory: `{}`.", cwd_path_str);

    // Only two optional argument are expected being the directory where to start the search
    // and the regexp pattern for paths and branch names.
    let args: Vec<String> = env::args().collect();
    let err_msg: String;
    let start_dir = if args.len() == 1 {
        Ok(env::current_dir().unwrap())
    } else if args.len() == 2 || args.len() == 3 {
        let sd = path::PathBuf::from(&args[1]);
        if !sd.as_path().exists() {
            err_msg = format!("Path `{}` doesn't exist.", sd.to_str().unwrap());
            Err(&err_msg[..])
        } else if !sd.as_path().is_dir() {
            err_msg = format!("Path `{}` is not a directory.", sd.to_str().unwrap());
            Err(&err_msg[..])
        } else {
            Ok(sd)
        }
    } else {
        err_msg = format!("Expecting 0, 1, or 2 arguments, {} arguments were given.", args.len());
        Err(&err_msg[..])
    };
    let start_dir = fs::canonicalize(&start_dir.unwrap()); // convert from relative to absolute
    println!("\tStarting searching from `{}`.", start_dir.as_ref().unwrap().to_str().unwrap());
    // start_dir is Result<PathBuf, Error>
    // start_dir.as_ref() converts from &Result<T, E> to Result<&T, &E>.

    let regex_pattern = if args.len() == 3 {
        Regex::new(&args[2]).unwrap()
    } else {
        Regex::new(".*").unwrap()
    };

    let mut check_dir_cnt: i32 = 0;
    let mut repo_abs_paths: Vec<std::path::PathBuf> = Vec::new();
    check_dir(start_dir.as_ref().unwrap().as_path(), &mut check_dir_cnt, &mut repo_abs_paths);
    repo_abs_paths.sort();use std::str;

    match check_dir_cnt {
        0 => println!("\tNo directories were checked."),
        1 => println!("\t1 directory was checked."),
        _ => println!("\t{} directories were checked.", check_dir_cnt)
    }
    let num_repos = repo_abs_paths.len();
    match num_repos {
        0 => println!("\tNo git repositories were found."),
        1 => println!("\t1 git repository was found."),
        _ => println!("\t{} git repositories were found.", num_repos)
    }

    let start_dir_comp: Vec<&OsStr> = start_dir.as_ref().unwrap().iter().collect();

    // Fetch repository updates in the background.
    for repo_path_abs in repo_abs_paths.iter() {
        let _ = Command::new("git").args(["-C", repo_path_abs.to_str().unwrap(), "fetch"])
            .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn();
    }

    // Collect repo statuses.
    let mut repo_statuses: Vec<String> = Vec::new();
    for repo_abs_path in repo_abs_paths.iter() {

        let status_stdout = Command::new("git").args(["-C", repo_abs_path.to_str().unwrap(), "status", "--branch", "--short"]).output().unwrap().stdout;
        let status_str = str::from_utf8(&status_stdout).unwrap();
        let status_str: Vec<&str> = status_str.split("\n").collect();
        let status_str = status_str[0]; // For example "## HEAD (no branch)", or "## master...origin/master".
        repo_statuses.push(status_str.to_string());
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
        let repo_rel_path: &std::path::PathBuf= &repo_rel_paths[n];
        let repo_status: &str = &repo_statuses[n];
        if regex_pattern.is_match(repo_rel_path.to_str().unwrap()) || regex_pattern.is_match(repo_status) {
            selected.push(true);
            let repo_rel_path_comps: Vec<&OsStr> = repo_rel_path.iter().collect();
            if repo_rel_path_comps.len() == 0 {
                current_dir_is_repo = true;
                let repo_abs_path = &repo_abs_paths[n];
                let repo_abs_path_comps: Vec<&OsStr> = repo_abs_path.iter().collect();
                current_dir_repo_name = repo_abs_path_comps[repo_abs_path_comps.len() - 1].to_str().unwrap().to_string();
            }
        } else {
            selected.push(false);
        }
    }

    let num_selected = selected.iter().filter(|&n| *n == true).count();
    match num_selected {
        0 => println!("\tNo git repositories were selected."),
        1 => println!("\t1 git repository was selected."),
        _ => println!("\t{} git repositories were selected.", num_selected)
    }
    if current_dir_is_repo {
        println!("\tCurrent directory ({}) is a repository.", current_dir_repo_name);
    }
    println!();

    // Sanity check.
    if repo_abs_paths.len() != repo_rel_paths.len() || repo_abs_paths.len() != selected.len() || repo_abs_paths.len() != repo_statuses.len() {
        panic!("There is a problem with logic.");
    }

    for n in 0..repo_rel_paths.len() {
        if selected[n] {
            let repo_rel_path: &std::path::PathBuf= &repo_rel_paths[n];
            let repo_status: &str = &repo_statuses[n];

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
                repo_dir_str = repo_rel_path_comps[repo_rel_path_comps.len()-1].to_str().unwrap();
            } else {
                // The starting directory is a repository itself.
                let repo_abs_path_comps: Vec<&OsStr> = repo_abs_paths[n].iter().collect();
                repo_dir_str = repo_abs_path_comps[repo_abs_path_comps.len() - 1].to_str().unwrap();
            }
            println!("{}{} {}.", indent, repo_dir_str.white().bold(), repo_status.green());
        }
    }
}
