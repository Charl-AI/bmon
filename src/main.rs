use clap::Parser;
use nvml_wrapper::Nvml;
use tabled::{
    settings::object::Rows,
    settings::{Extract, Modify, Panel, Style, Width},
    Table,
};

mod disk;
mod gpu;
mod process;
use disk::get_io_stats;
use gpu::{get_driver_stats, GPUStats};
use process::{get_cpu_stats, ProcessStats};

struct Machine {
    gpus: Vec<GPUStats>,
    processes: Vec<ProcessStats>,
    cuda_version: String,
    driver_version: String,
    num_cpus: String,
    ram_capacity: String,
    iowait: String,
    steal: String,
    idle: String,
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
        let (iowait, steal, idle) = get_io_stats();

        Self {
            gpus,
            processes,
            cuda_version,
            driver_version,
            num_cpus,
            ram_capacity,
            iowait,
            steal,
            idle,
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

    fn display_cpu_stats(&self, verbose: bool) {
        let mut table = Table::new(&self.processes);
        let truncate_width = if verbose { 75 } else { 20 };
        table.with(Modify::new(Rows::new(0..)).with(Width::truncate(truncate_width).suffix("...")));

        table.with(Panel::header(format!(
            "Num CPUs: {}  RAM Capacity: {}  IO Wait: {}  Steal: {}  Idle: {}",
            self.num_cpus, self.ram_capacity, self.iowait, self.steal, self.idle
        )));

        table.with(Style::re_structured_text());
        println!("\nCPU Usage:");
        println!("{}", table);
    }

    fn display_bottleneck_diagnostics(&self) {
        println!("\nBottleneck diagnosis:");
        for gpu in &self.gpus {
            if gpu.throttling.is_empty() {
                continue;
            }
            println!("GPU {} is throttling due to: {:?}", gpu.idx, gpu.throttling);
        }
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
