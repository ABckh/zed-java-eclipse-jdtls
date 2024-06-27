use std::fs;

use zed_extension_api::{
    current_platform, download_file, latest_github_release,
    lsp::{Completion, CompletionKind},
    make_file_executable, register_extension, set_language_server_installation_status,
    settings::LspSettings,
    CodeLabel, CodeLabelSpan, DownloadedFileType, Extension, GithubReleaseOptions,
    LanguageServerId, LanguageServerInstallationStatus, Os, Result, Worktree,
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

        let (platform, _arch) = current_platform();
        let asset_name = match platform {
            Os::Mac | Os::Linux => "eclipse.jdt.ls.tar.gz",
            Os::Windows => "eclipse.jdt.ls.zip",
        };
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?} \n", asset_name))?;

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

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<serde_json::Value>> {
        // jdtls only accepts settings via. workspace/didChangeConfiguration, not
        // initialization options, so pass the user's initialization options to
        // workspace/didChangeConfiguration as well.
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;
        Ok(settings.initialization_options)
    }

    fn label_for_completion(
        &self,
        _language_server_id: &LanguageServerId,
        completion: Completion,
    ) -> Option<CodeLabel> {
        match completion.kind? {
            CompletionKind::Method => {
                let (name_and_params, return_type) = completion.label.split_once(" : ")?;
                let name = name_and_params.split('(').next()?;
                let code = format!("{return_type} {name_and_params}");

                Some(CodeLabel {
                    spans: vec![
                        CodeLabelSpan::code_range(return_type.len() + 1..code.len()),
                        CodeLabelSpan::literal(" : ", None),
                        CodeLabelSpan::code_range(0..return_type.len()),
                    ],
                    filter_range: (0..name.len()).into(),
                    code,
                })
            }
            CompletionKind::Constructor => {
                let new = "new ";
                let code = format!("{new}{}", completion.label);
                let name = completion.label.split('(').next()?;

                Some(CodeLabel {
                    spans: vec![CodeLabelSpan::code_range(new.len()..code.len())],
                    filter_range: (0..name.len()).into(),
                    code,
                })
            }
            CompletionKind::Variable | CompletionKind::Field | CompletionKind::Constant => {
                let (name, r#type) = completion.label.split_once(" : ")?;
                let code = format!("{type} {name}");
                let highlight_name = match completion.kind? {
                    CompletionKind::Field => Some("property".to_string()),
                    CompletionKind::Constant => Some("constant".to_string()),
                    _ => None,
                };

                Some(CodeLabel {
                    spans: vec![
                        CodeLabelSpan::literal(name, highlight_name),
                        CodeLabelSpan::literal(" : ", None),
                        CodeLabelSpan::code_range(0..r#type.len()),
                    ],
                    filter_range: (0..name.len()).into(),
                    code,
                })
            }
            CompletionKind::Class | CompletionKind::Interface | CompletionKind::Enum => {
                let (name, namespace) = completion.label.split_once(" - ")?;
                let namespace_hint = format!(" ({namespace})");
                let code = format!("{name}{namespace_hint}");

                Some(CodeLabel {
                    spans: vec![
                        CodeLabelSpan::literal(name, Some("type".to_string())),
                        CodeLabelSpan::literal(namespace_hint, None),
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
            CompletionKind::EnumMember => {
                let name = completion.label.split(" : ").next()?;

                Some(CodeLabel {
                    code: name.to_string(),
                    spans: vec![CodeLabelSpan::code_range(0..name.len())],
                    filter_range: (0..name.len()).into(),
                })
            }
            _ => None,
        }
    }
}

register_extension!(JavaExtension);
