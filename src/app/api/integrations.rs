use crate::api::schema::{
    ExperimentalIntegrationInfo, IntegrationInfo, IntegrationInstallResult, IntegrationState,
    IntegrationUninstallResult, ResponseResult,
};
use crate::app::App;

use super::responses::{encode_error, encode_success};

impl App {
    pub(super) fn handle_integration_list(&self, id: String) -> String {
        let integrations = crate::integration::integration_recommendations()
            .into_iter()
            .map(|recommendation| IntegrationInfo {
                target: recommendation.target,
                label: recommendation.label.to_owned(),
                command: recommendation.command.to_owned(),
                available: recommendation.available,
                state: match recommendation.state {
                    crate::integration::IntegrationStatusKind::NotInstalled => {
                        IntegrationState::NotInstalled
                    }
                    crate::integration::IntegrationStatusKind::Current => IntegrationState::Current,
                    crate::integration::IntegrationStatusKind::Outdated => {
                        IntegrationState::Outdated
                    }
                },
            })
            .collect();
        let experimental = crate::integration::experimental_integration_statuses()
            .into_iter()
            .map(|status| ExperimentalIntegrationInfo {
                label: status.label.to_owned(),
                state: match status.state {
                    crate::integration::IntegrationStatusKind::NotInstalled => {
                        IntegrationState::NotInstalled
                    }
                    crate::integration::IntegrationStatusKind::Current => IntegrationState::Current,
                    crate::integration::IntegrationStatusKind::Outdated => {
                        IntegrationState::Outdated
                    }
                },
            })
            .collect();
        encode_success(
            id,
            ResponseResult::IntegrationList {
                integrations,
                experimental,
            },
        )
    }

    pub(super) fn handle_integration_install(
        &mut self,
        id: String,
        params: crate::api::schema::IntegrationInstallParams,
    ) -> String {
        let target = params.target;
        let messages = match crate::integration::install_target(target) {
            Ok(messages) => messages,
            Err(err) => return encode_error(id, "integration_install_failed", err.to_string()),
        };
        self.state.integration_recommendations = crate::integration::integration_recommendations();

        encode_success(
            id,
            ResponseResult::IntegrationInstall {
                target,
                details: IntegrationInstallResult { messages },
            },
        )
    }

    pub(super) fn handle_integration_uninstall(
        &mut self,
        id: String,
        params: crate::api::schema::IntegrationUninstallParams,
    ) -> String {
        let target = params.target;
        let messages = match crate::integration::uninstall_target(target) {
            Ok(messages) => messages,
            Err(err) => return encode_error(id, "integration_uninstall_failed", err.to_string()),
        };
        self.state.integration_recommendations = crate::integration::integration_recommendations();

        encode_success(
            id,
            ResponseResult::IntegrationUninstall {
                target,
                details: IntegrationUninstallResult { messages },
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let (_api_tx, api_rx) = tokio::sync::mpsc::unbounded_channel();
        App::new(
            &crate::config::Config::default(),
            crate::app::AppPolicy::TEST,
            None,
            api_rx,
            crate::api::EventHub::default(),
        )
    }

    #[test]
    fn integration_list_payload_carries_experimental_targets() {
        let _lock = crate::integration::integration_env_lock();
        let base = std::env::temp_dir().join(format!(
            "herdr-integration-list-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let env_vars = [
            ("MUSE_CONFIG_DIR", "muse"),
            ("KIRO_CONFIG_DIR", "kiro"),
            ("COMMANDCODE_CONFIG_DIR", "commandcode"),
        ];
        for (env_var, dir) in env_vars {
            let path = base.join(dir);
            std::fs::create_dir_all(&path).unwrap();
            std::env::set_var(env_var, &path);
        }

        let encoded = test_app().handle_integration_list("req_experimental".into());
        let response: crate::api::schema::SuccessResponse =
            serde_json::from_str(&encoded).expect("success response");
        let crate::api::schema::ResponseResult::IntegrationList { experimental, .. } =
            response.result
        else {
            panic!("expected integration list result");
        };
        let labels: Vec<&str> = experimental.iter().map(|row| row.label.as_str()).collect();
        for expected in ["muse", "kiro", "command-code"] {
            assert!(
                labels.contains(&expected),
                "missing {expected} in {labels:?}"
            );
        }

        for (env_var, _) in env_vars {
            std::env::remove_var(env_var);
        }
        let _ = std::fs::remove_dir_all(&base);
    }
}
