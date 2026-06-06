use std::process::Command;

#[derive(Clone)]
pub(crate) struct ServiceUnit {
    pub(crate) name: String,
    pub(crate) load_state: String,
    pub(crate) active_state: String,
    pub(crate) sub_state: String,
    pub(crate) description: String,
}

pub(crate) fn collect_service_units(limit: usize) -> Vec<ServiceUnit> {
    let output = Command::new("systemctl")
        .args([
            "list-units",
            "--type=service",
            "--all",
            "--no-legend",
            "--no-pager",
        ])
        .output();

    let Ok(output) = output else {
        return Vec::new();
    };

    if !output.status.success() {
        return Vec::new();
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(parse_service_line)
        .take(limit)
        .collect()
}

fn parse_service_line(line: &str) -> Option<ServiceUnit> {
    let mut parts = line.split_whitespace();
    let first = parts.next()?;
    let name = if first == "●" {
        parts.next()?.to_owned()
    } else {
        first.trim_start_matches('●').to_owned()
    };
    let load_state = parts.next()?.to_owned();
    let active_state = parts.next()?.to_owned();
    let sub_state = parts.next()?.to_owned();
    let description = parts.collect::<Vec<_>>().join(" ");

    Some(ServiceUnit {
        name,
        load_state,
        active_state,
        sub_state,
        description,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_systemctl_line() {
        let unit = parse_service_line("nginx.service loaded active running A web server").unwrap();

        assert_eq!(unit.name, "nginx.service");
        assert_eq!(unit.load_state, "loaded");
        assert_eq!(unit.active_state, "active");
        assert_eq!(unit.sub_state, "running");
        assert_eq!(unit.description, "A web server");
    }

    #[test]
    fn parses_failed_line_with_marker() {
        let unit = parse_service_line("● demo.service loaded failed failed Demo service").unwrap();

        assert_eq!(unit.name, "demo.service");
        assert_eq!(unit.active_state, "failed");
    }
}
