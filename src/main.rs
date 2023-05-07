use clap::Parser;
use nvml_wrapper::Nvml;
use std::process::Command;
use tabled::{
    settings::locator::ByColumnName,
    settings::object::Rows,
    settings::{Disable, Modify, Width},
    Table, Tabled,
};

mod gpu;
use gpu::GPUStats;

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
            .flat_map(|gpu| gpu.process_pids.clone())
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
