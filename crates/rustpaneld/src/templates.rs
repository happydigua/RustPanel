use askama::Template;

use crate::{
    i18n::TextLabels, services::ServiceUnit, system_metrics::SystemMetrics,
    updates::UpdateCheckResult,
};

#[derive(Clone)]
pub(crate) struct AppRow {
    pub(crate) name: String,
    pub(crate) id: String,
    pub(crate) service_name: String,
    pub(crate) upstream: String,
    pub(crate) domain_count: usize,
    pub(crate) deploy_step_count: usize,
}

#[derive(Template)]
#[template(
    source = r#"
<!doctype html>
<html lang="{{ labels.html_lang }}">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>RustPanel</title>
    <style>
        :root {
            color-scheme: light;
            --bg: #f6f7f9;
            --panel: #fff;
            --line: #d9e0ea;
            --text: #1f2937;
            --muted: #64748b;
            --accent: #0f766e;
            --ok: #047857;
            --bad: #b91c1c;
        }
        * { box-sizing: border-box; }
        body {
            margin: 0;
            background: var(--bg);
            color: var(--text);
            font: 14px/1.5 ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
        }
        .topbar {
            min-height: 56px;
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 16px;
            padding: 0 24px;
            border-bottom: 1px solid var(--line);
            background: var(--panel);
        }
        .brand { font-weight: 700; }
        .top-actions { display: flex; align-items: center; gap: 12px; color: var(--muted); font-size: 13px; }
        a { color: var(--accent); text-decoration: none; }
        main { width: min(1120px, calc(100vw - 32px)); margin: 28px auto; }
        .metrics { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 12px; margin-bottom: 20px; }
        .metric, .panel { background: var(--panel); border: 1px solid var(--line); border-radius: 8px; }
        .metric { padding: 16px; }
        .metric span { display: block; color: var(--muted); font-size: 13px; }
        .metric strong { display: block; margin-top: 6px; font-size: 24px; line-height: 1.2; }
        .stack { display: grid; gap: 20px; }
        .panel-header { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 16px 18px; border-bottom: 1px solid var(--line); }
        .panel-body { padding: 16px 18px; }
        h1 { margin: 0; font-size: 18px; line-height: 1.3; }
        .button { border: 1px solid #0b5c56; background: var(--accent); color: #fff; border-radius: 6px; padding: 8px 12px; text-decoration: none; white-space: nowrap; }
        table { width: 100%; border-collapse: collapse; }
        th, td { padding: 12px 18px; border-bottom: 1px solid var(--line); text-align: left; vertical-align: middle; }
        th { color: var(--muted); font-size: 12px; font-weight: 600; text-transform: uppercase; }
        tr:last-child td { border-bottom: 0; }
        code { padding: 2px 5px; border-radius: 4px; background: #eef2f6; color: #243244; font: 12px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }
        .muted { color: var(--muted); }
        .state-active { color: var(--ok); }
        .state-failed { color: var(--bad); }
        .system-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 14px; }
        .kv { margin: 0; display: grid; gap: 6px; }
        .kv dt { color: var(--muted); font-size: 13px; }
        .kv dd { margin: 0; }
        @media (max-width: 760px) {
            .topbar, .top-actions { align-items: flex-start; flex-direction: column; }
            .topbar { padding: 14px 16px; }
            main { width: calc(100vw - 24px); margin: 18px auto; }
            .metrics, .system-grid { grid-template-columns: 1fr; }
            table, thead, tbody, tr, th, td { display: block; }
            thead { display: none; }
            tr { padding: 12px 0; border-bottom: 1px solid var(--line); }
            td { border: 0; padding: 4px 16px; }
        }
    </style>
</head>
<body>
    <header class="topbar">
        <div class="brand">RustPanel</div>
        <div class="top-actions">
            <span>{{ labels.local_daemon }}</span>
            <span>{{ labels.language }}: <a href="{{ lang_zh_path }}">{{ labels.zh }}</a> / <a href="{{ lang_en_path }}">{{ labels.en }}</a></span>
            <a href="{{ logout_path }}">{{ labels.logout }}</a>
        </div>
    </header>
    <main>
        <section class="metrics" aria-label="Overview">
            <div class="metric"><span>{{ labels.sites }}</span><strong>{{ app_count }}</strong></div>
            <div class="metric"><span>{{ labels.domains }}</span><strong>{{ domain_count }}</strong></div>
            <div class="metric"><span>{{ labels.systemd_services }}</span><strong>{{ service_count }}</strong></div>
        </section>

        <div class="stack">
            <section class="panel">
                <div class="panel-header">
                    <h1>{{ labels.system }}</h1>
                    <a class="button" href="{{ update_check_path }}">{{ labels.update_check }}</a>
                </div>
                <div class="panel-body">
                    <div class="system-grid">
                        <dl class="kv"><dt>{{ labels.current_version }}</dt><dd><code>{{ version }}</code></dd></dl>
                        <dl class="kv"><dt>{{ labels.update_help }}</dt><dd><code>{{ labels.update_command }}</code></dd></dl>
                    </div>
                </div>
            </section>

            <section class="panel">
                <div class="panel-header"><h1>{{ labels.system_resources }}</h1></div>
                <div class="panel-body">
                    <div class="system-grid">
                        <dl class="kv"><dt>{{ labels.load_average }}</dt><dd><code>{{ metrics.load_average }}</code></dd></dl>
                        <dl class="kv"><dt>{{ labels.memory }}</dt><dd><code>{{ metrics.memory }}</code></dd></dl>
                        <dl class="kv"><dt>{{ labels.disk }}</dt><dd><code>{{ metrics.disk }}</code></dd></dl>
                        <dl class="kv"><dt>{{ labels.uptime }}</dt><dd><code>{{ metrics.uptime }}</code></dd></dl>
                    </div>
                </div>
            </section>

            <section class="panel">
                <div class="panel-header"><h1>{{ labels.systemd_services }}</h1></div>
                {% if has_services %}
                <table>
                    <thead>
                        <tr><th>{{ labels.service }}</th><th>{{ labels.state }}</th><th>{{ labels.name }}</th></tr>
                    </thead>
                    <tbody>
                        {% for service in service_units %}
                        <tr>
                            <td><code>{{ service.name }}</code><br><span class="muted">{{ service.load_state }}</span></td>
                            <td><span class="state-{{ service.active_state }}">{{ service.active_state }} / {{ service.sub_state }}</span></td>
                            <td>{{ service.description }}</td>
                        </tr>
                        {% endfor %}
                    </tbody>
                </table>
                {% else %}
                <div class="panel-body muted">{{ labels.no_services }}</div>
                {% endif %}
            </section>

            <section class="panel">
                <div class="panel-header">
                    <div>
                        <h1>{{ labels.sites }}</h1>
                        <div class="muted">{{ labels.managed_nginx_config }}: <code>{{ nginx_conf_dir }}</code></div>
                    </div>
                    <a class="button" href="{{ apps_path }}">{{ labels.open_list }}</a>
                </div>
                <table>
                    <thead>
                        <tr><th>{{ labels.name }}</th><th>{{ labels.service }}</th><th>{{ labels.upstream }}</th><th>{{ labels.domains }}</th><th>{{ labels.deploy }}</th></tr>
                    </thead>
                    <tbody>
                        {% for app in rows %}
                        <tr>
                            <td><strong>{{ app.name }}</strong><br><span class="muted">{{ app.id }}</span></td>
                            <td><code>{{ app.service_name }}</code></td>
                            <td><code>{{ app.upstream }}</code></td>
                            <td>{{ app.domain_count }}</td>
                            <td>{{ app.deploy_step_count }}</td>
                        </tr>
                        {% endfor %}
                    </tbody>
                </table>
            </section>
        </div>
    </main>
</body>
</html>
"#,
    ext = "html"
)]
pub(crate) struct DashboardTemplate {
    pub(crate) rows: Vec<AppRow>,
    pub(crate) service_units: Vec<ServiceUnit>,
    pub(crate) has_services: bool,
    pub(crate) app_count: usize,
    pub(crate) domain_count: usize,
    pub(crate) service_count: usize,
    pub(crate) nginx_conf_dir: String,
    pub(crate) apps_path: String,
    pub(crate) update_check_path: String,
    pub(crate) lang_zh_path: String,
    pub(crate) lang_en_path: String,
    pub(crate) logout_path: String,
    pub(crate) version: String,
    pub(crate) metrics: SystemMetrics,
    pub(crate) labels: TextLabels,
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

#[derive(Template)]
#[template(
    source = r#"
<table>
    <thead>
        <tr><th>{{ labels.name }}</th><th>{{ labels.service }}</th><th>{{ labels.upstream }}</th><th>{{ labels.domains }}</th></tr>
    </thead>
    <tbody>
        {% for app in rows %}
        <tr>
            <td>{{ app.name }}</td>
            <td>{{ app.service_name }}</td>
            <td>{{ app.upstream }}</td>
            <td>{{ app.domain_count }}</td>
        </tr>
        {% endfor %}
    </tbody>
</table>
"#,
    ext = "html"
)]
pub(crate) struct AppsPartialTemplate {
    pub(crate) rows: Vec<AppRow>,
    pub(crate) labels: TextLabels,
}

#[derive(Template)]
#[template(
    source = r#"
<!doctype html>
<html lang="{{ labels.html_lang }}">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>RustPanel - {{ labels.update_check }}</title>
    <style>
        :root { color-scheme: light; --bg: #f6f7f9; --panel: #fff; --line: #d9e0ea; --text: #1f2937; --muted: #64748b; --accent: #0f766e; --ok: #047857; --warn: #b45309; --bad: #b91c1c; }
        * { box-sizing: border-box; }
        body { margin: 0; background: var(--bg); color: var(--text); font: 14px/1.5 ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }
        .topbar { min-height: 56px; display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 0 24px; border-bottom: 1px solid var(--line); background: var(--panel); }
        .brand { font-weight: 700; }
        .top-actions { display: flex; align-items: center; gap: 12px; color: var(--muted); font-size: 13px; }
        a { color: var(--accent); text-decoration: none; }
        main { width: min(920px, calc(100vw - 32px)); margin: 28px auto; }
        .panel { background: var(--panel); border: 1px solid var(--line); border-radius: 8px; }
        .panel-header { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 16px 18px; border-bottom: 1px solid var(--line); }
        .panel-body { padding: 16px 18px; }
        h1, h2 { margin: 0; font-size: 18px; line-height: 1.3; }
        h2 { margin-top: 18px; }
        .status { display: inline-flex; border-radius: 6px; padding: 4px 8px; font-weight: 600; }
        .status.ok { color: var(--ok); background: #ecfdf5; }
        .status.warn { color: var(--warn); background: #fffbeb; }
        .status.error { color: var(--bad); background: #fef2f2; }
        code, pre { border-radius: 6px; background: #eef2f6; color: #243244; font: 12px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }
        code { padding: 2px 5px; }
        pre { overflow: auto; padding: 12px; white-space: pre-wrap; }
        .muted { color: var(--muted); }
        @media (max-width: 760px) {
            .topbar, .top-actions { align-items: flex-start; flex-direction: column; }
            .topbar { padding: 14px 16px; }
            main { width: calc(100vw - 24px); margin: 18px auto; }
        }
    </style>
</head>
<body>
    <header class="topbar">
        <div class="brand">RustPanel</div>
        <div class="top-actions">
            <a href="{{ dashboard_path }}">{{ labels.back_dashboard }}</a>
            <span>{{ labels.language }}: <a href="{{ lang_zh_path }}">{{ labels.zh }}</a> / <a href="{{ lang_en_path }}">{{ labels.en }}</a></span>
            <a href="{{ logout_path }}">{{ labels.logout }}</a>
        </div>
    </header>
    <main>
        <section class="panel">
            <div class="panel-header">
                <h1>{{ labels.update_check }}</h1>
                <span class="status {{ result.status_class }}">{{ result.status }}</span>
            </div>
            <div class="panel-body">
                <p class="muted">{{ labels.current_version }}: <code>{{ version }}</code></p>
                <p>{{ labels.update_help }}: <code>{{ result.update_command }}</code></p>
                <h2>{{ labels.check_result }}</h2>
                <pre>{{ result.output }}</pre>
            </div>
        </section>
    </main>
</body>
</html>
"#,
    ext = "html"
)]
pub(crate) struct UpdateCheckTemplate {
    pub(crate) dashboard_path: String,
    pub(crate) lang_zh_path: String,
    pub(crate) lang_en_path: String,
    pub(crate) logout_path: String,
    pub(crate) version: String,
    pub(crate) result: UpdateCheckResult,
    pub(crate) labels: TextLabels,
}
