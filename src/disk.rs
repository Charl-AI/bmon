use std::process::Command;

pub fn get_io_stats() -> (String, String, String) {
    let iostat = Command::new("iostat")
        .arg("-c")
        .output()
        .expect("failed to execute iostat command");

    let iostat_output = String::from_utf8(iostat.stdout).unwrap();

    let iowait = iostat_output
        .split_whitespace()
        .rev()
        .nth(2)
        .unwrap()
        .to_string();
    let iowait = format!("{}%", iowait);

    let steal = iostat_output
        .split_whitespace()
        .rev()
        .nth(1)
        .unwrap()
        .to_string();

    let steal = format!("{}%", steal);
    let idle = iostat_output.split_whitespace().last().unwrap().to_string();
    let idle = format!("{}%", idle);

    (iowait, steal, idle)
}
