use std::process::Command;
use tabled::Tabled;

#[derive(Tabled)]
#[tabled(rename_all = "UPPERCASE")]
pub struct ProcessStats {
    pid: u32,
    user: String,
    utilizations: String,
    elapsed: String,
    command: String,
}

impl ProcessStats {
    pub fn from_pid(pid: u32) -> Self {
        let ps = Command::new("ps")
            .arg("-p")
            .arg(pid.to_string())
            .arg("-o")
            .arg("pid=,user=,%cpu=,%mem=,etime=,command=")
            .output()
            .expect("failed to execute ps command");

        let ps_output = String::from_utf8(ps.stdout).unwrap();

        let user = ps_output.split_whitespace().nth(1).unwrap().to_string();
        let cpu_utilization = ps_output.split_whitespace().nth(2).unwrap().to_string();
        let memory_utilization = ps_output.split_whitespace().nth(3).unwrap().to_string();

        let utilizations = format!("CPU {}% RAM {}%", cpu_utilization, memory_utilization);

        let elapsed = ps_output.split_whitespace().nth(4).unwrap().to_string();
        // command is everything from the 5th word onwards
        let mut command = String::new();
        for (i, word) in ps_output.split_whitespace().enumerate() {
            if i < 5 {
                continue;
            }
            command.push_str(word);
            command.push(' ');
        }

        Self {
            pid,
            user,
            utilizations,
            elapsed,
            command,
        }
    }
}

pub fn get_cpu_stats() -> (String, String) {
    let nproc = Command::new("nproc")
        .output()
        .expect("failed to execute nproc command");
    let num_cpus = String::from_utf8(nproc.stdout)
        .unwrap()
        .strip_suffix('\n')
        .unwrap()
        .to_string();

    let free = Command::new("free")
        .arg("-h")
        .output()
        .expect("failed to execute free command");
    let free_output = String::from_utf8(free.stdout).unwrap();
    let ram_capacity = free_output.split_whitespace().nth(7).unwrap().to_string();

    (num_cpus, ram_capacity)
}
