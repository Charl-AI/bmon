use clap::Parser;
use nvml_wrapper::Nvml;
use tabled::{
    settings::object::Rows,
    settings::{Disable, Extract, Modify, Style, Width},
    Table,
};

mod gpu;
mod process;
use gpu::{get_driver_stats, GPUStats};
use process::{get_cpu_stats, ProcessStats};

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

        let (cuda_version, driver_version) = get_driver_stats(&nvml);

        let mut gpus: Vec<GPUStats> = vec![];
        let num_gpus = nvml.device_count().unwrap();
        for i in 0..num_gpus {
            let device = nvml.device_by_index(i).unwrap();
            let gpu = GPUStats::from_nvml_device(device);
            gpus.push(gpu);
        }
        let gpu_process_pids = gpus
            .iter()
            .flat_map(|gpu| gpu.processes.clone())
            .collect::<Vec<u32>>();

        let processes = gpu_process_pids
            .iter()
            .map(|pid| ProcessStats::from_pid(*pid))
            .collect::<Vec<ProcessStats>>();

        let (num_cpus, ram_capacity) = get_cpu_stats();

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
            // only display the first 6 columns in non-verbose mode
            table.with(Extract::segment(0.., 0..6));
        }
        table.with(Style::re_structured_text());
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
