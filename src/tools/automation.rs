use crate::audit;
use crate::config::FeatureGates;
use crate::error::{AetherError, ErrorContext};
use std::fs;

fn run(params: &serde_json::Value, cmd: &str, args: &[&str]) -> String {
    crate::tools::helpers::cmd_params(params, cmd, args)
}

pub fn handle(_gates: &FeatureGates, action: &str, params: serde_json::Value) -> String {
    let ctx = ErrorContext::new("system_automation", action);
    let result = match action {
        "journal_list" => Ok(run(&params, "journalctl", &["--list-boots"])),
        "journal_query" => Ok(run(&params, "journalctl", &["-n", "50", "--no-pager"])),
        "journal_disk_usage" => Ok(run(&params, "journalctl", &["--disk-usage"])),
        "timer_list" => Ok(run(&params, "systemctl", &["list-timers"])),
        "cron_list" => Ok(run(&params, "ls", &["/etc/cron.d"])),
        "cron_user_list" => Ok(run(&params, "crontab", &["-l"])),
        "anacron_list" => Ok(fs::read_to_string("/etc/anacrontab").unwrap_or_default()),
        "at_list" => Ok(run(&params, "atq", &[])),
        "tmpfiles_list" => Ok(run(&params, "ls", &["/usr/lib/tmpfiles.d"])),
        "sysctl_d_list" => Ok(run(&params, "sysctl", &["-a"])),
        "sysctl_d_apply" => {
            crate::tools::helpers::require_force_or_dry_run(&params, &ctx).map(|_| run(&params, "sysctl", &["-p"]))
        }
        "boot_analyze" => Ok(run(&params, "systemd-analyze", &["blame"])),
        other => Err(AetherError::not_implemented(ctx.clone(), other)),
    };
    match &result {
        Ok(_) => audit::log_success("system_automation", action, "ok"),
        Err(e) => audit::log_failure("system_automation", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}
