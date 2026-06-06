use std::{fs, path::Path};

#[derive(Clone)]
pub(crate) struct ProcessInfo {
    pub(crate) pid: u32,
    pub(crate) name: String,
    pub(crate) state: String,
    pub(crate) state_detail: String,
    pub(crate) memory: String,
    pub(crate) virtual_memory: String,
    rss_kib: u64,
}

pub(crate) fn collect_processes(limit: usize) -> Vec<ProcessInfo> {
    let Ok(entries) = fs::read_dir("/proc") else {
        return Vec::new();
    };

    let mut processes = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let pid = entry.file_name().to_string_lossy().parse::<u32>().ok()?;
            read_process_status(pid, &entry.path())
        })
        .collect::<Vec<_>>();

    processes.sort_by(|left, right| right.rss_kib.cmp(&left.rss_kib));
    processes.truncate(limit);
    processes
}

fn read_process_status(pid: u32, proc_dir: &Path) -> Option<ProcessInfo> {
    let contents = fs::read_to_string(proc_dir.join("status")).ok()?;
    let mut process = parse_status(pid, &contents);
    if let Some(command) = read_command_line(proc_dir) {
        process.name = command;
    }
    Some(process)
}

fn parse_status(pid: u32, contents: &str) -> ProcessInfo {
    let name = status_value(contents, "Name")
        .unwrap_or("unknown")
        .to_owned();
    let state_detail = status_value(contents, "State")
        .unwrap_or("unknown")
        .to_owned();
    let state = translate_process_state(&state_detail);
    let rss_kib = status_kib(contents, "VmRSS").unwrap_or(0);
    let vm_size_kib = status_kib(contents, "VmSize").unwrap_or(0);

    ProcessInfo {
        pid,
        name,
        state,
        state_detail,
        memory: human_kib(rss_kib),
        virtual_memory: human_kib(vm_size_kib),
        rss_kib,
    }
}

fn translate_process_state(state: &str) -> String {
    match state.chars().next() {
        Some('R') => "运行中",
        Some('S') => "休眠中",
        Some('D') => "不可中断等待",
        Some('T') | Some('t') => "已停止",
        Some('Z') => "僵尸进程",
        Some('I') => "空闲内核线程",
        _ => "未知",
    }
    .to_owned()
}

fn status_value<'a>(contents: &'a str, key: &str) -> Option<&'a str> {
    contents
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{key}:")))
        .map(str::trim)
}

fn status_kib(contents: &str, key: &str) -> Option<u64> {
    status_value(contents, key)?
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

fn read_command_line(proc_dir: &Path) -> Option<String> {
    let bytes = fs::read(proc_dir.join("cmdline")).ok()?;
    let command = bytes
        .split(|byte| *byte == 0)
        .filter_map(|part| std::str::from_utf8(part).ok())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    if command.is_empty() {
        None
    } else {
        Some(command)
    }
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
    fn parses_linux_status() {
        let process = parse_status(
            42,
            "Name:\tnginx\nState:\tS (sleeping)\nVmSize:\t204800 kB\nVmRSS:\t65536 kB\n",
        );

        assert_eq!(process.pid, 42);
        assert_eq!(process.name, "nginx");
        assert_eq!(process.state, "休眠中");
        assert_eq!(process.state_detail, "S (sleeping)");
        assert_eq!(process.memory, "64 MiB");
        assert_eq!(process.virtual_memory, "200 MiB");
    }
}
