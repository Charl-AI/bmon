use nvml_wrapper::{Device, Nvml};

use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use tabled::{settings::object::Rows, settings::Disable, Table, Tabled};

#[derive(Tabled)]
struct GPUStats {
    idx: u32,
    name: String,
    temp: String,
    power: String,
    utilization_rates: String,
    memory: String,
}

impl GPUStats {
    fn from_nvml_device(device: Device) -> Self {
        let idx = device.index().unwrap();
        let name = device.name().unwrap();

        let temp = device.temperature(TemperatureSensor::Gpu).unwrap();
        let temp = format!("{}°C", temp);

        let power_usage = device.power_usage().unwrap();
        let power_limit = device.enforced_power_limit().unwrap();
        // format power in W from mW
        let power = format!(
            "{:.0}W/{:.0}W",
            power_usage as f32 / 1000.0,
            power_limit as f32 / 1000.0
        );

        let gpu_utilization = device.utilization_rates().unwrap().gpu;
        let memory_utilization = device.utilization_rates().unwrap().memory;
        let utilization_rates = format!("GPU {}% MEM {}%", gpu_utilization, memory_utilization);

        let memory_used = device.memory_info().unwrap().used;
        let memory_total = device.memory_info().unwrap().total;
        let memory = format!(
            "{:.2}GB/{:.2}GB",
            memory_used as f32 / 1024.0 / 1024.0 / 1024.0,
            memory_total as f32 / 1024.0 / 1024.0 / 1024.0
        );
        Self {
            idx,
            name,
            temp,
            power,
            utilization_rates,
            memory,
        }
    }
}

fn main() {
    let nvml = Nvml::init().unwrap();
    let device_count = nvml.device_count().unwrap();
    println!("Device count: {}", device_count);

    let mut gpus: Vec<GPUStats> = vec![];
    for i in 0..device_count {
        let device = nvml.device_by_index(i).unwrap();
        let gpu = GPUStats::from_nvml_device(device);
        gpus.push(gpu);
    }

    let mut table = Table::new(gpus);
    table.with(Disable::row(Rows::first()));
    println!("{}", table);
}
