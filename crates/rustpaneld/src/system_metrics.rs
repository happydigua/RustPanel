use std::{collections::HashMap, process::Command};

#[derive(Clone)]
pub(crate) struct SystemMetrics {
    pub(crate) load_summary: String,
    pub(crate) load_detail: String,
    pub(crate) memory: String,
    pub(crate) disk: String,
    pub(crate) uptime: String,
}

pub(crate) fn collect_system_metrics() -> SystemMetrics {
    let load = load_average().unwrap_or_else(LoadAverage::unknown);
    SystemMetrics {
        load_summary: load.summary(),
        load_detail: load.detail(),
        memory: memory_usage().unwrap_or_else(|| "unknown".to_owned()),
        disk: disk_usage().unwrap_or_else(|| "unknown".to_owned()),
        uptime: uptime().unwrap_or_else(|| "unknown".to_owned()),
    }
}

#[derive(Clone, Copy)]
struct LoadAverage {
    one: f64,
    five: f64,
    fifteen: f64,
    cpu_cores: usize,
}

impl LoadAverage {
    fn unknown() -> Self {
        Self {
            one: 0.0,
            five: 0.0,
            fifteen: 0.0,
            cpu_cores: 0,
        }
    }

    fn summary(self) -> String {
        if self.cpu_cores == 0 {
            return "unknown".to_owned();
        }

        let pressure = self.one / self.cpu_cores as f64;
        let label = if pressure < 0.7 {
            "正常"
        } else if pressure < 1.0 {
            "偏高"
        } else {
            "过载"
        };

        format!("{label} ({:.0}% / core)", pressure * 100.0)
    }

    fn detail(self) -> String {
        if self.cpu_cores == 0 {
            return "unknown".to_owned();
        }

        format!(
            "1m {:.2}, 5m {:.2}, 15m {:.2}, {} cores",
            self.one, self.five, self.fifteen, self.cpu_cores
        )
    }
}

fn load_average() -> Option<LoadAverage> {
    let contents = std::fs::read_to_string("/proc/loadavg").ok()?;
    let mut parts = contents.split_whitespace();
    Some(LoadAverage {
        one: parts.next()?.parse().ok()?,
        five: parts.next()?.parse().ok()?,
        fifteen: parts.next()?.parse().ok()?,
        cpu_cores: std::thread::available_parallelism().ok()?.get(),
    })
}

fn memory_usage() -> Option<String> {
    let contents = std::fs::read_to_string("/proc/meminfo").ok()?;
    let values = parse_meminfo(&contents);
    let total = *values.get("MemTotal")?;
    let available = *values.get("MemAvailable")?;
    let used = total.saturating_sub(available);
    Some(format!("{} / {}", human_kib(used), human_kib(total)))
}

fn parse_meminfo(contents: &str) -> HashMap<String, u64> {
    contents
        .lines()
        .filter_map(|line| {
            let (key, rest) = line.split_once(':')?;
            let value = rest.split_whitespace().next()?.parse::<u64>().ok()?;
            Some((key.to_owned(), value))
        })
        .collect()
}

fn disk_usage() -> Option<String> {
    let output = Command::new("df")
        .args(["-h", "--output=used,size,pcent", "/"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().nth(1)?;
    let mut parts = line.split_whitespace();
    Some(format!(
        "{} / {} ({})",
        parts.next()?,
        parts.next()?,
        parts.next()?
    ))
}

fn uptime() -> Option<String> {
    let contents = std::fs::read_to_string("/proc/uptime").ok()?;
    let seconds = contents.split_whitespace().next()?.parse::<f64>().ok()? as u64;
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    Some(format!("{days}d {hours}h {minutes}m"))
}

fn human_kib(kib: u64) -> String {
    let mib = kib as f64 / 1024.0;
    if mib >= 1024.0 {
        format!("{:.1} GiB", mib / 1024.0)
    } else {
        format!("{mib:.0} MiB")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_memory() {
        assert_eq!(human_kib(1024), "1 MiB");
        assert_eq!(human_kib(1024 * 1024), "1.0 GiB");
    }

    #[test]
    fn explains_load_average() {
        let load = LoadAverage {
            one: 0.5,
            five: 0.3,
            fifteen: 0.2,
            cpu_cores: 2,
        };

        assert_eq!(load.summary(), "正常 (25% / core)");
        assert_eq!(load.detail(), "1m 0.50, 5m 0.30, 15m 0.20, 2 cores");
    }
}
