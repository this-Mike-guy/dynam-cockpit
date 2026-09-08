// Process 模块测试：平台路径、Codex 启动参数和进程清理行为。
// 保持测试模块位于原作用域，super 引用和 cfg 条件不变。
#[cfg(test)]
mod legacy_platform_adapter_cleanup_tests {
    use super::{orphaned_legacy_platform_adapter_pid_from_ps_line, utf8_command_output_snippet};

    #[test]
    fn matches_orphaned_legacy_platform_adapter() {
        let line = " 1359     1 /Users/jieli/.antigravity_cockpit/platform-packages/codex/current/adapter/macos/cockpit-codex-adapter";
        assert_eq!(
            orphaned_legacy_platform_adapter_pid_from_ps_line(line, 99999),
            Some(1359)
        );
    }

    #[test]
    fn ignores_non_orphaned_or_current_processes() {
        let line = " 1359 1805 /Users/jieli/.antigravity_cockpit/platform-packages/codex/current/adapter/macos/cockpit-codex-adapter";
        assert_eq!(
            orphaned_legacy_platform_adapter_pid_from_ps_line(line, 99999),
            None
        );

        let current_line = " 1359 1 /Users/jieli/.antigravity_cockpit/platform-packages/codex/current/adapter/macos/cockpit-codex-adapter";
        assert_eq!(
            orphaned_legacy_platform_adapter_pid_from_ps_line(current_line, 1359),
            None
        );
    }

    #[test]
    fn ignores_current_sidecar_and_official_apps() {
        let sidecar =
            " 64680 1805 /Applications/Cockpit Tools.app/Contents/MacOS/cockpit-cliproxy --parent-pid 1805";
        assert_eq!(
            orphaned_legacy_platform_adapter_pid_from_ps_line(sidecar, 99999),
            None
        );

        let official_codex =
            " 9300 1 /Applications/Codex.app/Contents/Frameworks/Codex Framework.framework/Helpers/browser_crashpad_handler";
        assert_eq!(
            orphaned_legacy_platform_adapter_pid_from_ps_line(official_codex, 99999),
            None
        );
    }

    #[test]
    fn command_output_snippet_drops_non_utf8_bytes() {
        assert_eq!(utf8_command_output_snippet(&[0xb4, 0xed, 0xce, 0xf3]), None);
    }

    #[test]
    fn command_output_snippet_keeps_utf8_text() {
        assert_eq!(
            utf8_command_output_snippet("No such process\n".as_bytes()).as_deref(),
            Some("No such process")
        );
    }
}

#[cfg(test)]
mod managed_sidecar_port_cleanup_tests {
    use super::{
        command_line_parent_pid, managed_sidecar_command_matches,
        managed_sidecar_parent_allows_cleanup, normalized_process_argument,
    };
    use std::path::Path;

    #[test]
    fn parses_sidecar_parent_pid_forms() {
        assert_eq!(
            command_line_parent_pid("cockpit-cliproxy --parent-pid 1805"),
            Some(1805)
        );
        assert_eq!(
            command_line_parent_pid("cockpit-cliproxy --parent-pid=64680 --config x"),
            Some(64680)
        );
        assert_eq!(command_line_parent_pid("cockpit-cliproxy"), None);
    }

    #[test]
    fn requires_expected_sidecar_binary_and_config_path() {
        let config =
            Path::new("/Users/demo/.antigravity_cockpit/codex_local_access_sidecar/config.json");
        let command = "/Applications/Cockpit Tools.app/Contents/MacOS/cockpit-cliproxy --config /Users/demo/.antigravity_cockpit/codex_local_access_sidecar/config.json --parent-pid 1805";
        assert!(managed_sidecar_command_matches(
            command,
            "cockpit-cliproxy",
            config
        ));
        assert!(!managed_sidecar_command_matches(
            command,
            "cockpit-cliproxy",
            Path::new("/Users/demo/another/config.json")
        ));
        assert!(!managed_sidecar_command_matches(
            "python server.py --config /Users/demo/.antigravity_cockpit/codex_local_access_sidecar/config.json",
            "cockpit-cliproxy",
            config
        ));
    }

    #[test]
    fn normalizes_windows_process_arguments() {
        assert_eq!(
            normalized_process_argument(
                r#""C:\Program Files\Cockpit Tools\cockpit-cliproxy.exe""#
            ),
            "c:/program files/cockpit tools/cockpit-cliproxy.exe"
        );
    }

    #[test]
    fn cleanup_rejects_live_siblings_and_detached_sidecars() {
        assert!(managed_sidecar_parent_allows_cleanup(
            Some(1805),
            1805,
            true
        ));
        assert!(managed_sidecar_parent_allows_cleanup(
            Some(1804),
            1805,
            false
        ));
        assert!(!managed_sidecar_parent_allows_cleanup(
            Some(1804),
            1805,
            true
        ));
        assert!(!managed_sidecar_parent_allows_cleanup(
            Some(0),
            1805,
            false
        ));
        assert!(!managed_sidecar_parent_allows_cleanup(
            None, 1805, false
        ));
    }
}

#[cfg(all(test, target_os = "macos"))]
mod qoder_macos_process_tests {
    use super::is_qoder_macos_main_process_command_line;

    #[test]
    fn matches_current_and_legacy_qoder_main_processes() {
        assert!(is_qoder_macos_main_process_command_line(
            "/Applications/Qoder IDE.app/Contents/MacOS/Qoder"
        ));
        assert!(is_qoder_macos_main_process_command_line(
            "/Applications/Qoder.app/Contents/MacOS/Qoder"
        ));
        assert!(!is_qoder_macos_main_process_command_line(
            "/Applications/Qoder IDE.app/Contents/Frameworks/Qoder Helper.app/Contents/MacOS/Qoder Helper --type=gpu-process --user-data-dir=/tmp/qoder"
        ));
        assert!(!is_qoder_macos_main_process_command_line(
            "/Applications/Qoder IDE.app/Contents/Frameworks/Electron Framework.framework/Helpers/chrome_crashpad_handler"
        ));
    }
}

#[cfg(all(test, target_os = "macos"))]
mod codex_macos_launch_tests {
    use super::{
        is_codex_direct_app_server_command_line, is_codex_macos_main_process_command_line,
        select_codex_direct_app_server_descendants, CodexProcessTreeEntry,
    };

    #[test]
    fn matches_chatgpt_and_legacy_codex_main_processes() {
        assert!(is_codex_macos_main_process_command_line(
            "/applications/chatgpt.app/contents/macos/chatgpt"
        ));
        assert!(is_codex_macos_main_process_command_line(
            "/applications/codex.app/contents/macos/codex"
        ));
        assert!(!is_codex_macos_main_process_command_line(
            "/applications/chatgpt.app/contents/resources/codex app-server"
        ));
    }

    #[test]
    fn matches_only_bundled_direct_app_server_commands() {
        let executable = "/Applications/ChatGPT.app/Contents/Resources/codex";
        assert!(is_codex_direct_app_server_command_line(
            "/Applications/ChatGPT.app/Contents/Resources/codex app-server --analytics-default-enabled",
            executable,
        ));
        assert!(is_codex_direct_app_server_command_line(
            "\"/Applications/ChatGPT.app/Contents/Resources/codex\" app-server --listen stdio://",
            executable,
        ));
        assert!(!is_codex_direct_app_server_command_line(
            "/Applications/ChatGPT.app/Contents/Resources/codex app-server daemon",
            executable,
        ));
        assert!(!is_codex_direct_app_server_command_line(
            "/opt/homebrew/bin/codex app-server --analytics-default-enabled",
            executable,
        ));
        assert!(!is_codex_direct_app_server_command_line(
            "/Applications/ChatGPT.app/Contents/Resources/codex exec hello",
            executable,
        ));
    }

    #[test]
    fn selects_direct_app_server_only_from_target_main_process_tree() {
        let executable = "/Applications/ChatGPT.app/Contents/Resources/codex";
        let entries = vec![
            CodexProcessTreeEntry {
                pid: 100,
                parent_pid: 1,
                command_line: "/Applications/ChatGPT.app/Contents/MacOS/ChatGPT".to_string(),
            },
            CodexProcessTreeEntry {
                pid: 110,
                parent_pid: 100,
                command_line: "helper".to_string(),
            },
            CodexProcessTreeEntry {
                pid: 120,
                parent_pid: 110,
                command_line: format!("{} app-server --analytics-default-enabled", executable),
            },
            CodexProcessTreeEntry {
                pid: 200,
                parent_pid: 1,
                command_line: "/Applications/ChatGPT.app/Contents/MacOS/ChatGPT".to_string(),
            },
            CodexProcessTreeEntry {
                pid: 220,
                parent_pid: 200,
                command_line: format!("{} app-server --analytics-default-enabled", executable),
            },
        ];

        assert_eq!(
            select_codex_direct_app_server_descendants(&entries, &[100], executable),
            vec![120]
        );
    }
}

#[cfg(test)]
mod codex_launch_args_tests {
    use super::{
        build_codex_app_launch_args, codex_managed_store_launch_unsafe_error,
        CODEX_MANAGED_STORE_LAUNCH_UNSAFE_PREFIX,
    };

    #[test]
    fn keeps_user_launch_args_without_adding_remote_debugging() {
        assert!(build_codex_app_launch_args(&[]).is_empty());
        assert_eq!(
            build_codex_app_launch_args(&[
                " --remote-debugging-port=9333 ".to_string(),
                "".to_string(),
                " --disable-gpu ".to_string(),
            ]),
            vec![
                "--remote-debugging-port=9333".to_string(),
                "--disable-gpu".to_string(),
            ]
        );
    }

    #[test]
    fn managed_store_launch_error_is_machine_readable_and_keeps_causes() {
        let error = codex_managed_store_launch_unsafe_error("denied", "fallback failed");
        assert!(error.starts_with(CODEX_MANAGED_STORE_LAUNCH_UNSAFE_PREFIX));
        assert!(error.contains("direct_error=denied"));
        assert!(error.contains("powershell_error=fallback failed"));
    }
}

#[cfg(test)]
mod codex_linux_layout_tests {
    use super::{
        linux_codex_discovery_paths, select_codex_direct_app_server_descendants,
        CodexProcessTreeEntry,
    };
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    #[test]
    fn discovery_includes_official_linux_package_and_path_launchers() {
        let home = Path::new("/home/demo");
        let path = OsString::from("/custom/bin:/usr/bin");
        let candidates = linux_codex_discovery_paths(Some(home), Some(path.as_os_str()));
        assert!(candidates.contains(&PathBuf::from("/usr/bin/chatgpt")));
        assert!(candidates.contains(&PathBuf::from("/usr/lib/chatgpt/ChatGPT")));
        assert!(candidates.contains(&PathBuf::from("/home/demo/.local/bin/chatgpt")));
        assert!(candidates.contains(&PathBuf::from("/custom/bin/chatgpt")));
    }

    #[test]
    fn selects_linux_bundled_app_server_from_target_desktop_tree() {
        let executable = "/usr/lib/chatgpt/resources/codex";
        let entries = vec![
            CodexProcessTreeEntry {
                pid: 100,
                parent_pid: 1,
                command_line: "/usr/lib/chatgpt/ChatGPT".to_string(),
            },
            CodexProcessTreeEntry {
                pid: 120,
                parent_pid: 100,
                command_line: format!("{} app-server --analytics-default-enabled", executable),
            },
            CodexProcessTreeEntry {
                pid: 220,
                parent_pid: 200,
                command_line: format!("{} app-server --analytics-default-enabled", executable),
            },
        ];
        assert_eq!(
            select_codex_direct_app_server_descendants(&entries, &[100], executable),
            vec![120]
        );
    }
}

#[cfg(test)]
mod codex_path_migration_tests {
    use super::{
        is_codex_embedded_backend_executable, score_windows_candidate,
        should_migrate_legacy_codex_launch_path, should_probe_legacy_codex_launch_path,
    };
    use std::collections::HashSet;
    use std::path::Path;

    #[test]
    fn migrates_official_windows_store_codex_path_when_chatgpt_exists() {
        assert!(should_migrate_legacy_codex_launch_path(
            Path::new(
                r"C:\Program Files\WindowsApps\OpenAI.Codex_1.0.0.0_x64__8wekyb3d8bbwe\app\Codex.exe"
            ),
            Path::new(
                r"C:\Program Files\WindowsApps\OpenAI.Codex_2.0.0.0_x64__2p2nqsd0c76g0\app\ChatGPT.exe"
            ),
        ));
    }

    #[test]
    fn keeps_legacy_path_when_chatgpt_is_not_detected() {
        assert!(!should_migrate_legacy_codex_launch_path(
            Path::new(
                r"C:\Program Files\WindowsApps\OpenAI.Codex_1.0.0.0_x64__8wekyb3d8bbwe\app\Codex.exe"
            ),
            Path::new(
                r"C:\Program Files\WindowsApps\OpenAI.Codex_1.0.0.0_x64__8wekyb3d8bbwe\app\Codex.exe"
            ),
        ));
    }

    #[test]
    fn does_not_replace_custom_codex_executable() {
        assert!(!should_migrate_legacy_codex_launch_path(
            Path::new(r"D:\Tools\Codex.exe"),
            Path::new(
                r"C:\Program Files\WindowsApps\OpenAI.ChatGPT_2.0.0.0_x64__8wekyb3d8bbwe\app\ChatGPT.exe"
            ),
        ));
    }

    #[test]
    fn migrates_official_macos_codex_path_when_chatgpt_exists() {
        assert!(should_migrate_legacy_codex_launch_path(
            Path::new("/Applications/Codex.app"),
            Path::new("/Applications/ChatGPT.app/Contents/MacOS/ChatGPT"),
        ));
        assert!(should_migrate_legacy_codex_launch_path(
            Path::new("/Applications/Codex.app/Contents/MacOS/Codex"),
            Path::new("/Applications/ChatGPT.app"),
        ));
    }

    #[test]
    fn keeps_macos_legacy_path_when_chatgpt_is_not_detected() {
        assert!(!should_migrate_legacy_codex_launch_path(
            Path::new("/Applications/Codex.app"),
            Path::new("/Applications/Codex.app/Contents/MacOS/Codex"),
        ));
    }

    #[test]
    fn does_not_replace_custom_macos_codex_path() {
        assert!(!should_migrate_legacy_codex_launch_path(
            Path::new("/Users/test/Applications/Codex.app"),
            Path::new("/Applications/ChatGPT.app/Contents/MacOS/ChatGPT"),
        ));
    }

    #[test]
    fn only_probes_official_legacy_codex_paths_for_migration() {
        assert!(should_probe_legacy_codex_launch_path(Path::new(
            r"C:\Program Files\WindowsApps\OpenAI.Codex_1.0.0.0_x64__8wekyb3d8bbwe\app\Codex.exe"
        )));
        assert!(should_probe_legacy_codex_launch_path(Path::new(
            "/Applications/Codex.app"
        )));
        assert!(!should_probe_legacy_codex_launch_path(Path::new(
            r"C:\Program Files\WindowsApps\OpenAI.ChatGPT_2.0.0.0_x64__8wekyb3d8bbwe\app\ChatGPT.exe"
        )));
        assert!(!should_probe_legacy_codex_launch_path(Path::new(
            r"D:\Tools\Codex.exe"
        )));
        assert!(!should_probe_legacy_codex_launch_path(Path::new(
            "/Users/test/Applications/Codex.app"
        )));
    }

    #[test]
    fn scan_rejects_codex_keyword_helper_executables() {
        let exe_names = HashSet::from(["chatgpt.exe".to_string(), "codex.exe".to_string()]);
        let keywords = vec!["chatgpt".to_string(), "codex".to_string()];

        assert!(score_windows_candidate(
            Path::new("C:/Tools/CodexHelper.exe"),
            &exe_names,
            &keywords,
        )
        .is_none());
        assert!(
            score_windows_candidate(Path::new("C:/Tools/ChatGPT.exe"), &exe_names, &keywords,)
                .is_some()
        );
    }

    #[test]
    fn scan_excludes_embedded_resources_backend() {
        assert!(is_codex_embedded_backend_executable(Path::new(
            r"C:\Users\Test\AppData\Local\OpenAI\Codex\bin\8e5b6932251c2c1c\codex.exe"
        )));
        assert!(is_codex_embedded_backend_executable(Path::new(
            r"C:\Program Files\WindowsApps\OpenAI.Codex_26.707.9564.0_x64__2p2nqsd0c76g0\app\resources\codex.exe"
        )));
        assert!(!is_codex_embedded_backend_executable(Path::new(
            r"C:\Program Files\WindowsApps\OpenAI.Codex_26.707.9564.0_x64__2p2nqsd0c76g0\app\ChatGPT.exe"
        )));
        assert!(!is_codex_embedded_backend_executable(Path::new(
            r"C:\Program Files\WindowsApps\OpenAI.Codex_1.0.0.0_x64__2p2nqsd0c76g0\app\Codex.exe"
        )));
    }
}

#[cfg(test)]
mod codex_windows_launch_preflight_tests {
    use super::{
        find_codex_windows_app_main_exe, is_codex_windows_store_desktop_executable,
        is_valid_windows_codex_launch_candidate, parse_codex_store_version_from_dir_name,
        resolve_windows_codex_custom_path, should_migrate_legacy_codex_launch_path,
    };
    use std::path::{Path, PathBuf};

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("dynam-codex-launch-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn recognizes_codex_package_even_when_desktop_is_named_chatgpt() {
        for path in [
            r"C:\Program Files\WindowsApps\OpenAI.Codex_26.901.6511.0_x64__2p2nqsd0c76g0\app\ChatGPT.exe",
            "C:/Program Files/WindowsApps/OpenAI.Codex_26.901.6511.0_x64__2p2nqsd0c76g0/app/Codex.exe",
        ] {
            assert!(is_codex_windows_store_desktop_executable(Path::new(path)));
            assert!(is_valid_windows_codex_launch_candidate(Path::new(path)));
        }
        assert_eq!(
            parse_codex_store_version_from_dir_name("OpenAI.Codex_26.901.6511.0_x64__2p2nqsd0c76g0"),
            Some(vec![26, 901, 6511, 0])
        );
    }

    #[test]
    fn rejects_classic_and_backend_even_when_the_filename_matches() {
        for path in [
            r"C:\Program Files\WindowsApps\OpenAI.ChatGPT-Desktop_1.0.0.0_x64__publisher\app\ChatGPT.exe",
            r"C:\Program Files\WindowsApps\OpenAI.ChatGPT_1.0.0.0_x64__publisher\app\ChatGPT.exe",
            r"C:\Program Files\WindowsApps\OpenAI.Codex_26.901.6511.0_x64__2p2nqsd0c76g0\app\resources\codex.exe",
            r"C:\Users\Test\AppData\Local\OpenAI\Codex\bin\buildhash\codex.exe",
        ] {
            assert!(!is_codex_windows_store_desktop_executable(Path::new(path)));
            assert!(!is_valid_windows_codex_launch_candidate(Path::new(path)));
        }
        assert!(parse_codex_store_version_from_dir_name("OpenAI.ChatGPT_1.0.0.0_x64__publisher").is_none());
        assert!(!should_migrate_legacy_codex_launch_path(
            Path::new(r"C:\Program Files\WindowsApps\OpenAI.Codex_1.0.0.0_x64__publisher\app\Codex.exe"),
            Path::new(r"C:\Program Files\WindowsApps\OpenAI.ChatGPT-Desktop_2.0.0.0_x64__publisher\app\ChatGPT.exe"),
        ));
    }

    #[test]
    fn resolves_custom_application_folder_to_file_before_launch() {
        let fixture = Fixture::new();
        let app_dir = fixture.0.join("app");
        std::fs::create_dir(&app_dir).unwrap();
        // An executable-looking directory must not satisfy preflight.
        std::fs::create_dir(app_dir.join("ChatGPT.exe")).unwrap();
        assert!(find_codex_windows_app_main_exe(&app_dir).is_none());
        assert!(resolve_windows_codex_custom_path(fixture.0.to_str().unwrap()).is_none());
        let executable = app_dir.join("Codex.exe");
        std::fs::write(&executable, b"synthetic executable fixture").unwrap();
        assert_eq!(resolve_windows_codex_custom_path(fixture.0.to_str().unwrap()), Some(executable.clone()));
        assert_eq!(resolve_windows_codex_custom_path(executable.to_str().unwrap()), Some(executable));
        assert!(resolve_windows_codex_custom_path(fixture.0.join("missing.exe").to_str().unwrap()).is_none());
    }
}

#[cfg(test)]
mod codex_desktop_launch_confirmation_tests {
    use super::{observe_codex_desktop_pid_until, wait_for_observed_codex_desktop_pid};
    use std::cell::Cell;
    use std::collections::VecDeque;
    use std::time::Duration;

    #[test]
    fn waits_for_late_desktop_after_launcher_handoff() {
        // None is an unmatched launcher; only the later GUI is recognized by
        // the profile-aware process probe.
        let mut observations = VecDeque::from([None, None, Some(42), Some(42)]);
        let result = wait_for_observed_codex_desktop_pid(
            || observations.pop_front().flatten(),
            Duration::from_secs(1),
            Duration::ZERO,
        );
        assert_eq!(result, Some(42));
        assert!(observations.is_empty());
    }

    #[test]
    fn transient_process_does_not_complete_launch_confirmation() {
        let mut observations = VecDeque::from([Some(7), None, Some(42), Some(42)]);
        let result = wait_for_observed_codex_desktop_pid(
            || observations.pop_front().flatten(),
            Duration::from_secs(1),
            Duration::ZERO,
        );
        assert_eq!(result, Some(42));
        assert!(observations.is_empty());
    }

    #[test]
    fn slow_successful_discovery_still_gets_one_stability_check() {
        let elapsed = Cell::new(Duration::ZERO);
        let mut calls = 0;
        let result = observe_codex_desktop_pid_until(
            || {
                calls += 1;
                elapsed.set(Duration::from_secs(5));
                Some(42)
            },
            || elapsed.get(),
            Duration::from_secs(3),
            Duration::ZERO,
        );
        assert_eq!(result, Some(42));
        assert_eq!(calls, 2);
    }

    #[test]
    fn exited_or_unmatched_process_times_out_without_success() {
        let mut observations = VecDeque::from([Some(7), None]);
        let result = wait_for_observed_codex_desktop_pid(
            || observations.pop_front().flatten(),
            Duration::from_millis(10),
            Duration::from_millis(1),
        );
        assert_eq!(result, None);
    }
}

#[cfg(test)]
mod codex_windows_app_server_cleanup_tests {
    use super::{codex_windows_captured_process_matches, is_codex_windows_direct_app_server};
    use std::path::Path;

    const BACKEND: &str = r"C:\Users\Test\AppData\Local\OpenAI\Codex\bin\buildhash\codex.exe";

    fn matches(parent: u32, executable: &str, args: &[&str]) -> bool {
        let arguments: Vec<String> = std::iter::once(executable)
            .chain(args.iter().copied())
            .map(str::to_string)
            .collect();
        is_codex_windows_direct_app_server(parent, Path::new(executable), &arguments, &[100])
    }

    #[test]
    fn only_captures_direct_stdio_children_of_selected_desktops() {
        assert!(matches(100, BACKEND, &["app-server"]));
        assert!(matches(100, BACKEND, &["-c", "features.code_mode_host=true", "app-server", "--analytics-default-enabled", "-c", "synthetic=true"]));
        assert!(matches(100, BACKEND, &["--config=synthetic=true", "app-server"]));
        assert!(!matches(100, BACKEND, &["-c", "app-server"]));
        assert!(!matches(100, BACKEND, &["-c", "synthetic=app-server", "exec"]));
        assert!(matches(100, BACKEND, &["app-server", "--listen", "stdio://"]));
        assert!(matches(100, BACKEND, &["app-server", "--listen=stdio://"]));
        assert!(!matches(200, BACKEND, &["app-server"]));
        assert!(!matches(0, BACKEND, &["app-server"]));
        assert!(!matches(100, BACKEND, &["exec", "synthetic"]));
        assert!(!matches(100, BACKEND, &["app-server", "daemon", "start"]));
        assert!(!matches(100, BACKEND, &["app-server", "--listen", "ws://127.0.0.1:1234"]));
        assert!(!matches(100, BACKEND, &["app-server", "--listen=ws://127.0.0.1:1234"]));
        assert!(!matches(100, r"C:\Other\codex.exe", &["app-server"]));
    }

    #[test]
    fn refuses_reused_pid_or_changed_executable_before_cleanup() {
        let original = Path::new(BACKEND);
        assert!(codex_windows_captured_process_matches(1000, original, 1000, original));
        assert!(!codex_windows_captured_process_matches(1000, original, 1001, original));
        assert!(!codex_windows_captured_process_matches(0, original, 0, original));
        assert!(!codex_windows_captured_process_matches(1000, original, 1000, Path::new(r"C:\Other\codex.exe")));
    }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::{
        running_app_candidate_matches, windows_app_launch_signature,
        windows_trae_candidate_matches_platform,
    };
    use crate::modules::trae_account::TraePlatformKind;
    use std::path::Path;

    #[test]
    fn windows_launch_signatures_cover_provider_apps() {
        for app in [
            "antigravity_ide",
            "cursor",
            "zed",
            "codebuddy",
            "codebuddy_cn",
            "qoder",
            "trae",
            "trae_solo",
            "trae_cn",
            "trae_solo_cn",
            "workbuddy",
            "windsurf",
            "kiro",
            "codex",
            "claude",
            "vscode",
        ] {
            let signature =
                windows_app_launch_signature(app).unwrap_or_else(|| panic!("missing {app}"));
            assert!(
                !signature.exe_names.is_empty(),
                "{app} must define executable names"
            );
            assert!(
                !signature.common_paths.is_empty(),
                "{app} must define common install paths"
            );
            assert!(
                !signature.display_keywords.is_empty(),
                "{app} must define display keywords"
            );
        }
    }

    #[test]
    fn antigravity_ide_signature_uses_ide_executable_only() {
        let signature = windows_app_launch_signature("antigravity_ide")
            .expect("antigravity ide signature must exist");
        assert!(signature
            .exe_names
            .iter()
            .any(|name| name.eq_ignore_ascii_case("Antigravity IDE.exe")));
        assert!(!signature
            .exe_names
            .iter()
            .any(|name| name.eq_ignore_ascii_case("Antigravity.exe")));
    }

    #[test]
    fn codex_signature_accepts_chatgpt_and_legacy_codex_executables() {
        let signature = windows_app_launch_signature("codex").expect("codex signature must exist");
        assert!(signature
            .exe_names
            .iter()
            .any(|name| name.eq_ignore_ascii_case("ChatGPT.exe")));
        assert!(signature
            .exe_names
            .iter()
            .any(|name| name.eq_ignore_ascii_case("Codex.exe")));
    }

    #[test]
    fn running_codex_match_accepts_main_executable_and_rejects_embedded_backend() {
        let signature = windows_app_launch_signature("codex").expect("codex signature must exist");
        assert!(running_app_candidate_matches(
            "codex",
            Path::new(
                r"C:\Program Files\WindowsApps\OpenAI.Codex_26.707.9564.0_x64__2p2nqsd0c76g0\app\ChatGPT.exe"
            ),
            signature,
        ));
        assert!(!running_app_candidate_matches(
            "codex",
            Path::new(
                r"C:\Program Files\WindowsApps\OpenAI.Codex_26.707.9564.0_x64__2p2nqsd0c76g0\app\resources\codex.exe"
            ),
            signature,
        ));
    }

    #[test]
    fn running_claude_match_requires_claude_executable() {
        let signature =
            windows_app_launch_signature("claude").expect("claude signature must exist");
        assert!(running_app_candidate_matches(
            "claude",
            Path::new(r"C:\Program Files\WindowsApps\Claude_1.0.0\app\Claude.exe"),
            signature,
        ));
        assert!(!running_app_candidate_matches(
            "claude",
            Path::new(r"C:\Tools\Electron.exe"),
            signature,
        ));
    }

    #[test]
    fn trae_windows_scan_candidates_match_exact_platform_dirs() {
        let trae = Path::new(r"D:\Users\李杰\AppData\Local\Programs\Trae\Trae.exe");
        let trae_cn = Path::new(r"D:\Users\李杰\AppData\Local\Programs\Trae CN\Trae CN.exe");
        let solo_cn =
            Path::new(r"D:\Users\李杰\AppData\Local\Programs\TRAE SOLO CN\TRAE SOLO CN.exe");

        assert!(windows_trae_candidate_matches_platform(
            trae,
            TraePlatformKind::Trae
        ));
        assert!(!windows_trae_candidate_matches_platform(
            trae,
            TraePlatformKind::TraeCn
        ));
        assert!(windows_trae_candidate_matches_platform(
            trae_cn,
            TraePlatformKind::TraeCn
        ));
        assert!(!windows_trae_candidate_matches_platform(
            trae_cn,
            TraePlatformKind::Trae
        ));
        assert!(windows_trae_candidate_matches_platform(
            solo_cn,
            TraePlatformKind::TraeSoloCn
        ));
        assert!(!windows_trae_candidate_matches_platform(
            solo_cn,
            TraePlatformKind::TraeSolo
        ));
    }
}
