use clap::Parser;
use nvml_wrapper::{
    bitmasks::device::ThrottleReasons, enum_wrappers::device::TemperatureSensor, Device, Nvml,
};
use std::process::Command;
use tabled::{
    settings::locator::ByColumnName,
    settings::object::Rows,
    settings::{Disable, Modify, Width},
    Table, Tabled,
};

#[derive(Tabled)]
#[tabled(rename_all = "UPPERCASE")]
struct GPUStats {
    idx: u32,
    name: String,
    temp: String,
    power: String,
    utilizations: String,
    memory: String,

    // these are not displayed unless verbose is true
    capability: String,
    cores: u32,
    fan: String,
    display: String,

    #[tabled(skip)]
    throttling: ThrottleReasons,

    #[tabled(skip)]
    compute_process_pids: Vec<u32>,
    processes: String, // same as above but in a diasplayable format
}

impl GPUStats {
    fn from_nvml_device(device: Device) -> Self {
        let idx = device.index().unwrap();
        let name = device.name().unwrap();

        let temp = device.temperature(TemperatureSensor::Gpu).unwrap();
        let temp = format!("{}°C", temp);

        let power_usage = device.power_usage().unwrap();
        let power_limit = device.enforced_power_limit().unwrap();
        let power = format!(
            "{:.0}W/{:.0}W",
            power_usage as f32 / 1000.0,
            power_limit as f32 / 1000.0
        );

        let gpu_utilization = device.utilization_rates().unwrap().gpu;
        let memory_utilization = device.utilization_rates().unwrap().memory;
        let utilizations = format!("GPU {}% VRAM {}%", gpu_utilization, memory_utilization);

        let memory_used = device.memory_info().unwrap().used;
        let memory_total = device.memory_info().unwrap().total;
        let memory = format!(
            "{:.2}GB/{:.2}GB",
            memory_used as f32 / 1024.0 / 1024.0 / 1024.0,
            memory_total as f32 / 1024.0 / 1024.0 / 1024.0
        );

        let compute_cap = device.cuda_compute_capability().unwrap();
        let capability = format!("{}.{}", compute_cap.major, compute_cap.minor);
        let cores = device.num_cores().unwrap();

        let throttling = device.current_throttle_reasons().unwrap();

        let n_fans = device.num_fans().unwrap();
        let fan = if n_fans == 0 {
            "N/A".to_string()
        } else {
            // fans reports average speed of all fans
            let mut sum_fans = 0;
            for i in 0..n_fans {
                sum_fans += device.fan_speed(i).unwrap();
            }
            format!("{}%", sum_fans / n_fans)
        };

        let display_connected = device.is_display_connected().unwrap();
        let display_active = device.is_display_active().unwrap();
        let display = if display_active {
            "Active".to_string()
        } else if display_connected {
            "Connected".to_string()
        } else {
            "None".to_string()
        };

        let compute_processes = device.running_compute_processes().unwrap();
        let compute_process_pids = compute_processes
            .iter()
            .map(|process| process.pid)
            .collect::<Vec<u32>>();
        let processes = compute_processes
            .iter()
            .map(|process| process.pid.clone().to_string())
            .collect::<Vec<String>>()
            .join(", ");

        Self {
            idx,
            name,
            temp,
            power,
            utilizations,
            memory,

            capability,
            cores,
            fan,
            display,

            throttling,

            compute_process_pids,
            processes,
        }
    }
}

#[derive(Tabled)]
#[tabled(rename_all = "UPPERCASE")]
struct ProcessStats {
    pid: u32,
    user: String,
    utilizations: String,
    elapsed: String,
    command: String,
}

impl ProcessStats {
    fn from_pid(pid: u32) -> Self {
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

        let utilizations = format!("CPU {}% MEM {}%", cpu_utilization, memory_utilization);

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

struct Machine {
    cuda_version: String,
    driver_version: String,
    gpus: Vec<GPUStats>,
    processes: Vec<ProcessStats>,
    num_cpus: String,
    ram_capacity: String,
}

impl Machine {
    fn new() -> Self {
        let nvml = Nvml::init().unwrap();

        // NB: cuda version begins as an int e.g. 12000
        // this is converted to a float e.g. 12.0
        let cuda_version = nvml.sys_cuda_driver_version().unwrap();
        let cuda_version = cuda_version as f32 / 1000.0;
        let cuda_version = format!("{:.1}", cuda_version);

        let driver_version = nvml.sys_driver_version().unwrap();

        let mut gpus: Vec<GPUStats> = vec![];
        let num_gpus = nvml.device_count().unwrap();
        for i in 0..num_gpus {
            let device = nvml.device_by_index(i).unwrap();
            let gpu = GPUStats::from_nvml_device(device);
            gpus.push(gpu);
        }
        let gpu_process_pids = gpus
            .iter()
            .flat_map(|gpu| gpu.compute_process_pids.clone())
            .collect::<Vec<u32>>();

        let processes = gpu_process_pids
            .iter()
            .map(|pid| ProcessStats::from_pid(*pid))
            .collect::<Vec<ProcessStats>>();

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

        Self {
            cuda_version,
            driver_version,
            gpus,
            processes,
            num_cpus,
            ram_capacity,
        }
    }

    fn display_gpu_stats(&self, verbose: bool) {
        println!(
            "CUDA Version {} | Driver Version {}",
            self.cuda_version, self.driver_version
        );
        let mut table = Table::new(&self.gpus);
        if !verbose {
            table
                .with(Disable::column(ByColumnName::new("CAPABILITY")))
                .with(Disable::column(ByColumnName::new("CORES")))
                .with(Disable::column(ByColumnName::new("FAN")))
                .with(Disable::column(ByColumnName::new("DISPLAY")))
                .with(Disable::column(ByColumnName::new("PROCESSES")))
                .with(Disable::row(Rows::first()));
        }
        println!("{}", table);

        if !verbose {
            return;
        }

        for gpu in &self.gpus {
            if gpu.throttling.is_empty() {
                continue;
            }
            print!("\x1b[31m"); // make throttling reasons red
            println!("GPU {} is throttling due to: {:?}", gpu.idx, gpu.throttling);
            print!("\x1b[0m"); // reset color
        }
    }

    fn display_cpu_stats(&self, verbose: bool) {
        println!(
            "Num CPUs {} | RAM Capacity {}",
            self.num_cpus, self.ram_capacity
        );
        // check if there are any processes running on the GPU
        if self.processes.is_empty() {
            println!("No compute processes running on GPU");
            return;
        }

        let mut table = Table::new(&self.processes);

        let truncate_width = if verbose { 75 } else { 20 };
        table.with(Modify::new(Rows::new(0..)).with(Width::truncate(truncate_width).suffix("...")));
        if !verbose {
            table.with(Disable::row(Rows::first()));
        }
        println!("{}", table);
    }
}

#[derive(Parser)]
struct Args {
    /// Whether to display extra information, including
    /// bottleneck diagnosis information
    #[arg(short, long, default_value = "false")]
    verbose: bool,

    /// Whether to display GPU stats
    #[arg(short, long, default_value = "true")]
    gpu: bool,

    /// Whether to display CPU stats
    #[arg(short, long, default_value = "false")]
    cpu: bool,

    /// Whether to display network stats
    #[arg(short, long, default_value = "false")]
    network: bool,

    /// Whether to display disk stats
    #[arg(short, long, default_value = "false")]
    disk: bool,
}

fn main() {
    let args: Args = Args::parse();
    let machine = Machine::new();
    machine.display_gpu_stats(args.verbose);
    machine.display_cpu_stats(args.verbose);
}
