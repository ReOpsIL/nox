use std::time::Instant;
use sysinfo::{System, Cpu, Process};

pub struct SystemMonitor {
    system: System,
    start_time: Instant,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self {
            system,
            start_time: Instant::now(),
        }
    }
    
    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }
    
    pub fn get_uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
    
    pub fn get_cpu_usage_percent(&mut self) -> f32 {
        self.system.refresh_cpu();
        self.system.global_cpu_info().cpu_usage()
    }
    
    pub fn get_memory_usage_mb(&mut self) -> usize {
        self.system.refresh_memory();
        let used_memory = self.system.used_memory();
        (used_memory / 1024 / 1024) as usize // Convert from bytes to MB
    }
    
    pub fn get_total_memory_mb(&mut self) -> u64 {
        self.system.refresh_memory();
        let total_memory = self.system.total_memory();
        total_memory / 1024 / 1024 // Convert from bytes to MB
    }
    
    pub fn get_memory_usage_percent(&mut self) -> f32 {
        let used = self.get_memory_usage_mb() as f32;
        let total = self.get_total_memory_mb() as f32;
        
        if total > 0.0 {
            (used / total) * 100.0
        } else {
            0.0
        }
    }
    
    pub fn get_process_count(&mut self) -> usize {
        self.system.refresh_processes();
        self.system.processes().len()
    }
    
    pub fn get_system_load_average(&mut self) -> Option<f64> {
        self.system.refresh_cpu();
        // Load average is not available in all systems via sysinfo
        None
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}