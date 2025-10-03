use std::env;
#[derive(Default)]
pub struct Stats {
    pub max_call: u64,
    pub total_row: u64,
    pub total_calls: u64,
    pub total_cpu: f32,
    pub total_ram: u64,
    pub total_disk: u64,
    pub total_time: u64,
    pub max_cpu: f32,
    pub max_ram: u64,
    pub max_disk: u64,
    pub max_time: u64,
}

pub struct StatEvent {
    pub cpu: f32,
    pub ram: u64,
    pub disk: u64,
    pub time: u64,
}

impl Stats {
    pub fn update(&mut self, cpu: f32, ram: u64, disk: u64, time: u64) {
        if self.max_call < 1 {
            self.max_call = env::var("MAXCALL")
                            .unwrap_or_else(|_| "100000000000000".to_string()) // fallback String
                            .parse::<u64>()
                            .unwrap_or(100_000_000_000_000);
        }
        self.total_calls += 1;
        if self.total_calls > self.max_call {
            self.total_row += 1;
            self.total_calls -= self.max_call;
        }
        self.total_cpu += cpu;
        self.total_ram += ram;
        self.total_disk += disk;
        self.total_time += time;

        if cpu > self.max_cpu { self.max_cpu = cpu; }
        if ram > self.max_ram { self.max_ram = ram; }
        if disk > self.max_disk { self.max_disk = disk; }        
        if ram > self.max_ram { self.max_ram = ram; }
        if time > self.max_time { self.max_time = time; }

    }

    pub fn draw_table(&self) {
        if self.total_calls == 0 {
            return;
        }
        let avg_cpu = self.total_cpu / self.total_calls as f32;
        let avg_ram = self.total_ram / self.total_calls;
        let avg_disk = self.total_disk / self.total_calls;
        let avg_time = self.total_time / self.total_calls;

        print!("\x1B[2J\x1B[3J\x1B[H");
        println!("===========================[http://127.0.0.1:3000]============================");
        println!("┌─────────────┬───────────────┬───────────────┬───────────────┬───────────────┐");
        println!("│ TotalCalls  │   CPU %       │   RAM (MB)    │  Disk (MB)    │  Time (ms)    │");
        println!("├─────────────┼───────────────┼───────────────┼───────────────┼───────────────┤");
        println!(
            "│ {:11} │ Avg: {:8.2} │ Avg: {:8} │ Avg: {:8} │ Avg: {:8} │",
            self.format_number(self.total_calls), avg_cpu, avg_ram , avg_disk , avg_time
        );
        println!(
            "│ {:11} │ Max: {:8.2} │ Max: {:8} │ Max: {:8} │ Max: {:8} │",
            "", self.max_cpu, self.max_ram , self.max_disk , self.max_time
        );
        println!("└─────────────┴───────────────┴───────────────┴───────────────┴───────────────┘");
    }

    pub fn format_number(&self, n: u64) -> String {
        let suffixes = ["", "K", "M", "B", "T", "P", "E"]; 

        let mut i = 0;
        let mut value = n as f64;

        while value >= 1000.0 && i < suffixes.len() - 1 {
            value /= 1000.0;
            i += 1;
        }
        if self.total_row > 0 {
            if value < 10.0 && i > 0 {
                format!("{}-{:.2}{}",self.total_row, value, suffixes[i]) // 1.23K
            } else if value < 100.0 && i > 0 {
                format!("{}-{:.1}{}",self.total_row,  value, suffixes[i]) // 12.3K
            } else {
                format!("{}-{:.0}{}",self.total_row,  value, suffixes[i]) // 123K, 1M...
            }
            
        } else {
            if value < 10.0 && i > 0 {
                format!("{:.2}{}", value, suffixes[i]) // 1.23K
            } else if value < 100.0 && i > 0 {
                format!("{:.1}{}", value, suffixes[i]) // 12.3K
            } else {
                format!("{:.0}{}", value, suffixes[i]) // 123K, 1M...
            }
        }
    }
}
