use std::fs;

use zed_extension_api::{
    current_platform, download_file, latest_github_release,
    lsp::{Completion, CompletionKind},
    make_file_executable, register_extension, set_language_server_installation_status, CodeLabel,
    CodeLabelSpan, DownloadedFileType, Extension, GithubReleaseOptions, LanguageServerId,
    LanguageServerInstallationStatus, Os, Result, Worktree,
};

struct JavaExtension {
    cached_binary_path: Option<String>,
}

impl JavaExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<String> {
        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        if let Some(path) = worktree.which("jdtls") {
            self.cached_binary_path = Some(path.clone());
            return Ok(path);
        }

        set_language_server_installation_status(
            &language_server_id,
            &LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = latest_github_release(
            "ABckh/eclipse.jdt.ls",
            GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let asset_name = "eclipse.jdt.ls.tar.gz";
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?} \n", asset_name))?;

        let (platform, _arch) = current_platform();
        let version_dir = "eclipse.jdt.ls";
        let binary_path = format!("{version_dir}/bin/jdtls");

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            set_language_server_installation_status(
                &language_server_id,
                &LanguageServerInstallationStatus::Downloading,
            );

            download_file(
                &asset.download_url,
                &version_dir,
                match platform {
                    Os::Mac | Os::Linux => DownloadedFileType::GzipTar,
                    Os::Windows => DownloadedFileType::Zip,
                },
            )
            .map_err(|e| format!("failed to download file: {e}"))?;

            make_file_executable(&binary_path)?;

            let entries =
                fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
                if entry.file_name().to_str() != Some(&version_dir) {
                    fs::remove_dir_all(&entry.path()).ok();
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl Extension for JavaExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<zed_extension_api::Command> {
        Ok(zed_extension_api::Command {
            command: self.language_server_binary_path(language_server_id, worktree)?,
            args: Vec::new(),
            env: Default::default(),
        })
    }

    fn label_for_completion(
        &self,
        _language_server_id: &LanguageServerId,
        completion: Completion,
    ) -> Option<CodeLabel> {
        match completion.kind? {
            CompletionKind::Method => {
                let (name_and_params, return_type) = completion.label.split_once(" : ")?;
                let (name, _) = name_and_params.split_once('(')?;
                let code = format!("{return_type} {name_and_params}");

                Some(CodeLabel {
                    spans: vec![CodeLabelSpan::code_range(0..code.len())],
                    filter_range: (return_type.len() + 1..return_type.len() + 1 + name.len())
                        .into(),
                    code,
                })
            }
            CompletionKind::Class | CompletionKind::Interface => {
                let (name, _namespace) = completion.label.split_once(" - ")?;
                let import_hint = format!(" (import {})", completion.detail?);
                let code = format!("{name}{import_hint}");

                Some(CodeLabel {
                    spans: vec![
                        CodeLabelSpan::literal(name, Some("type".to_string())),
                        CodeLabelSpan::literal(import_hint, None),
                    ],
                    filter_range: (0..name.len()).into(),
                    code,
                })
            }
            CompletionKind::Keyword => Some(CodeLabel {
                spans: vec![CodeLabelSpan::code_range(0..completion.label.len())],
                filter_range: (0..completion.label.len()).into(),
                code: completion.label,
            }),
            _ => {
                dbg!(&completion);

                None
            }
        }
    }
}

register_extension!(JavaExtension);
