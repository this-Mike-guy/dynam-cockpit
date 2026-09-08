//! A running desktop is observable; its task activity is not available through
//! the desktop's private stdio app-server connection. Never equate a PID with idle.

use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

pub async fn desktop_is_running() -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(super::process::check_codex_desktop_running)
    .await
    .map_err(|error| format!("Could not check whether Codex is open: {error}"))?
}

/// Ask before a user-initiated operation can close the desktop or replace its
/// profile. Background switching must defer instead of repeatedly prompting.
/// Cancellation is the existing launch cancellation protocol, so no error or
/// retry dialog suggests that the user needs to complete a cancelled switch.
pub async fn confirm_desktop_change(app: &AppHandle) -> Result<(), String> {
    let running = desktop_is_running().await;
    if matches!(running, Ok(false)) {
        return Ok(());
    }

    let status = if running.is_ok() {
        "Codex is open. DYNAM Cockpit cannot reliably tell whether its tasks are idle."
    } else {
        "DYNAM Cockpit could not verify whether Codex is open or running tasks."
    };

    let (sender, receiver) = tokio::sync::oneshot::channel();
    let mut dialog = app
        .dialog()
        .message(format!(
            "{status}\n\n\
             Continuing may close Codex and interrupt active tasks while applying the account or instance change. \
             Finish or stop your tasks first.\n\n\
             Continue only when you are ready to interrupt any active work.",
        ))
        .title("DYNAM Cockpit — Check running tasks")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Continue and allow restart".into(),
            "Cancel".into(),
        ));
    if let Some(window) = app.get_webview_window("main") {
        dialog = dialog.parent(&window);
    }
    dialog.show(move |confirmed| {
        let _ = sender.send(confirmed);
    });

    if receiver.await.unwrap_or(false) {
        Ok(())
    } else {
        Err("CODEX_START_CANCELLED".to_string())
    }
}
