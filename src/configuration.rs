use config::{Config, ConfigError, FileFormat};
use directories::ProjectDirs;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    #[serde(rename = "matcher")]
    pub matcher_config: Option<MatcherConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MatcherConfig {
    pub ignore_specs: Option<Vec<MatcherSpec>>,
    pub skip_specs: Option<Vec<MatcherSpec>>,
}

#[derive(Debug, Deserialize)]
pub struct MatcherSpec {
    pub pattern: String,
}

impl Configuration {
    pub fn load(app_name: &str) -> Result<Configuration, ConfigError> {
        let project_dirs = ProjectDirs::from("", "", app_name).unwrap();
        let config_dir = project_dirs.config_dir();
        let mut config = Config::new();

        if let Some(file_path) = config_dir.join(format!("{}{}", app_name, ".yaml")).to_str() {
            config.merge(config::File::new(file_path, FileFormat::Yaml).required(false))?;
        }

        config.try_into()
    }
}
