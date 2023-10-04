use colored::Colorize;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path;
use std::process::Command;
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
    println!("\tstarted from `{}`", exe_path_str);
    println!("\tcurrent working directory: `{}`", cwd_path_str);

    // Only one optional argument is expected being the directory where to start the search.
    let args: Vec<String> = env::args().collect();
    let err_msg: String;
    let start_dir = if args.len() == 1 {
        Ok(env::current_dir().unwrap())
    } else if args.len() == 2 {
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
        err_msg = format!("Expecting 0 or 1 arguments, {} were given.", args.len());
        Err(&err_msg[..])
    };
    let start_dir = fs::canonicalize(&start_dir.unwrap()); // convert from relative to absolute
    println!("Starting searching from `{}`", start_dir.as_ref().unwrap().to_str().unwrap());
    // start_dir is Result<PathBuf, Error>
    // start_dir.as_ref() converts from &Result<T, E> to Result<&T, &E>.

    let mut check_dir_cnt: i32 = 0;
    let mut git_dir_vec: Vec<std::path::PathBuf> = Vec::new();
    check_dir(start_dir.as_ref().unwrap().as_path(), &mut check_dir_cnt, &mut git_dir_vec);
    git_dir_vec.sort();use std::str;

    println!("{} directories were checked, {} git repo(s) were found:", check_dir_cnt, git_dir_vec.len());

    let start_dir_comp: Vec<&OsStr> = start_dir.as_ref().unwrap().iter().collect();

    for git_dir in git_dir_vec.iter() {

        let status_stdout = Command::new("git").args(["-C", git_dir.to_str().unwrap(), "status", "--branch", "--short"]).output().unwrap().stdout;
        let status_str = str::from_utf8(&status_stdout).unwrap();
        let status_str: Vec<&str> = status_str.split("\n").collect();
        let status_str = status_str[0]; // For example "## HEAD (no branch)", or "## master...origin/master".

        let git_dir_comp: Vec<&OsStr> = git_dir.iter().collect();
        let mut git_dir = std::path::PathBuf::new();
        for n in start_dir_comp.len()..git_dir_comp.len() {
            git_dir.push(git_dir_comp[n]);
        }

        // git_dir is an absolute path to the git repository, for example /home/boris/projects/lsgit2
        // start_dir is an absolute path to the search directory, for example /home/boris/projects/
        let mut indent = String::from("");
        for i in start_dir_comp.len()..(git_dir_comp.len() - 1) {
            // indent.push('\t');
            indent += git_dir_comp[i].to_str().unwrap();
            indent += "/";
        }
        // let git_dir_str = git_dir.to_str().unwrap();
        let git_dir_str = git_dir_comp[git_dir_comp.len()-1].to_str().unwrap();
        println!("{}{} {}.", indent, git_dir_str.white().bold(), status_str.green());
    }
}
