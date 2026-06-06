#[derive(Clone, Copy)]
pub(crate) enum Language {
    Zh,
    En,
}

#[derive(Clone, Copy)]
pub(crate) struct TextLabels {
    pub(crate) html_lang: &'static str,
    pub(crate) local_daemon: &'static str,
    pub(crate) sites: &'static str,
    pub(crate) domains: &'static str,
    pub(crate) managed_nginx_config: &'static str,
    pub(crate) open_list: &'static str,
    pub(crate) name: &'static str,
    pub(crate) service: &'static str,
    pub(crate) state: &'static str,
    pub(crate) upstream: &'static str,
    pub(crate) deploy: &'static str,
    pub(crate) system: &'static str,
    pub(crate) system_resources: &'static str,
    pub(crate) systemd_services: &'static str,
    pub(crate) load_average: &'static str,
    pub(crate) memory: &'static str,
    pub(crate) disk: &'static str,
    pub(crate) uptime: &'static str,
    pub(crate) current_version: &'static str,
    pub(crate) update_check: &'static str,
    pub(crate) update_help: &'static str,
    pub(crate) update_command: &'static str,
    pub(crate) back_dashboard: &'static str,
    pub(crate) check_result: &'static str,
    pub(crate) language: &'static str,
    pub(crate) zh: &'static str,
    pub(crate) en: &'static str,
    pub(crate) login: &'static str,
    pub(crate) logout: &'static str,
    pub(crate) username: &'static str,
    pub(crate) password: &'static str,
    pub(crate) login_failed: &'static str,
    pub(crate) no_services: &'static str,
}

impl Language {
    pub(crate) fn from_param(param: Option<&str>) -> Self {
        match param {
            Some("en") => Self::En,
            _ => Self::Zh,
        }
    }

    pub(crate) fn code(self) -> &'static str {
        match self {
            Self::Zh => "zh",
            Self::En => "en",
        }
    }

    pub(crate) fn labels(self) -> TextLabels {
        match self {
            Self::Zh => zh_labels(),
            Self::En => en_labels(),
        }
    }

    pub(crate) fn up_to_date_text(self) -> &'static str {
        match self {
            Self::Zh => "已是最新",
            Self::En => "Up to date",
        }
    }

    pub(crate) fn update_available_text(self) -> &'static str {
        match self {
            Self::Zh => "有可用更新",
            Self::En => "Update available",
        }
    }

    pub(crate) fn check_failed_text(self) -> &'static str {
        match self {
            Self::Zh => "检测失败",
            Self::En => "Check failed",
        }
    }
}

fn zh_labels() -> TextLabels {
    TextLabels {
        html_lang: "zh-CN",
        local_daemon: "本机面板服务",
        sites: "站点",
        domains: "域名",
        managed_nginx_config: "RustPanel 管理的 Nginx 配置",
        open_list: "查看列表",
        name: "名称",
        service: "服务",
        state: "状态",
        upstream: "上游地址",
        deploy: "部署",
        system: "系统",
        system_resources: "系统资源",
        systemd_services: "Systemd 服务",
        load_average: "负载",
        memory: "内存",
        disk: "磁盘",
        uptime: "运行时间",
        current_version: "当前版本",
        update_check: "检查更新",
        update_help: "如需更新，在服务器 SSH 中执行",
        update_command: "sudo rustpanel update",
        back_dashboard: "返回首页",
        check_result: "检测结果",
        language: "语言",
        zh: "中文",
        en: "English",
        login: "登录",
        logout: "退出",
        username: "账号",
        password: "密码",
        login_failed: "账号或密码错误",
        no_services: "没有发现 systemd 服务，或当前系统不支持 systemctl。",
    }
}

fn en_labels() -> TextLabels {
    TextLabels {
        html_lang: "en",
        local_daemon: "Local panel daemon",
        sites: "Sites",
        domains: "Domains",
        managed_nginx_config: "Managed Nginx config",
        open_list: "Open list",
        name: "Name",
        service: "Service",
        state: "State",
        upstream: "Upstream",
        deploy: "Deploy",
        system: "System",
        system_resources: "System resources",
        systemd_services: "Systemd services",
        load_average: "Load",
        memory: "Memory",
        disk: "Disk",
        uptime: "Uptime",
        current_version: "Current version",
        update_check: "Check updates",
        update_help: "To update, run this over SSH",
        update_command: "sudo rustpanel update",
        back_dashboard: "Back to dashboard",
        check_result: "Check result",
        language: "Language",
        zh: "中文",
        en: "English",
        login: "Log in",
        logout: "Log out",
        username: "Username",
        password: "Password",
        login_failed: "Invalid username or password",
        no_services: "No systemd services found, or systemctl is not available.",
    }
}
