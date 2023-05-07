use nvml_wrapper::{
    bitmasks::device::ThrottleReasons, enum_wrappers::device::TemperatureSensor, Device,
};
use tabled::Tabled;

#[derive(Tabled)]
#[tabled(rename_all = "UPPERCASE")]
pub struct GPUStats {
    pub idx: u32,

    #[tabled(skip)]
    pub throttling: ThrottleReasons,

    #[tabled(skip)]
    pub process_pids: Vec<u32>,

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

    processes: String, // same as above but in a diasplayable format
}

impl GPUStats {
    pub fn from_nvml_device(device: Device) -> Self {
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
        let process_pids = compute_processes
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

            process_pids,
            processes,
        }
    }
}
