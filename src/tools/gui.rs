use crate::audit;
use crate::config::FeatureGates;
use crate::error::{self, AetherError, ErrorContext};

fn run(params: &serde_json::Value, cmd: &str, args: &[&str]) -> String {
    crate::tools::helpers::cmd_params(params, cmd, args)
}

fn ps(params: &serde_json::Value, key: &str, ctx: &ErrorContext) -> String {
    error::get_string_param(params, key, ctx).unwrap_or("").to_string()
}

pub fn handle(_gates: &FeatureGates, action: &str, params: serde_json::Value) -> String {
    let ctx = ErrorContext::new("gui_automation", action);
    let result = match action {
        "mouse_move" => Ok(run(&params, "xdotool", &["mousemove", &ps(&params, "x", &ctx), &ps(&params, "y", &ctx)])),
        "mouse_click" => Ok(run(&params, "xdotool", &["click", &ps(&params, "button", &ctx)])),
        "mouse_position" => Ok(run(&params, "xdotool", &["getmouselocation"])),
        "keyboard_type" => Ok(run(&params, "xdotool", &["type", &ps(&params, "text", &ctx)])),
        "keyboard_press" => Ok(run(&params, "xdotool", &["key", &ps(&params, "key", &ctx)])),
        "find_window" => Ok(run(&params, "xdotool", &["search", "--name", &ps(&params, "title", &ctx)])),
        "list_windows" => Ok(run(&params, "wmctrl", &["-l"])),
        "screenshot" => Ok(run(&params, "import", &["-window", "root", "/tmp/screenshot.png"])),
        "clipboard_read" => Ok(run(&params, "xclip", &["-selection", "clipboard", "-o"])),
        "display_list" => Ok(run(&params, "xrandr", &["--listmonitors"])),
        "display_info" => Ok(run(&params, "xrandr", &["--verbose"])),
        "audio_list_sinks" => Ok(run(&params, "pactl", &["list", "sinks", "short"])),
        "audio_volume_get" => Ok(run(&params, "pactl", &["get-sink-volume", "@DEFAULT_SINK@"])),
        "audio_volume_set" => {
            Ok(run(&params, "pactl", &["set-sink-volume", "@DEFAULT_SINK@", &ps(&params, "volume", &ctx)]))
        }
        "audio_mute" => Ok(run(&params, "pactl", &["set-sink-mute", "@DEFAULT_SINK@", "toggle"])),
        "screensaver_lock" => Ok(run(&params, "loginctl", &["lock-session"])),
        other => Err(AetherError::not_implemented(ctx.clone(), other)),
    };
    match &result {
        Ok(_) => audit::log_success("gui_automation", action, "ok"),
        Err(e) => audit::log_failure("gui_automation", action, &e.to_string()),
    }
    result.unwrap_or_else(|e| format!("Error: {e}"))
}
