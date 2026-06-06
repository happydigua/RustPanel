use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PanelPaths {
    pub etc_dir: PathBuf,
    pub apps_dir: PathBuf,
    pub state_dir: PathBuf,
    pub acme_webroot: PathBuf,
    pub nginx_conf_dir: PathBuf,
    pub systemd_unit_dir: PathBuf,
}

impl Default for PanelPaths {
    fn default() -> Self {
        Self {
            etc_dir: PathBuf::from("/etc/rustpanel"),
            apps_dir: PathBuf::from("/etc/rustpanel/apps"),
            state_dir: PathBuf::from("/var/lib/rustpanel"),
            acme_webroot: PathBuf::from("/var/lib/rustpanel/acme"),
            nginx_conf_dir: PathBuf::from("/etc/nginx/conf.d/rustpanel"),
            systemd_unit_dir: PathBuf::from("/etc/systemd/system"),
        }
    }
}

impl PanelPaths {
    pub fn env_file_for(&self, app_id: &str) -> PathBuf {
        self.apps_dir.join(format!("{app_id}.env"))
    }

    pub fn systemd_unit_for(&self, unit_name: &str) -> PathBuf {
        self.systemd_unit_dir.join(unit_name)
    }

    pub fn nginx_config_for(&self, app_id: &str, domain: &str) -> PathBuf {
        self.nginx_conf_dir.join(format!("{app_id}-{domain}.conf"))
    }
}
