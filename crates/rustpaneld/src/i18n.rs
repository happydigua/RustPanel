#[derive(Clone, Copy)]
pub(crate) enum Language {
    Zh,
    En,
}

#[derive(Clone, Copy)]
pub(crate) struct TextLabels {
    pub(crate) html_lang: &'static str,
    pub(crate) local_daemon: &'static str,
    pub(crate) overview: &'static str,
    pub(crate) processes: &'static str,
    pub(crate) sites: &'static str,
    pub(crate) domains: &'static str,
    pub(crate) ssl: &'static str,
    pub(crate) managed_nginx_config: &'static str,
    pub(crate) open_list: &'static str,
    pub(crate) name: &'static str,
    pub(crate) pid: &'static str,
    pub(crate) process: &'static str,
    pub(crate) service: &'static str,
    pub(crate) state: &'static str,
    pub(crate) upstream: &'static str,
    pub(crate) system_resources: &'static str,
    pub(crate) systemd_services: &'static str,
    pub(crate) load_average: &'static str,
    pub(crate) memory: &'static str,
    pub(crate) memory_total: &'static str,
    pub(crate) memory_used: &'static str,
    pub(crate) memory_available: &'static str,
    pub(crate) memory_usage_percent: &'static str,
    pub(crate) disk: &'static str,
    pub(crate) uptime: &'static str,
    pub(crate) virtual_memory: &'static str,
    pub(crate) current_version: &'static str,
    pub(crate) update_check: &'static str,
    pub(crate) run_update: &'static str,
    pub(crate) update_status: &'static str,
    pub(crate) certificate_status: &'static str,
    pub(crate) update_help: &'static str,
    pub(crate) update_command: &'static str,
    pub(crate) check_result: &'static str,
    pub(crate) language: &'static str,
    pub(crate) zh: &'static str,
    pub(crate) en: &'static str,
    pub(crate) login: &'static str,
    pub(crate) logout: &'static str,
    pub(crate) username: &'static str,
    pub(crate) password: &'static str,
    pub(crate) email: &'static str,
    pub(crate) issue_certificate: &'static str,
    pub(crate) domain_required: &'static str,
    pub(crate) login_failed: &'static str,
    pub(crate) no_processes: &'static str,
    pub(crate) no_services: &'static str,
    pub(crate) service_filter_running: &'static str,
    pub(crate) service_filter_failed: &'static str,
    pub(crate) service_filter_stopped: &'static str,
    pub(crate) service_filter_all: &'static str,
    pub(crate) no_sites: &'static str,
    pub(crate) no_certificates: &'static str,
    pub(crate) config_file: &'static str,
    pub(crate) certificate: &'static str,
    pub(crate) certificates: &'static str,
    pub(crate) expiry: &'static str,
    pub(crate) nginx: &'static str,
    pub(crate) certbot: &'static str,
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

    pub(crate) fn state_text(self, value: &str) -> String {
        match (self, value) {
            (Self::Zh, "loaded") => "已加载",
            (Self::Zh, "not-found") => "未找到",
            (Self::Zh, "active") => "运行中",
            (Self::Zh, "inactive") => "未运行",
            (Self::Zh, "failed") => "失败",
            (Self::Zh, "running") => "运行中",
            (Self::Zh, "exited") => "已退出",
            (Self::Zh, "dead") => "已停止",
            _ => value,
        }
        .to_owned()
    }
}

fn zh_labels() -> TextLabels {
    TextLabels {
        html_lang: "zh-CN",
        local_daemon: "本机面板服务",
        overview: "概览",
        processes: "进程",
        sites: "站点",
        domains: "域名",
        ssl: "SSL",
        managed_nginx_config: "RustPanel 管理的 Nginx 配置",
        open_list: "查看列表",
        name: "名称",
        pid: "PID",
        process: "进程",
        service: "服务",
        state: "状态",
        upstream: "上游地址",
        system_resources: "系统资源",
        systemd_services: "系统服务",
        load_average: "CPU 负载",
        memory: "内存",
        memory_total: "总内存",
        memory_used: "已使用内存",
        memory_available: "可用内存",
        memory_usage_percent: "内存使用率",
        disk: "磁盘",
        uptime: "运行时间",
        virtual_memory: "虚拟内存",
        current_version: "当前版本",
        update_check: "检查更新",
        run_update: "执行更新",
        update_status: "更新状态",
        certificate_status: "证书任务状态",
        update_help: "也可在服务器 SSH 中执行",
        update_command: "sudo rustpanel update",
        check_result: "检测结果",
        language: "语言",
        zh: "中文",
        en: "English",
        login: "登录",
        logout: "退出",
        username: "账号",
        password: "密码",
        email: "邮箱",
        issue_certificate: "申请证书",
        domain_required: "请输入已解析到本服务器的域名",
        login_failed: "账号或密码错误",
        no_processes: "没有读取到进程信息。",
        no_services: "没有发现 systemd 服务，或当前系统不支持 systemctl。",
        service_filter_running: "运行中",
        service_filter_failed: "异常",
        service_filter_stopped: "已停止",
        service_filter_all: "全部",
        no_sites: "还没有发现 Nginx 站点配置。",
        no_certificates: "还没有发现 certbot 证书。",
        config_file: "配置文件",
        certificate: "证书",
        certificates: "证书",
        expiry: "过期时间",
        nginx: "Nginx",
        certbot: "Certbot",
    }
}

fn en_labels() -> TextLabels {
    TextLabels {
        html_lang: "en",
        local_daemon: "Local panel daemon",
        overview: "Overview",
        processes: "Processes",
        sites: "Sites",
        domains: "Domains",
        ssl: "SSL",
        managed_nginx_config: "Managed Nginx config",
        open_list: "Open list",
        name: "Name",
        pid: "PID",
        process: "Process",
        service: "Service",
        state: "State",
        upstream: "Upstream",
        system_resources: "System resources",
        systemd_services: "Systemd services",
        load_average: "Load",
        memory: "Memory",
        memory_total: "Total memory",
        memory_used: "Used memory",
        memory_available: "Available memory",
        memory_usage_percent: "Memory usage",
        disk: "Disk",
        uptime: "Uptime",
        virtual_memory: "Virtual memory",
        current_version: "Current version",
        update_check: "Check updates",
        run_update: "Run update",
        update_status: "Update status",
        certificate_status: "Certificate job status",
        update_help: "Fallback SSH command",
        update_command: "sudo rustpanel update",
        check_result: "Check result",
        language: "Language",
        zh: "中文",
        en: "English",
        login: "Log in",
        logout: "Log out",
        username: "Username",
        password: "Password",
        email: "Email",
        issue_certificate: "Issue certificate",
        domain_required: "Enter a domain that already points to this server",
        login_failed: "Invalid username or password",
        no_processes: "No process information was found.",
        no_services: "No systemd services found, or systemctl is not available.",
        service_filter_running: "Running",
        service_filter_failed: "Failed",
        service_filter_stopped: "Stopped",
        service_filter_all: "All",
        no_sites: "No Nginx site config was found.",
        no_certificates: "No certbot certificates were found.",
        config_file: "Config file",
        certificate: "Certificate",
        certificates: "Certificates",
        expiry: "Expiry",
        nginx: "Nginx",
        certbot: "Certbot",
    }
}
