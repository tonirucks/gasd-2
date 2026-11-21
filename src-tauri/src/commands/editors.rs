use super::{AppState, CommandError, Result, with_service};
use serde::Deserialize;
use std::process::Command;
use tauri::State;

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Editor {
    Vscode,
    Zed,
}

fn editor_command(editor: Editor, folder: &str) -> Command {
    let executable = match editor {
        Editor::Vscode => "code-insiders",
        Editor::Zed => "zed",
    };
    #[cfg(not(windows))]
    let mut command = {
        let mut command = Command::new("/bin/sh");
        command.args(["-lc", &format!("exec {executable} \"$GASD_EDITOR_FOLDER\"")]);
        command
    };
    #[cfg(windows)]
    let mut command = {
        let mut command = Command::new("powershell.exe");
        command.args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!("$ErrorActionPreference = 'Stop'; & {executable} $env:GASD_EDITOR_FOLDER; exit $LASTEXITCODE"),
        ]);
        command
    };
    // Never interpolate paths into shell source.
    command.env("GASD_EDITOR_FOLDER", folder);
    command.current_dir(folder);
    command
}

#[tauri::command]
pub async fn tasks_open_editor(state: State<'_, AppState>, id: i64, editor: Editor) -> Result<()> {
    let task = with_service(state, move |service| service.get_task(id)).await?;
    let folder = task.folder.ok_or_else(|| CommandError {
        code: "validation",
        message: "Link a folder before opening an editor.".into(),
    })?;
    if !std::path::Path::new(&folder).is_dir() {
        return Err(CommandError {
            code: "validation",
            message: "The linked folder no longer exists.".into(),
        });
    }
    tauri::async_runtime::spawn_blocking(move || {
        let output = editor_command(editor, &folder).output().map_err(|_| CommandError {
            code: "editor",
            message: "Could not start the shell to open the editor.".into(),
        })?;
        if !output.status.success() {
            return Err(CommandError {
                code: "editor",
                message: "Could not open the editor. Check that `code-insiders` or `zed` is installed and available on your shell's PATH.".into(),
            });
        }
        Ok(())
    })
    .await
    .map_err(|_| CommandError {
        code: "editor",
        message: "The editor launch failed. Please try again.".into(),
    })?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_is_passed_as_data_not_shell_source() {
        let folder = "/home/user/my project/$(touch injected); ' \" #notes";
        for editor in [Editor::Vscode, Editor::Zed] {
            let command = editor_command(editor, folder);
            assert_eq!(
                command.get_current_dir(),
                Some(std::path::Path::new(folder))
            );
            assert!(command.get_envs().any(|(key, value)| {
                key == "GASD_EDITOR_FOLDER" && value == Some(std::ffi::OsStr::new(folder))
            }));
            assert!(
                !command
                    .get_args()
                    .any(|arg| arg.to_string_lossy().contains(folder))
            );
        }
    }
}
