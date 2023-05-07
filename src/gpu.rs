use nvml_wrapper::{
    bitmasks::device::ThrottleReasons, enum_wrappers::device::TemperatureSensor, Device, Nvml,
};
use tabled::Tabled;

#[derive(Tabled)]
#[tabled(rename_all = "PascalCase")]
pub struct GPUStats {
    pub idx: u32,
    #[tabled(skip)]
    pub throttling: ThrottleReasons,
    name: String,
    #[tabled(display_with("Self::display_temp", self))]
    temp: u32,
    #[tabled(display_with("Self::display_power", self))]
    power: (u32, u32), // (usage, limit)
    #[tabled(display_with("Self::display_utilizations", self))]
    utilizations: (u32, u32), // (gpu, memory)
    #[tabled(display_with("Self::display_memory", self))]
    memory: (u64, u64), // (used, total) in bytes

    // these are not displayed unless verbose is true
    #[tabled(display_with("Self::display_capability", self))]
    capability: (i32, i32), // (major, minor)
    cores: u32,
    fan: String,
    display: String,
    #[tabled(display_with("Self::display_processes", self))]
    pub processes: Vec<u32>,
}

impl GPUStats {
    pub fn from_nvml_device(device: Device) -> Self {
        let idx = device.index().unwrap();
        let name = device.name().unwrap();

        let temp = device.temperature(TemperatureSensor::Gpu).unwrap();

        let power_usage = device.power_usage().unwrap();
        let power_limit = device.enforced_power_limit().unwrap();
        let power = (power_usage, power_limit);

        let gpu_utilization = device.utilization_rates().unwrap().gpu;
        let memory_utilization = device.utilization_rates().unwrap().memory;
        let utilizations = (gpu_utilization, memory_utilization);

        let memory_used = device.memory_info().unwrap().used;
        let memory_total = device.memory_info().unwrap().total;
        let memory = (memory_used, memory_total);

        let compute_cap = device.cuda_compute_capability().unwrap();
        let capability = (compute_cap.major, compute_cap.minor);
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
        let processes = compute_processes
            .iter()
            .map(|process| process.pid)
            .collect::<Vec<u32>>();

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

            processes,
        }
    }

    fn display_processes(&self) -> String {
        let processes = self.processes.clone();
        processes
            .iter()
            .map(|pid| pid.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn display_temp(&self) -> String {
        format!("{}°C", self.temp)
    }

    fn display_power(&self) -> String {
        let (power_usage, power_limit) = self.power;
        format!(
            "{:.0}W/{:.0}W",
            power_usage as f32 / 1000.0,
            power_limit as f32 / 1000.0
        )
    }
    fn display_utilizations(&self) -> String {
        let (gpu_utilization, memory_utilization) = self.utilizations;
        format!("GPU {}% VRAM {}%", gpu_utilization, memory_utilization)
    }

    fn display_memory(&self) -> String {
        let (memory_used, memory_total) = self.memory;
        format!(
            "{:.2}GB/{:.2}GB",
            memory_used as f32 / 1024.0 / 1024.0 / 1024.0,
            memory_total as f32 / 1024.0 / 1024.0 / 1024.0
        )
    }

    fn display_capability(&self) -> String {
        let (major, minor) = self.capability;
        format!("{}.{}", major, minor)
    }
}

pub fn get_driver_stats(nvml: &Nvml) -> (String, String) {
    // NB: cuda version begins as an int e.g. 12000
    // this is converted to a float e.g. 12.0
    let cuda_version = nvml.sys_cuda_driver_version().unwrap();
    let cuda_version = cuda_version as f32 / 1000.0;
    let cuda_version = format!("{:.1}", cuda_version);
    let driver_version = nvml.sys_driver_version().unwrap();

    (cuda_version, driver_version)
}
