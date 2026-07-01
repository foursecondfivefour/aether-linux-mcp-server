use crate::audit;
use crate::config::FeatureGates;
use crate::error::{AetherError, ErrorContext};
use std::fs;
use std::process::Command;

fn run(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd)
        .args(args)
        .output()
        .map(|o| format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
        .unwrap_or_else(|_| format!("'{}' not available", cmd))
}

pub fn handle(_gates: &FeatureGates, action: &str, _params: serde_json::Value) -> String {
    let ctx = ErrorContext::new("system_automation", action);
    let result = match action {
        "journal_list" => Ok(run("journalctl", &["--list-boots"])),
        "journal_query" => Ok(run("journalctl", &["-n", "50", "--no-pager"])),
        "journal_disk_usage" => Ok(run("journalctl", &["--disk-usage"])),
        "timer_list" => Ok(run("systemctl", &["list-timers"])),
        "cron_list" => Ok(run("ls", &["/etc/cron.d"])),
        "cron_user_list" => Ok(run("crontab", &["-l"])),
        "anacron_list" => Ok(fs::read_to_string("/etc/anacrontab").unwrap_or_default()),
        "at_list" => Ok(run("atq", &[])),
        "tmpfiles_list" => Ok(run("ls", &["/usr/lib/tmpfiles.d"])),
        "sysctl_d_list" => Ok(run("sysctl", &["-a"])),
        "sysctl_d_apply" => Ok(run("sysctl", &["-p"])),
        "boot_analyze" => Ok(run("systemd-analyze", &["blame"])),
        other => Err(AetherError::not_implemented(ctx.clone(), other)),
    };
    match &result {
        Ok(_) => audit::log_success("system_automation", action, "ok"),
        Err(e) => audit::log_failure("system_automation", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}
