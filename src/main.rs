use nvml_wrapper::{Device, Nvml};

use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use tabled::{
    settings::locator::ByColumnName, settings::object::Rows, settings::Disable, Table, Tabled,
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
    throttling: String,
    fan: String,
    display: String,

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
        let utilizations = format!("GPU {}% MEM {}%", gpu_utilization, memory_utilization);

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

        // let throttle_reasons = device.current_throttle_reasons().unwrap();
        // let throttling = if throttle_reasons.is_empty() {
        //     "None".to_string()
        // } else {
        //     throttle_reasons
        //         .iter()
        //         .map(|reason| format!("{:?}", reason))
        //         .collect::<Vec<String>>()
        //         .join(", ")
        // };
        let throttling = "N/A".to_string();

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
            throttling,
            fan,
            display,

            compute_process_pids,
            processes,
        }
    }
}

struct Machine {
    cuda_version: String,
    driver_version: String,
    gpus: Vec<GPUStats>,
    gpu_compute_process_pids: Vec<u32>,
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
        let gpu_compute_process_pids = gpus
            .iter()
            .flat_map(|gpu| gpu.compute_process_pids.clone())
            .collect::<Vec<u32>>();

        Self {
            cuda_version,
            driver_version,
            gpus,
            gpu_compute_process_pids,
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
                .with(Disable::column(ByColumnName::new("THROTTLING")))
                .with(Disable::column(ByColumnName::new("FAN")))
                .with(Disable::column(ByColumnName::new("DISPLAY")))
                .with(Disable::column(ByColumnName::new("PROCESSES")))
                .with(Disable::row(Rows::first()));
        }
        println!("{}", table);
    }
}

fn main() {
    let machine = Machine::new();
    let verbose = true;
    machine.display_gpu_stats(verbose);
}
