use tokio::process::Command;

use crate::i18n::Language;

#[derive(Clone)]
pub(crate) struct UpdateCheckResult {
    pub(crate) status: String,
    pub(crate) status_class: String,
    pub(crate) output: String,
    pub(crate) update_command: String,
}

impl UpdateCheckResult {
    pub(crate) fn empty() -> Self {
        Self {
            status: String::new(),
            status_class: "idle".to_owned(),
            output: String::new(),
            update_command: "sudo rustpanel update".to_owned(),
        }
    }
}

pub(crate) async fn run_update_check(language: Language) -> UpdateCheckResult {
    let output = Command::new("rustpanel").arg("update-check").output().await;

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let display_output = display_update_check_output(&stdout);
            if stdout.contains("Status: update available") {
                UpdateCheckResult {
                    status: language.update_available_text().to_owned(),
                    status_class: "warn".to_owned(),
                    output: display_output,
                    update_command: "sudo rustpanel update".to_owned(),
                }
            } else {
                UpdateCheckResult {
                    status: language.up_to_date_text().to_owned(),
                    status_class: "ok".to_owned(),
                    output: display_output,
                    update_command: "sudo rustpanel update".to_owned(),
                }
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            UpdateCheckResult {
                status: language.check_failed_text().to_owned(),
                status_class: "error".to_owned(),
                output: stderr.trim().to_owned(),
                update_command: "sudo rustpanel update".to_owned(),
            }
        }
        Err(error) => UpdateCheckResult {
            status: language.check_failed_text().to_owned(),
            status_class: "error".to_owned(),
            output: error.to_string(),
            update_command: "sudo rustpanel update".to_owned(),
        },
    }
}

fn display_update_check_output(output: &str) -> String {
    output
        .lines()
        .filter(|line| !line.starts_with("Update:"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hides_ssh_update_command_from_panel_output() {
        let output = "Version: 0.1.4\nStatus: update available\nUpdate: sudo rustpanel update\n";

        assert_eq!(
            display_update_check_output(output),
            "Version: 0.1.4\nStatus: update available"
        );
    }
}
