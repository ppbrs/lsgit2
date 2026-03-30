use std::str::FromStr;

/// Recursively scan a directory for Git checkouts.
/// If a directory is a git repository checkout, add its path to git_dir_vec.
/// Increment scan_dir_cnt for each checked directory.
/// # Arguments
/// *
/// * max_depth: 0 if limited, >0 if limited.
pub fn scan_dir(
    dir_path: &std::path::Path,
    start_dir_path: &std::path::Path,
    max_depth: usize,
    scan_dir_cnt: &mut i32,
    git_dir_vec: &mut Vec<std::path::PathBuf>,
) {
    // An attribute of a git repository is a .git directory or a .git file.
    *scan_dir_cnt = *scan_dir_cnt + 1;

    let entries = dir_path.read_dir().unwrap();
    for entry in entries {
        //     let entry = entry.unwrap();
        let path = entry.unwrap().path();
        if path.file_name().unwrap() == ".git" {
            // println!("GIT: {:?}", path.parent());
            git_dir_vec.push(
                std::path::PathBuf::from_str(path.parent().unwrap().to_str().unwrap()).unwrap(),
            );
        } else if path.is_dir() {
            let dir_rel_path = path.as_path().strip_prefix(&start_dir_path).unwrap();
            let dir_rel_depth = dir_rel_path.components().count();
            // println!("{:?}, {:?}", dir_rel_path, dir_rel_depth);
            if max_depth != 0 && dir_rel_depth <= max_depth {
                scan_dir(
                    path.as_path(),
                    start_dir_path,
                    max_depth,
                    scan_dir_cnt,
                    git_dir_vec,
                );
            }
        }
    }
}
