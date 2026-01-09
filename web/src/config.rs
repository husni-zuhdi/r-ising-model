use std::env;

/// Struct Config for setup environment variables
#[derive(PartialEq, Debug, Clone)]
pub struct Config {
    /// Service Endpoint
    /// Default to localhost.
    /// Usually set to 127.0.0.1 or 0.0.0.0.
    pub svc_endpoint: String,
    /// Service Port
    /// Listening port of service.
    /// Default to 8080.
    /// Usually set to 80, 443, 3000, or 8080.
    pub svc_port: String,
    /// Log Level
    /// From `tracing:Level`.
    /// Default to INFO.
    /// Set to DEBUG for development. Usually set to INFO or WARN in production.
    pub log_level: tracing::Level,
    /// Environment
    /// Type of environment.
    /// Default to `Release`. Can be `Development` or `Release`.
    pub environment: Environment,
}

/// Environment Type
#[derive(PartialEq, Debug, Clone)]
pub enum Environment {
    Development,
    Release,
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Default for Config {
    /// By default running on localhost:8080 in release
    /// with log-level info and data from memory
    fn default() -> Self {
        let svc_endpoint: String = "localhost".to_string();
        let svc_port: String = "8080".to_string();
        let log_level = tracing::Level::INFO;
        let environment = Environment::Release;

        Self {
            svc_endpoint,
            svc_port,
            log_level,
            environment,
        }
    }
}

impl Config {
    /// Setup config from environment variables
    pub async fn from_envar() -> Self {
        // Required
        let svc_endpoint: String = env::var("SVC_ENDPOINT")
            .expect("Failed to load SVC_ENDPOINT environment variable. Double check your config");
        let svc_port: String = env::var("SVC_PORT")
            .expect("failed to load SVC_PORT environment variable. Double check your config");
        let log_level = Self::parse_log_level();
        let environment = Self::parse_environment();

        Self {
            svc_endpoint,
            svc_port,
            log_level,
            environment,
        }
    }
    /// Parse Log Level
    fn parse_environment() -> Environment {
        match env::var("ENVIRONMENT") {
            Err(e) => {
                println!(
                "Failed to load ENVIRONMENT environment variable. Set default to 'Release'. Error {e}"
            );
                Environment::Release
            }
            Ok(val) => match val.as_str() {
                "release" | "Release" | "RELEASE" => Environment::Release,
                "development" | "Development" | "DEVELOPMENT" => Environment::Development,
                _ => Environment::Release,
            },
        }
    }
    /// Parse Log Level
    fn parse_log_level() -> tracing::Level {
        match env::var("LOG_LEVEL") {
            Err(e) => {
                println!(
                "Failed to load LOG_LEVEL environment variable. Set default to 'info'. Error {e}"
            );
                tracing::Level::INFO
            }
            Ok(val) => match val.as_str() {
                "ERROR" => tracing::Level::ERROR,
                "WARN" => tracing::Level::WARN,
                "INFO" => tracing::Level::INFO,
                "DEBUG" => tracing::Level::DEBUG,
                "TRACE" => tracing::Level::TRACE,
                _ => tracing::Level::INFO,
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default() {
        let svc_endpoint = "localhost";
        let svc_port = "8080";
        let log_level = tracing::Level::INFO;
        let environment = Environment::Release;

        let result = Config::default();

        assert_eq!(result.svc_endpoint, svc_endpoint);
        assert_eq!(result.svc_port, svc_port);
        assert_eq!(result.log_level, log_level);
        assert_eq!(result.environment, environment);
    }
}
