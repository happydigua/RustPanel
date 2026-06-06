use askama::Template;

use crate::{
    i18n::TextLabels, privileged::JobStatus, processes::ProcessInfo, sites::SiteInfo, ssl::SslInfo,
    system_metrics::SystemMetrics, updates::UpdateCheckResult,
};

#[derive(Clone)]
pub(crate) struct ServiceRow {
    pub(crate) name: String,
    pub(crate) load_state: String,
    pub(crate) state_label: String,
    pub(crate) state_class: String,
    pub(crate) description: String,
}

#[derive(Template)]
#[template(
    source = r#"
<!doctype html>
<html lang="{{ labels.html_lang }}">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>RustPanel - {{ page_title }}</title>
    <style>
        :root {
            color-scheme: light;
            --bg: #f6f7f9;
            --panel: #fff;
            --line: #d9e0ea;
            --text: #1f2937;
            --muted: #64748b;
            --accent: #0f766e;
            --accent-soft: #e7f5f2;
            --ok: #047857;
            --warn: #b45309;
            --bad: #b91c1c;
        }
        * { box-sizing: border-box; }
        body {
            margin: 0;
            background: var(--bg);
            color: var(--text);
            font: 14px/1.5 ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
        }
        a { color: var(--accent); text-decoration: none; }
        button, input {
            font: inherit;
        }
        button {
            display: inline-flex;
            align-items: center;
            justify-content: center;
            min-height: 36px;
            border: 0;
            border-radius: 6px;
            padding: 0 14px;
            background: var(--accent);
            color: #fff;
            font-weight: 700;
            cursor: pointer;
        }
        input {
            width: 100%;
            min-height: 38px;
            border: 1px solid var(--line);
            border-radius: 6px;
            padding: 7px 10px;
            background: #fff;
            color: var(--text);
        }
        label {
            display: grid;
            gap: 6px;
            color: var(--muted);
            font-size: 13px;
        }
        .shell { min-height: 100vh; display: grid; grid-template-columns: 224px minmax(0, 1fr); }
        .sidebar { background: #ffffff; border-right: 1px solid var(--line); padding: 18px 14px; }
        .brand { font-size: 18px; font-weight: 700; margin: 0 8px 18px; }
        .nav { display: grid; gap: 4px; }
        .nav a { color: var(--text); border-radius: 6px; padding: 9px 10px; }
        .nav a.active { background: var(--accent-soft); color: #0b5c56; font-weight: 700; }
        .content { min-width: 0; }
        .topbar {
            min-height: 58px;
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 16px;
            padding: 0 24px;
            border-bottom: 1px solid var(--line);
            background: var(--panel);
        }
        .topbar h1 { margin: 0; font-size: 18px; line-height: 1.3; }
        .top-actions { display: flex; align-items: center; gap: 12px; color: var(--muted); font-size: 13px; }
        main { width: min(1160px, calc(100vw - 272px)); margin: 24px auto; }
        .metrics { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 12px; margin-bottom: 20px; }
        .metric, .panel { background: var(--panel); border: 1px solid var(--line); border-radius: 8px; }
        .metric { padding: 16px; }
        .metric span { display: block; color: var(--muted); font-size: 13px; }
        .metric strong { display: block; margin-top: 6px; font-size: 24px; line-height: 1.2; }
        .stack { display: grid; gap: 20px; }
        .grid-2 { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 20px; }
        .panel-header { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 15px 18px; border-bottom: 1px solid var(--line); }
        .panel-body { padding: 16px 18px; }
        .form-grid { display: grid; grid-template-columns: minmax(0, 1fr) minmax(220px, 0.6fr) auto; gap: 12px; align-items: end; }
        .flash { margin-bottom: 16px; border-radius: 8px; padding: 11px 14px; border: 1px solid var(--line); background: #fff; }
        .flash.ok { border-color: #bbf7d0; background: #f0fdf4; color: var(--ok); }
        .flash.error { border-color: #fecaca; background: #fef2f2; color: var(--bad); }
        h2 { margin: 0; font-size: 16px; line-height: 1.3; }
        table { width: 100%; border-collapse: collapse; }
        th, td { padding: 11px 18px; border-bottom: 1px solid var(--line); text-align: left; vertical-align: middle; overflow-wrap: anywhere; }
        th { color: var(--muted); font-size: 12px; font-weight: 600; text-transform: uppercase; }
        tr:last-child td { border-bottom: 0; }
        code { padding: 2px 5px; border-radius: 4px; background: #eef2f6; color: #243244; font: 12px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }
        pre { overflow: auto; margin: 0; padding: 12px; border-radius: 6px; background: #eef2f6; color: #243244; white-space: pre-wrap; font: 12px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }
        .muted { color: var(--muted); }
        .status { display: inline-flex; border-radius: 6px; padding: 3px 8px; font-weight: 700; }
        .status.ok { color: var(--ok); background: #ecfdf5; }
        .status.warn { color: var(--warn); background: #fffbeb; }
        .status.error, .status.failed { color: var(--bad); background: #fef2f2; }
        .status.idle { color: var(--muted); background: #f1f5f9; }
        .segments { display: inline-flex; flex-wrap: wrap; gap: 4px; padding: 3px; border: 1px solid var(--line); border-radius: 8px; background: #f8fafc; }
        .segments a { border-radius: 6px; padding: 6px 10px; color: var(--muted); }
        .segments a.active { background: var(--panel); color: var(--accent); font-weight: 700; box-shadow: 0 1px 2px rgba(15, 23, 42, 0.08); }
        .kv { margin: 0; display: grid; gap: 6px; }
        .kv dt { color: var(--muted); font-size: 13px; }
        .kv dd { margin: 0; }
        @media (max-width: 860px) {
            .shell { display: block; }
            .sidebar { border-right: 0; border-bottom: 1px solid var(--line); }
            .nav { grid-template-columns: repeat(3, minmax(0, 1fr)); }
            .topbar, .top-actions { align-items: flex-start; flex-direction: column; }
            .topbar { padding: 14px 16px; }
            main { width: calc(100vw - 24px); margin: 18px auto; }
            .metrics, .grid-2, .form-grid { grid-template-columns: 1fr; }
            table, thead, tbody, tr, th, td { display: block; }
            thead { display: none; }
            tr { padding: 12px 0; border-bottom: 1px solid var(--line); }
            td { border: 0; padding: 4px 16px; }
        }
    </style>
</head>
<body>
    <div class="shell">
        <aside class="sidebar">
            <div class="brand">RustPanel</div>
            <nav class="nav">
                <a class="{{ overview_class }}" href="{{ overview_path }}">{{ labels.overview }}</a>
                <a class="{{ processes_class }}" href="{{ processes_path }}">{{ labels.processes }}</a>
                <a class="{{ services_class }}" href="{{ services_path }}">{{ labels.systemd_services }}</a>
                <a class="{{ sites_class }}" href="{{ sites_path }}">{{ labels.sites }}</a>
                <a class="{{ ssl_class }}" href="{{ ssl_path }}">{{ labels.ssl }}</a>
                <a class="{{ update_class }}" href="{{ update_path }}">{{ labels.update_check }}</a>
            </nav>
        </aside>
        <div class="content">
            <header class="topbar">
                <h1>{{ page_title }}</h1>
                <div class="top-actions">
                    <span>{{ labels.local_daemon }}</span>
                    <span>{{ labels.language }}: <a href="{{ lang_zh_path }}">{{ labels.zh }}</a> / <a href="{{ lang_en_path }}">{{ labels.en }}</a></span>
                    <a href="{{ logout_path }}">{{ labels.logout }}</a>
                </div>
            </header>
            <main>
                {% if has_flash %}
                <div class="flash {{ flash_class }}">{{ flash_message }}</div>
                {% endif %}

                {% if active_page == "overview" %}
                <section class="metrics" aria-label="Overview">
                    <div class="metric"><span>{{ labels.sites }}</span><strong>{{ site_count }}</strong></div>
                    <div class="metric"><span>{{ labels.domains }}</span><strong>{{ domain_count }}</strong></div>
                    <div class="metric"><span>{{ labels.systemd_services }}</span><strong>{{ service_count }}</strong></div>
                    <div class="metric"><span>{{ labels.processes }}</span><strong>{{ process_count }}</strong></div>
                </section>
                <div class="grid-2">
                    <section class="panel">
                        <div class="panel-header"><h2>{{ labels.system_resources }}</h2></div>
                        <div class="panel-body">
                            <div class="grid-2">
                                <dl class="kv"><dt>{{ labels.load_average }}</dt><dd><code>{{ metrics.load_summary }}</code><br><span class="muted">{{ metrics.load_detail }}</span></dd></dl>
                                <dl class="kv"><dt>{{ labels.memory }}</dt><dd><code>{{ metrics.memory }}</code></dd></dl>
                                <dl class="kv"><dt>{{ labels.disk }}</dt><dd><code>{{ metrics.disk }}</code></dd></dl>
                                <dl class="kv"><dt>{{ labels.uptime }}</dt><dd><code>{{ metrics.uptime }}</code></dd></dl>
                            </div>
                        </div>
                    </section>
                    <section class="panel">
                        <div class="panel-header"><h2>{{ labels.update_check }}</h2></div>
                        <div class="panel-body">
                            <dl class="kv"><dt>{{ labels.current_version }}</dt><dd><code>{{ version }}</code></dd></dl>
                            <p class="muted">{{ labels.update_help }}: <code>{{ labels.update_command }}</code></p>
                        </div>
                    </section>
                </div>
                <section class="panel" style="margin-top:20px">
                    <div class="panel-header"><h2>{{ labels.processes }}</h2><a href="{{ processes_path }}">{{ labels.open_list }}</a></div>
                    {% if has_processes %}
                    <table>
                        <thead><tr><th>{{ labels.pid }}</th><th>{{ labels.process }}</th><th>{{ labels.memory }}</th><th>{{ labels.virtual_memory }}</th><th>{{ labels.state }}</th></tr></thead>
                        <tbody>
                            {% for process in processes %}
                            <tr><td><code>{{ process.pid }}</code></td><td>{{ process.name }}</td><td>{{ process.memory }}</td><td>{{ process.virtual_memory }}</td><td>{{ process.state }}<br><span class="muted">{{ process.state_detail }}</span></td></tr>
                            {% endfor %}
                        </tbody>
                    </table>
                    {% else %}
                    <div class="panel-body muted">{{ labels.no_processes }}</div>
                    {% endif %}
                </section>
                {% endif %}

                {% if active_page == "processes" %}
                <section class="metrics" aria-label="Memory">
                    <div class="metric"><span>{{ labels.memory_total }}</span><strong>{{ metrics.memory_total }}</strong></div>
                    <div class="metric"><span>{{ labels.memory_used }}</span><strong>{{ metrics.memory_used }}</strong></div>
                    <div class="metric"><span>{{ labels.memory_available }}</span><strong>{{ metrics.memory_available }}</strong></div>
                    <div class="metric"><span>{{ labels.memory_usage_percent }}</span><strong>{{ metrics.memory_usage_percent }}</strong></div>
                </section>
                <section class="panel">
                    <div class="panel-header"><h2>{{ labels.processes }}</h2></div>
                    {% if has_processes %}
                    <table>
                        <thead><tr><th>{{ labels.pid }}</th><th>{{ labels.process }}</th><th>{{ labels.memory }}</th><th>{{ labels.virtual_memory }}</th><th>{{ labels.state }}</th></tr></thead>
                        <tbody>
                            {% for process in processes %}
                            <tr><td><code>{{ process.pid }}</code></td><td>{{ process.name }}</td><td>{{ process.memory }}</td><td>{{ process.virtual_memory }}</td><td>{{ process.state }}<br><span class="muted">{{ process.state_detail }}</span></td></tr>
                            {% endfor %}
                        </tbody>
                    </table>
                    {% else %}
                    <div class="panel-body muted">{{ labels.no_processes }}</div>
                    {% endif %}
                </section>
                {% endif %}

                {% if active_page == "services" %}
                <section class="panel">
                    <div class="panel-header">
                        <h2>{{ labels.systemd_services }}</h2>
                        <div class="segments">
                            <a class="{{ services_running_class }}" href="{{ services_running_path }}">{{ labels.service_filter_running }}</a>
                            <a class="{{ services_failed_class }}" href="{{ services_failed_path }}">{{ labels.service_filter_failed }}</a>
                            <a class="{{ services_stopped_class }}" href="{{ services_stopped_path }}">{{ labels.service_filter_stopped }}</a>
                            <a class="{{ services_all_class }}" href="{{ services_all_path }}">{{ labels.service_filter_all }}</a>
                        </div>
                    </div>
                    {% if has_services %}
                    <table>
                        <thead><tr><th>{{ labels.service }}</th><th>{{ labels.state }}</th><th>{{ labels.name }}</th></tr></thead>
                        <tbody>
                            {% for service in services %}
                            <tr>
                                <td><code>{{ service.name }}</code><br><span class="muted">{{ service.load_state }}</span></td>
                                <td><span class="status {{ service.state_class }}">{{ service.state_label }}</span></td>
                                <td>{{ service.description }}</td>
                            </tr>
                            {% endfor %}
                        </tbody>
                    </table>
                    {% else %}
                    <div class="panel-body muted">{{ labels.no_services }}</div>
                    {% endif %}
                </section>
                {% endif %}

                {% if active_page == "sites" %}
                <section class="panel">
                    <div class="panel-header">
                        <div>
                            <h2>{{ labels.sites }}</h2>
                            <div class="muted">{{ labels.managed_nginx_config }}: <code>{{ managed_nginx_config }}</code></div>
                        </div>
                    </div>
                    {% if has_sites %}
                    <table>
                        <thead><tr><th>{{ labels.domains }}</th><th>{{ labels.upstream }}</th><th>{{ labels.ssl }}</th><th>{{ labels.config_file }}</th></tr></thead>
                        <tbody>
                            {% for site in sites %}
                            <tr><td>{{ site.domain }}</td><td><code>{{ site.upstream }}</code></td><td>{{ site.ssl }}</td><td><code>{{ site.config_path }}</code></td></tr>
                            {% endfor %}
                        </tbody>
                    </table>
                    {% else %}
                    <div class="panel-body muted">{{ labels.no_sites }}</div>
                    {% endif %}
                </section>
                {% endif %}

                {% if active_page == "ssl" %}
                <div class="grid-2">
                    <section class="panel">
                        <div class="panel-header"><h2>{{ labels.nginx }}</h2></div>
                        <div class="panel-body"><span class="status {{ ssl_info.nginx_status_class }}">{{ ssl_info.nginx_status }}</span></div>
                    </section>
                    <section class="panel">
                        <div class="panel-header"><h2>{{ labels.certbot }}</h2></div>
                        <div class="panel-body"><span class="status {{ ssl_info.certbot_status_class }}">{{ ssl_info.certbot_status }}</span></div>
                    </section>
                </div>
                <section class="panel" style="margin-top:20px">
                    <div class="panel-header"><h2>{{ labels.issue_certificate }}</h2></div>
                    <div class="panel-body">
                        <form class="form-grid" method="post" action="{{ certificate_issue_path }}">
                            <input type="hidden" name="lang" value="{{ lang_code }}">
                            <label>{{ labels.domains }}
                                <input name="domain" placeholder="example.com" required>
                            </label>
                            <label>{{ labels.email }}
                                <input name="email" type="email" placeholder="admin@example.com" required>
                            </label>
                            <button type="submit">{{ labels.issue_certificate }}</button>
                        </form>
                        <p class="muted">{{ labels.domain_required }}</p>
                        <dl class="kv">
                            <dt>{{ labels.certificate_status }}</dt>
                            <dd><span class="status {{ certificate_job_status.status_class }}">{{ certificate_job_status.status }}</span></dd>
                            <dt>{{ labels.check_result }}</dt>
                            <dd>{{ certificate_job_status.message }}</dd>
                        </dl>
                        <pre>{{ certificate_job_status.output }}</pre>
                    </div>
                </section>
                <section class="panel" style="margin-top:20px">
                    <div class="panel-header"><h2>{{ labels.certificates }}</h2></div>
                    {% if has_certificates %}
                    <table>
                        <thead><tr><th>{{ labels.certificate }}</th><th>{{ labels.domains }}</th><th>{{ labels.expiry }}</th></tr></thead>
                        <tbody>
                            {% for certificate in ssl_info.certificates %}
                            <tr><td>{{ certificate.name }}</td><td>{{ certificate.domains }}</td><td>{{ certificate.expiry }}</td></tr>
                            {% endfor %}
                        </tbody>
                    </table>
                    {% else %}
                    <div class="panel-body muted">{{ labels.no_certificates }}</div>
                    {% endif %}
                </section>
                {% endif %}

                {% if active_page == "update" %}
                <section class="panel">
                    <div class="panel-header">
                        <h2>{{ labels.update_check }}</h2>
                        <span class="status {{ update_result.status_class }}">{{ update_result.status }}</span>
                    </div>
                    <div class="panel-body">
                        <p class="muted">{{ labels.current_version }}: <code>{{ version }}</code></p>
                        <form method="post" action="{{ update_run_path }}">
                            <button type="submit">{{ labels.run_update }}</button>
                        </form>
                        <p>{{ labels.update_help }}: <code>{{ update_result.update_command }}</code></p>
                        <dl class="kv">
                            <dt>{{ labels.update_status }}</dt>
                            <dd><span class="status {{ update_job_status.status_class }}">{{ update_job_status.status }}</span></dd>
                            <dt>{{ labels.check_result }}</dt>
                            <dd>{{ update_job_status.message }}</dd>
                        </dl>
                        <pre>{{ update_job_status.output }}</pre>
                        <h2>{{ labels.check_result }}</h2>
                        <pre>{{ update_result.output }}</pre>
                    </div>
                </section>
                {% endif %}
            </main>
        </div>
    </div>
</body>
</html>
"#,
    ext = "html"
)]
pub(crate) struct PanelTemplate {
    pub(crate) labels: TextLabels,
    pub(crate) page_title: String,
    pub(crate) active_page: String,
    pub(crate) lang_code: String,
    pub(crate) overview_class: String,
    pub(crate) processes_class: String,
    pub(crate) services_class: String,
    pub(crate) sites_class: String,
    pub(crate) ssl_class: String,
    pub(crate) update_class: String,
    pub(crate) overview_path: String,
    pub(crate) processes_path: String,
    pub(crate) services_path: String,
    pub(crate) services_running_path: String,
    pub(crate) services_failed_path: String,
    pub(crate) services_stopped_path: String,
    pub(crate) services_all_path: String,
    pub(crate) services_running_class: String,
    pub(crate) services_failed_class: String,
    pub(crate) services_stopped_class: String,
    pub(crate) services_all_class: String,
    pub(crate) sites_path: String,
    pub(crate) ssl_path: String,
    pub(crate) update_path: String,
    pub(crate) update_run_path: String,
    pub(crate) certificate_issue_path: String,
    pub(crate) lang_zh_path: String,
    pub(crate) lang_en_path: String,
    pub(crate) logout_path: String,
    pub(crate) version: String,
    pub(crate) metrics: SystemMetrics,
    pub(crate) processes: Vec<ProcessInfo>,
    pub(crate) has_processes: bool,
    pub(crate) process_count: usize,
    pub(crate) services: Vec<ServiceRow>,
    pub(crate) has_services: bool,
    pub(crate) service_count: usize,
    pub(crate) sites: Vec<SiteInfo>,
    pub(crate) has_sites: bool,
    pub(crate) site_count: usize,
    pub(crate) domain_count: usize,
    pub(crate) managed_nginx_config: String,
    pub(crate) ssl_info: SslInfo,
    pub(crate) has_certificates: bool,
    pub(crate) update_result: UpdateCheckResult,
    pub(crate) update_job_status: JobStatus,
    pub(crate) certificate_job_status: JobStatus,
    pub(crate) has_flash: bool,
    pub(crate) flash_class: String,
    pub(crate) flash_message: String,
}

#[derive(Template)]
#[template(
    source = r#"
<!doctype html>
<html lang="{{ labels.html_lang }}">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>RustPanel - {{ labels.login }}</title>
    <style>
        :root { color-scheme: light; --bg: #f6f7f9; --panel: #fff; --line: #d9e0ea; --text: #1f2937; --muted: #64748b; --accent: #0f766e; --bad: #b91c1c; }
        * { box-sizing: border-box; }
        body { min-height: 100vh; margin: 0; display: grid; place-items: center; background: var(--bg); color: var(--text); font: 14px/1.5 ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }
        main { width: min(420px, calc(100vw - 32px)); background: var(--panel); border: 1px solid var(--line); border-radius: 8px; padding: 22px; }
        h1 { margin: 0 0 18px; font-size: 20px; line-height: 1.3; }
        label { display: grid; gap: 6px; margin-bottom: 14px; color: var(--muted); }
        input { width: 100%; border: 1px solid var(--line); border-radius: 6px; padding: 9px 10px; font: inherit; color: var(--text); }
        button { width: 100%; border: 1px solid #0b5c56; background: var(--accent); color: #fff; border-radius: 6px; padding: 10px 12px; font: inherit; cursor: pointer; }
        a { color: var(--accent); text-decoration: none; }
        .error { margin: 0 0 14px; color: var(--bad); }
        .langs { margin-top: 14px; color: var(--muted); font-size: 13px; }
    </style>
</head>
<body>
    <main>
        <h1>RustPanel</h1>
        {% if login_failed %}<p class="error">{{ labels.login_failed }}</p>{% endif %}
        <form method="post" action="{{ login_path }}">
            <input type="hidden" name="lang" value="{{ lang_code }}">
            <label>{{ labels.username }}<input name="username" autocomplete="username" required></label>
            <label>{{ labels.password }}<input name="password" type="password" autocomplete="current-password" required></label>
            <button type="submit">{{ labels.login }}</button>
        </form>
        <div class="langs">{{ labels.language }}: <a href="{{ lang_zh_path }}">{{ labels.zh }}</a> / <a href="{{ lang_en_path }}">{{ labels.en }}</a></div>
    </main>
</body>
</html>
"#,
    ext = "html"
)]
pub(crate) struct LoginTemplate {
    pub(crate) login_path: String,
    pub(crate) lang_zh_path: String,
    pub(crate) lang_en_path: String,
    pub(crate) lang_code: String,
    pub(crate) login_failed: bool,
    pub(crate) labels: TextLabels,
}
