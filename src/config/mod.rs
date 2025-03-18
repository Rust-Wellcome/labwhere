use labwhere::errors::LabwhereError;
use log::info;
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct AppConfig {
    pub(crate) database_directory: Option<String>,
    pub(crate) environment: Option<String>,
}

/// Reads the configuration from a file and deserializes it into an `AppConfig` struct.
///
/// # Arguments
///
/// * `path` - A string slice that holds the path to the configuration file.
///
/// # Returns
///
/// * `Ok(AppConfig)` - If the configuration file is successfully read and deserialized.
/// * `Err(LabwhereError)` - If there is an error reading the file or deserializing the content.
///
/// # Errors
///
/// This function will return an error if:
/// * The file cannot be read.
/// * The content of the file cannot be deserialized into an `AppConfig` struct.
///
/// # Example
///
/// ```rust
/// let config = read_config("config.yaml").unwrap();
/// println!("{:?}", config);
/// ```
pub(crate) async fn read_config(path: &str) -> Result<AppConfig, LabwhereError> {
    // Read from config and serialise into AppConfig
    match fs::read_to_string(path).await {
        Ok(content) => {
            let result: AppConfig = serde_yml::from_str(&content).unwrap();
            info!("{}", format!("{:?}", result));
            Ok(result)
        }
        Err(_err) => return Err(LabwhereError::config_error("Failed to read config.")),
    }
}
