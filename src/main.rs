use std::env;
use std::path;

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
    println!("Starting searching from `{}`", start_dir.unwrap().to_str().unwrap());
}
