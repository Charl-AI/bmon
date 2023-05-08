use clap::Parser;
use nvml_wrapper::Nvml;
use tabled::{
    settings::{Extract, Panel, Style},
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
        let mut table = Table::new(&self.gpus);

        if !verbose {
            // only display the first 6 columns in non-verbose mode
            table.with(Extract::segment(0.., 0..6));
        }
        table.with(Panel::header(format!(
            "Driver Version: {}       CUDA Version: {}",
            self.driver_version, self.cuda_version
        )));

        table.with(Style::re_structured_text());
        println!("\nGPU Usage:");
        println!("{}", table);
    }

    fn display_cpu_stats(&self, _verbose: bool) {
        let (iowait, steal, idle) = get_io_stats();

        let mut table = Table::new(&self.processes);

        table.with(Panel::header(format!(
            "Num CPUs: {}  RAM Capacity: {}  IO Wait: {}  Steal: {}  Idle: {}",
            self.num_cpus, self.ram_capacity, iowait, steal, idle
        )));

        table.with(Style::re_structured_text());
        println!("\nCPU Usage:");
        println!("{}", table);
    }

    fn display_bottleneck_diagnostics(&self) {
        print!("\x1b[31m"); // make throttling reasons red
        println!("\nBottleneck diagnosis:");
        for gpu in &self.gpus {
            if gpu.throttling.is_empty() {
                continue;
            }
            println!("GPU {} is throttling due to: {:?}", gpu.idx, gpu.throttling);
        }
        print!("\x1b[0m"); // reset color
    }
}

const PKG_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const PKG_DESC: &str = env!("CARGO_PKG_DESCRIPTION");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(author=PKG_AUTHORS, version=PKG_VERSION, about=PKG_DESC)]
struct Args {
    /// Displays all possible stats, equivalent to -bcdnv
    #[arg(short, long, default_value = "false")]
    all: bool,

    /// Whether to display bottleneck diagnosis. Defaults to false.
    #[arg(short, long, default_value = "false")]
    bottleneck: bool,

    /// Whether to display CPU stats. Defaults to false.
    #[arg(short, long, default_value = "false")]
    cpu: bool,

    /// Whether to display disk stats. Defaults to false.
    #[arg(short, long, default_value = "false")]
    disk: bool,

    /// Whether to display network stats. Defaults to false.
    #[arg(short, long, default_value = "false")]
    network: bool,

    /// Whether to display extra information. Defaults to false.
    #[arg(short, long, default_value = "false")]
    verbose: bool,
}

fn main() {
    let args: Args = Args::parse();
    let machine = Machine::new();

    machine.display_gpu_stats(args.verbose);

    if args.disk || args.all {
        unimplemented!("Disk stats not implemented yet");
    }

    if args.network || args.all {
        unimplemented!("Network stats not implemented yet");
    }

    if args.cpu || args.all {
        machine.display_cpu_stats(args.verbose);
    }

    if args.bottleneck || args.all {
        machine.display_bottleneck_diagnostics();
    }
}
