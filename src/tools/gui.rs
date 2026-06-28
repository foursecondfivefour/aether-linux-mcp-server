use crate::audit;
use crate::config::FeatureGates;
use crate::error::{self, AetherError, ErrorContext};
use std::process::Command;

fn run(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd).args(args).output()
        .map(|o| format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
        .unwrap_or_else(|_| format!("'{}' not available", cmd))
}

fn ps(params: &serde_json::Value, key: &str, ctx: &ErrorContext) -> String {
    error::get_string_param(params, key, ctx).unwrap_or("").to_string()
}

pub fn handle(_gates: &FeatureGates, action: &str, params: serde_json::Value) -> String {
    let ctx = ErrorContext::new("gui_automation", action);
    let result = match action {
        "mouse_move" => Ok(run("xdotool", &["mousemove", &ps(&params, "x", &ctx), &ps(&params, "y", &ctx)])),
        "mouse_click" => Ok(run("xdotool", &["click", &ps(&params, "button", &ctx)])),
        "mouse_position" => Ok(run("xdotool", &["getmouselocation"])),
        "keyboard_type" => Ok(run("xdotool", &["type", &ps(&params, "text", &ctx)])),
        "keyboard_press" => Ok(run("xdotool", &["key", &ps(&params, "key", &ctx)])),
        "find_window" => Ok(run("xdotool", &["search", "--name", &ps(&params, "title", &ctx)])),
        "list_windows" => Ok(run("wmctrl", &["-l"])),
        "screenshot" => Ok(run("import", &["-window", "root", "/tmp/screenshot.png"])),
        "clipboard_read" => Ok(run("xclip", &["-selection", "clipboard", "-o"])),
        "display_list" => Ok(run("xrandr", &["--listmonitors"])),
        "display_info" => Ok(run("xrandr", &["--verbose"])),
        "audio_list_sinks" => Ok(run("pactl", &["list", "sinks", "short"])),
        "audio_volume_get" => Ok(run("pactl", &["get-sink-volume", "@DEFAULT_SINK@"])),
        "audio_volume_set" => Ok(run("pactl", &["set-sink-volume", "@DEFAULT_SINK@", &ps(&params, "volume", &ctx)])),
        "audio_mute" => Ok(run("pactl", &["set-sink-mute", "@DEFAULT_SINK@", "toggle"])),
        "screensaver_lock" => Ok(run("loginctl", &["lock-session"])),
        other => Err(AetherError::not_implemented(ctx.clone(), other)),
    };
    match &result {
        Ok(_) => audit::log_success("gui_automation", action, "ok"),
        Err(e) => audit::log_failure("gui_automation", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}
