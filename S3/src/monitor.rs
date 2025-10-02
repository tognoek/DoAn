use sysinfo::{System, Disks, CpuRefreshKind, RefreshKind};
use std::time::Instant;

pub struct Monitor {
    sys: System,
    disks: Disks,
    start_mem: u64,
    start_cpu: f32,
    start_time: Instant,
    start_disk: u64,
}

impl Monitor {
    pub fn start() -> Self {
        let mut sys = System::new_with_specifics(
            RefreshKind::new().with_cpu(CpuRefreshKind::everything()),
        );
        sys.refresh_all();

        let mut disks = Disks::new_with_refreshed_list();
        disks.refresh();

        let used_mem = sys.used_memory();

        sys.refresh_cpu_usage();
        let cpu = avg_cpu(&sys);

        let disk_used: u64 = disks.iter().map(|d| d.total_space() - d.available_space()).sum();

        println!(
            "[Begin] RAM: {} MB | CPU: {:.2}% | DiskUsed: {} MB",
            used_mem / 1024 / 1024,
            cpu,
            disk_used / 1024 / 1024
        );

        Self {
            sys,
            disks,
            start_mem: used_mem,
            start_cpu: cpu,
            start_time: Instant::now(),
            start_disk: disk_used,
        }
    }

    pub fn end(mut self) {
        self.sys.refresh_memory();
        self.sys.refresh_cpu_usage();
        self.disks.refresh();

        let used_mem = self.sys.used_memory();
        let cpu = avg_cpu(&self.sys);
        let disk_used: u64 = self.disks.iter().map(|d| d.total_space() - d.available_space()).sum();

        println!(
            "[End] RAM: {} MB | CPU: {:.2}% | DiskUsed: {} MB",
            used_mem / 1024 / 1024,
            cpu,
            disk_used / 1024 / 1024
        );

        println!(
            "[TOGNOEK] ΔRAM: {} MB | ΔCPU: {:.2}% | ΔDisk: {} MB | Time: {:?}",
            (used_mem as i64 - self.start_mem as i64) / 1024 / 1024,
            cpu - self.start_cpu,
            (disk_used as i64 - self.start_disk as i64) / 1024 / 1024,
            self.start_time.elapsed()
        );
    }
}

fn avg_cpu(sys: &System) -> f32 {
    let cpus = sys.cpus();
    if cpus.is_empty() {
        0.0
    } else {
        cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32
    }
}
