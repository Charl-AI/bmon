use std::process::Command;

#[derive(Debug)]
struct GPU {
    index: String,
    name: String,
    compute_cap: String,
    power_draw: String,
    temperature: String,
    memory_used: String,
    memory_total: String,
    memory_utilization: String,
    gpu_utilization: String,
}

#[derive(Debug)]
struct Machine {
    timestamp: String,
    num_gpus: String,
    driver_version: String,
    gpus: Vec<GPU>,
}

impl Machine {
    fn from_nvidia_smi_output(output: String) -> Machine {
        // get timestamp, driver_version, count from start of first line
        let split = output.split(',');
        let machine_stats = split.take(3).collect::<Vec<&str>>();

        let lines = output.split('\n');
        let mut gpus = Vec::new();

        for line in lines {
            if line.len() > 0 {
                gpus.push(GPU::from_csv_line(line.to_string()));
            }
        }
        Machine {
            timestamp: machine_stats[0].to_string(),
            num_gpus: machine_stats[1].to_string(),
            driver_version: machine_stats[2].to_string(),
            gpus: gpus,
        }
    }
}

impl GPU {
    fn from_csv_line(output: String) -> GPU {
        let split = output.split(',');
        let mut split = split.skip(3); // skip timestamp, count, driver_version
        GPU {
            index: split.next().unwrap().trim().parse().unwrap(),
            name: split.next().unwrap().trim().to_string(),
            compute_cap: split.next().unwrap().trim().to_string(),
            power_draw: split.next().unwrap().trim().parse().unwrap(),
            temperature: split.next().unwrap().trim().parse().unwrap(),
            memory_used: split.next().unwrap().trim().parse().unwrap(),
            memory_total: split.next().unwrap().trim().parse().unwrap(),
            memory_utilization: split.next().unwrap().trim().parse().unwrap(),
            gpu_utilization: split.next().unwrap().trim().parse().unwrap(),
        }
    }
}

fn main() {
    // the nvidia-smi command here returns a csv with each query separated by a comma, and each GPU separated by a newline
    // parts of the query that are constant across the machine (e.g. driver version, count) are duplicated for each GPU
    let output = Command::new("nvidia-smi")
        .arg("--query-gpu=timestamp,count,driver_version,index,name,compute_cap,power.draw,temperature.gpu,memory.used,memory.total,utilization.memory,utilization.gpu")
        .arg("--format=csv,noheader")
        .output()
        .expect("failed to execute process");
    let output = String::from_utf8(output.stdout).unwrap();

    println!("{:?}", output);
    let machine = Machine::from_nvidia_smi_output(output);
    println!("{:?}", machine);
}
