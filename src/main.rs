use std::process::Command;

#[derive(Debug)]
struct GPU {
    index: String,
    name: String,
    compute_cap: String,
    power_draw: String,
    temperature: String,
    gpu_memory_used: String,
    gpu_memory_total: String,
    gpu_memory_utilization: String,
    gpu_utilization: String,
    pid: String,
    user: String,
    cpu_utilization: String,
    memory_utilization: String,
    elapsed_time: String,
    command: String,
}
enum Host {
    Local,
    Remote { hostname: String },
}

#[derive(Debug)]
struct MachineQuery {
    gpu_stats: String,
    gpu_processes: String,
    hostname: String,
    ps_output: String,
}

impl MachineQuery {
    fn new(host: Host) -> MachineQuery {
        let hostname = match host {
            Host::Local => {
                let name = Command::new("hostname")
                    .output()
                    .expect("failed to execute hostname")
                    .stdout;
                String::from_utf8(name).unwrap()
            }
            Host::Remote { hostname } => hostname,
        };
        let gpu_stats = Command::new("nvidia-smi")
            .arg("--query-gpu=timestamp,count,driver_version,index,name,compute_cap,power.draw,temperature.gpu,memory.used,memory.total,utilization.memory,utilization.gpu")
            .arg("--format=csv,noheader")
            .output()
            .expect("failed to execute nvidia-smi");
        let gpu_stats = String::from_utf8(gpu_stats.stdout).unwrap();

        let gpu_processes = Command::new("nvidia-smi")
            .arg("--query-compute-apps=pid")
            .arg("--format=csv,noheader")
            .output()
            .expect("failed to execute nvidia-smi");
        let gpu_processes = String::from_utf8(gpu_processes.stdout).unwrap();

        // for each PID in gpu_processes, run ps -p PID -o "pid,user,%cpu,%mem,etime,command"
        let mut ps_output = String::new();
        for pid in gpu_processes.split('\n') {
            if pid.len() > 0 {
                let ps = Command::new("ps")
                    .arg("-p")
                    .arg(pid)
                    .arg("-o")
                    .arg("pid=,user=,%cpu=,%mem=,etime=,command=")
                    .output()
                    .expect("failed to execute ps");
                ps_output.push_str(&String::from_utf8(ps.stdout).unwrap());
                // ps_output.push('\n');
            }
        }
        MachineQuery {
            gpu_stats,
            gpu_processes,
            hostname,
            ps_output,
        }
    }
}

#[derive(Debug)]
struct Machine {
    hostname: String,
    timestamp: String,
    num_gpus: String,
    driver_version: String,
    gpus: Vec<GPU>,
}

impl Machine {
    fn from_machine_query(query: MachineQuery) -> Machine {
        // get timestamp, driver_version, count from start of first line
        let split = query.gpu_stats.split(',');
        let machine_stats = split.take(3).collect::<Vec<&str>>();

        let stats_lines = query.gpu_stats.split('\n'); // gpus are separated by newlines
        let processes_lines = query.ps_output.split('\n');
        let mut gpus = Vec::new();

        for (stats_line, process_line) in stats_lines.zip(processes_lines) {
            if stats_line.len() > 0 && process_line.len() > 0 {
                gpus.push(GPU::from_stats_and_process(
                    stats_line.to_string(),
                    process_line.to_string(),
                ));
            }
        }
        Machine {
            hostname: query.hostname,
            timestamp: machine_stats[0].to_string(),
            num_gpus: machine_stats[1].to_string(),
            driver_version: machine_stats[2].to_string(),
            gpus,
        }
    }
}

impl GPU {
    fn from_stats_and_process(stats: String, process: String) -> GPU {
        let stats_split = stats.split(',');
        let mut stats_split = stats_split.skip(3); // skip timestamp, count, driver_version

        // process is not comma separated, so we need to split on whitespace
        let mut process_split = process.split_whitespace();
        GPU {
            index: stats_split.next().unwrap().trim().parse().unwrap(),
            name: stats_split.next().unwrap().trim().to_string(),
            compute_cap: stats_split.next().unwrap().trim().to_string(),
            power_draw: stats_split.next().unwrap().trim().parse().unwrap(),
            temperature: stats_split.next().unwrap().trim().parse().unwrap(),
            gpu_memory_used: stats_split.next().unwrap().trim().parse().unwrap(),
            gpu_memory_total: stats_split.next().unwrap().trim().parse().unwrap(),
            gpu_memory_utilization: stats_split.next().unwrap().trim().parse().unwrap(),
            gpu_utilization: stats_split.next().unwrap().trim().parse().unwrap(),
            pid: process_split.next().unwrap().trim().parse().unwrap(),
            user: process_split.next().unwrap().trim().to_string(),
            cpu_utilization: process_split.next().unwrap().trim().parse().unwrap(),
            memory_utilization: process_split.next().unwrap().trim().parse().unwrap(),
            elapsed_time: process_split.next().unwrap().trim().to_string(),
            command: process_split.next().unwrap().trim().to_string(),
        }
    }
}

fn main() {
    let query = MachineQuery::new(Host::Local);
    println!("{:#?}", query);
    let machine = Machine::from_machine_query(query);
    println!("{:#?}", machine);
}
