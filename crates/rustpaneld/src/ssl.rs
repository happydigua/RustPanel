use std::process::Command;

#[derive(Clone)]
pub(crate) struct CertificateInfo {
    pub(crate) name: String,
    pub(crate) domains: String,
    pub(crate) expiry: String,
}

#[derive(Clone)]
pub(crate) struct SslInfo {
    pub(crate) nginx_status: String,
    pub(crate) nginx_status_class: String,
    pub(crate) certbot_status: String,
    pub(crate) certbot_status_class: String,
    pub(crate) certificates: Vec<CertificateInfo>,
}

pub(crate) fn collect_ssl_info() -> SslInfo {
    let nginx = command_status("nginx", &["-v"]);
    let certbot = command_status("certbot", &["--version"]);
    SslInfo {
        nginx_status: nginx.label,
        nginx_status_class: nginx.class,
        certbot_status: certbot.label,
        certbot_status_class: certbot.class,
        certificates: collect_certificates(),
    }
}

struct CommandStatus {
    label: String,
    class: String,
}

fn command_status(command: &str, args: &[&str]) -> CommandStatus {
    match Command::new(command).args(args).output() {
        Ok(output) if output.status.success() => CommandStatus {
            label: "已安装".to_owned(),
            class: "ok".to_owned(),
        },
        _ => CommandStatus {
            label: "未安装".to_owned(),
            class: "idle".to_owned(),
        },
    }
}

fn collect_certificates() -> Vec<CertificateInfo> {
    let Ok(output) = Command::new("certbot").arg("certificates").output() else {
        return Vec::new();
    };

    if !output.status.success() {
        return Vec::new();
    }

    parse_certbot_certificates(&String::from_utf8_lossy(&output.stdout))
}

fn parse_certbot_certificates(contents: &str) -> Vec<CertificateInfo> {
    let mut certificates = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_domains = String::new();
    let mut current_expiry = String::new();

    for line in contents.lines().map(str::trim) {
        if let Some(name) = line.strip_prefix("Certificate Name:") {
            if let Some(name) = current_name.take() {
                certificates.push(CertificateInfo {
                    name,
                    domains: current_domains.clone(),
                    expiry: current_expiry.clone(),
                });
            }

            current_name = Some(name.trim().to_owned());
            current_domains.clear();
            current_expiry.clear();
            continue;
        }

        if let Some(domains) = line.strip_prefix("Domains:") {
            current_domains = domains.trim().to_owned();
            continue;
        }

        if let Some(expiry) = line.strip_prefix("Expiry Date:") {
            current_expiry = expiry.trim().to_owned();
        }
    }

    if let Some(name) = current_name {
        certificates.push(CertificateInfo {
            name,
            domains: current_domains,
            expiry: current_expiry,
        });
    }

    certificates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_certbot_certificates() {
        let certs = parse_certbot_certificates(
            r#"
Certificate Name: example.com
    Domains: example.com www.example.com
    Expiry Date: 2026-09-01 12:00:00+00:00 (VALID: 90 days)
"#,
        );

        assert_eq!(certs.len(), 1);
        assert_eq!(certs[0].name, "example.com");
        assert_eq!(certs[0].domains, "example.com www.example.com");
    }
}
