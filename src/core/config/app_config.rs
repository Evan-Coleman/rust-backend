use super::constants;
use config::{Config, ConfigError, Environment, File};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;
use std::time::Duration;
use tracing::info;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    #[serde(default = "default_protocol")]
    pub protocol: String,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheConfig {
    pub enabled: bool,
    pub ttl_seconds: u64,
    pub max_capacity: u64,
    #[serde(default = "default_reconnect_interval")]
    pub reconnect_interval_seconds: u64,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub enabled: bool,
    pub url: String,
    pub max_connections: u32,
    pub connect_timeout_seconds: u64,
    pub idle_timeout_seconds: Option<u64>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: "postgres://postgres:postgres@localhost:5432/app".to_string(),
            max_connections: 10,
            connect_timeout_seconds: 30,
            idle_timeout_seconds: Some(300),
        }
    }
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiConfig {
    pub petstore_url: String,
    pub api_key: Option<String>,
}

/// Entra ID (Azure AD) authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntraConfig {
    /// Tenant ID (from environment variable)
    #[serde(default)]
    pub tenant_id: String,

    /// Client ID (from environment variable)
    #[serde(default)]
    pub client_id: String,

    /// Audience (from environment variable)
    #[serde(default)]
    pub audience: String,

    /// Scope (from environment variable)
    #[serde(default)]
    pub scope: String,

    /// Token URL (from environment variable)
    #[serde(default)]
    pub token_url: String,

    /// JWKS URI format (for key discovery)
    #[serde(default = "default_jwks_uri_format")]
    pub jwks_uri_format: String,

    /// Authorize URL format
    #[serde(default = "default_authorize_url_format")]
    pub authorize_url_format: String,

    /// Token URL format
    #[serde(default = "default_token_url_format")]
    pub token_url_format: String,

    /// Issuer URL formats
    #[serde(default = "default_issuer_url_formats")]
    pub issuer_url_formats: Vec<String>,

    /// Admin roles (users with these roles can access admin endpoints)
    pub admin_roles: Vec<String>,

    /// Read-only roles (users with these roles can access read-only endpoints)
    pub read_only_roles: Vec<String>,

    /// Full access roles (users with these roles can access full access endpoints)
    pub full_access_roles: Vec<String>,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub enabled: bool,
    pub debug: bool,
    pub entra: EntraConfig,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            debug: false,
            entra: EntraConfig {
                tenant_id: env::var(constants::auth::env_vars::TENANT_ID).unwrap_or_default(),
                client_id: env::var(constants::auth::env_vars::CLIENT_ID).unwrap_or_default(),
                audience: env::var(constants::auth::env_vars::AUDIENCE).unwrap_or_default(),
                scope: env::var(constants::auth::env_vars::SCOPE).unwrap_or_default(),
                token_url: env::var(constants::auth::env_vars::TOKEN_URL).unwrap_or_default(),
                jwks_uri_format: default_jwks_uri_format(),
                authorize_url_format: default_authorize_url_format(),
                token_url_format: default_token_url_format(),
                issuer_url_formats: default_issuer_url_formats(),
                admin_roles: Vec::new(),
                read_only_roles: Vec::new(),
                full_access_roles: Vec::new(),
            },
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "json".to_string()
}

/// Reliability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityConfig {
    /// Retry configuration
    #[serde(default)]
    pub retry: RetryConfig,

    /// Circuit breaker configuration
    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig,

    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limit: RateLimitConfig,

    /// Timeout configuration
    #[serde(default)]
    pub timeout: TimeoutConfig,

    /// Concurrency limits
    #[serde(default)]
    pub concurrency: ConcurrencyConfig,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Whether retries are enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Maximum number of retry attempts
    #[serde(default = "default_retry_attempts")]
    pub max_attempts: u32,

    /// Base delay between retries in milliseconds
    #[serde(default = "default_retry_delay")]
    pub base_delay_ms: u64,

    /// Max delay between retries in milliseconds
    #[serde(default = "default_retry_max_delay")]
    pub max_delay_ms: u64,

    /// Whether to use exponential backoff
    #[serde(default = "default_true")]
    pub use_exponential_backoff: bool,

    /// Status codes that should trigger a retry
    #[serde(default = "default_retry_status_codes")]
    pub retry_status_codes: Vec<u16>,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Whether circuit breaker is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Number of consecutive failures before opening the circuit (legacy mode)
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold: u32,

    /// Time window in seconds for tracking failure rate
    #[serde(default = "default_window_seconds")]
    pub window_seconds: u64,

    /// Failure percentage threshold (0-100) that triggers the circuit breaker
    #[serde(default = "default_failure_percentage")]
    pub failure_percentage: u8,

    /// Whether to use the legacy consecutive failures mode (false = use rolling window)
    #[serde(default = "default_false")]
    pub use_consecutive_failures: bool,

    /// HTTP status codes that should be considered failures
    #[serde(default = "default_failure_status_codes")]
    pub failure_status_codes: Vec<u16>,

    /// Time in milliseconds the circuit stays open before moving to half-open
    #[serde(default = "default_reset_timeout")]
    pub reset_timeout_ms: u64,

    /// Number of successful requests in half-open state to close the circuit
    #[serde(default = "default_success_threshold")]
    pub success_threshold: u32,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Whether rate limiting is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Number of requests allowed per time window
    #[serde(default = "default_rate_limit")]
    pub requests_per_window: u32,

    /// Time window in seconds
    #[serde(default = "default_rate_window")]
    pub window_seconds: u64,

    /// Whether to apply per-client rate limiting
    #[serde(default = "default_false")]
    pub per_client: bool,
}

/// Timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Whether to enable request timeouts
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}

/// Concurrency configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyConfig {
    /// Whether to limit concurrency
    #[serde(default = "default_false")]
    pub enabled: bool,

    /// Maximum number of concurrent requests
    #[serde(default = "default_max_concurrency")]
    pub max_concurrent_requests: u32,
}

impl Default for ReliabilityConfig {
    fn default() -> Self {
        Self {
            retry: RetryConfig::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
            rate_limit: RateLimitConfig::default(),
            timeout: TimeoutConfig::default(),
            concurrency: ConcurrencyConfig::default(),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            max_attempts: default_retry_attempts(),
            base_delay_ms: default_retry_delay(),
            max_delay_ms: default_retry_max_delay(),
            use_exponential_backoff: default_true(),
            retry_status_codes: default_retry_status_codes(),
        }
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            failure_threshold: default_failure_threshold(),
            window_seconds: default_window_seconds(),
            failure_percentage: default_failure_percentage(),
            use_consecutive_failures: default_false(),
            failure_status_codes: default_failure_status_codes(),
            reset_timeout_ms: default_reset_timeout(),
            success_threshold: default_success_threshold(),
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            requests_per_window: default_rate_limit(),
            window_seconds: default_rate_window(),
            per_client: default_false(),
        }
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            timeout_seconds: default_timeout(),
        }
    }
}

impl Default for ConcurrencyConfig {
    fn default() -> Self {
        Self {
            enabled: default_false(),
            max_concurrent_requests: default_max_concurrency(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_retry_attempts() -> u32 {
    3
}

fn default_retry_delay() -> u64 {
    100
}

fn default_retry_max_delay() -> u64 {
    1000
}

fn default_failure_threshold() -> u32 {
    5
}

fn default_reset_timeout() -> u64 {
    30000
}

fn default_success_threshold() -> u32 {
    2
}

fn default_rate_limit() -> u32 {
    100
}

fn default_rate_window() -> u64 {
    60
}

fn default_timeout() -> u64 {
    30
}

fn default_max_concurrency() -> u32 {
    100
}

fn default_retry_status_codes() -> Vec<u16> {
    vec![408, 429, 500, 502, 503, 504]
}

fn default_window_seconds() -> u64 {
    60
}

fn default_failure_percentage() -> u8 {
    50
}

fn default_failure_status_codes() -> Vec<u16> {
    vec![500, 502, 503, 504]
}

/// Default protocol value (http/https)
fn default_protocol() -> String {
    "http".to_string()
}

/// Default JWKS URI format for Microsoft Entra ID
pub fn default_jwks_uri_format() -> String {
    "https://login.microsoftonline.com/{}/discovery/v2.0/keys".to_string()
}

/// Default authorize URL format for Microsoft Entra ID
pub fn default_authorize_url_format() -> String {
    "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize".to_string()
}

/// Default token URL format for Microsoft Entra ID
pub fn default_token_url_format() -> String {
    "https://login.microsoftonline.com/{}/oauth2/v2.0/token".to_string()
}

/// Default issuer URL formats for Microsoft Entra ID
pub fn default_issuer_url_formats() -> Vec<String> {
    vec![
        // v2.0 endpoints
        "https://login.microsoftonline.com/{}/v2.0".to_string(),
        "https://login.microsoftonline.com/{}/v2.0/".to_string(),
        // v1.0 endpoints
        "https://sts.windows.net/{}".to_string(),
        "https://sts.windows.net/{}/".to_string(),
        // Additional formats
        "https://login.microsoftonline.com/{}".to_string(),
        "https://login.microsoftonline.com/{}/".to_string(),
    ]
}

/// OpenAPI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiConfig {
    /// Name of the OpenAPI spec file (just the filename, not the full path)
    #[serde(default = "default_openapi_spec_file")]
    pub spec_file: String,
}

impl Default for OpenApiConfig {
    fn default() -> Self {
        Self {
            spec_file: default_openapi_spec_file(),
        }
    }
}

/// Default name for the OpenAPI spec file
fn default_openapi_spec_file() -> String {
    "rust-backend.yaml".to_string()
}

/// Environment type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EnvironmentType {
    Development,
    Testing,
    Staging,
    Production,
}

impl Default for EnvironmentType {
    fn default() -> Self {
        // Default to development for safety
        EnvironmentType::Development
    }
}

impl std::fmt::Display for EnvironmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnvironmentType::Development => write!(f, "development"),
            EnvironmentType::Testing => write!(f, "testing"),
            EnvironmentType::Staging => write!(f, "staging"),
            EnvironmentType::Production => write!(f, "production"),
        }
    }
}

impl From<String> for EnvironmentType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "development" | "dev" => EnvironmentType::Development,
            "testing" | "test" => EnvironmentType::Testing,
            "staging" => EnvironmentType::Staging,
            "production" | "prod" => EnvironmentType::Production,
            _ => EnvironmentType::Development, // Default to development for safety
        }
    }
}

/// Management endpoint security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointSecurityConfig {
    /// Whether basic health check is public (no auth)
    #[serde(default = "default_true")]
    pub public_health: bool,

    /// Whether detailed health info is available
    #[serde(default = "default_health_details")]
    pub expose_health_details: bool,

    /// Whether detailed health check is public (no auth) - usually false in production
    #[serde(default = "default_false")]
    pub public_detailed_health: bool,

    /// Whether metrics endpoint is public (no auth) - usually false in production
    #[serde(default = "default_false")]
    pub public_metrics: bool,

    /// Whether to show sensitive info in health checks - usually false in production
    #[serde(default = "default_false")]
    pub expose_sensitive_info: bool,
}

impl Default for EndpointSecurityConfig {
    fn default() -> Self {
        Self {
            public_health: true,
            expose_health_details: true,
            public_detailed_health: false,
            public_metrics: false,
            expose_sensitive_info: false,
        }
    }
}

/// Get appropriate endpoint security based on environment
pub fn get_endpoint_security_for_env(
    env_type: &EnvironmentType,
    config: Option<EndpointSecurityConfig>,
) -> EndpointSecurityConfig {
    // If explicit config is provided, use it
    if let Some(config) = config {
        return config;
    }

    // Otherwise use environment-based defaults
    match env_type {
        EnvironmentType::Development => EndpointSecurityConfig {
            public_health: true,
            expose_health_details: true,
            public_detailed_health: true,
            public_metrics: true,
            expose_sensitive_info: true,
        },
        EnvironmentType::Testing => EndpointSecurityConfig {
            public_health: true,
            expose_health_details: true,
            public_detailed_health: true,
            public_metrics: true,
            expose_sensitive_info: false,
        },
        EnvironmentType::Staging => EndpointSecurityConfig {
            public_health: true,
            expose_health_details: true,
            public_detailed_health: false,
            public_metrics: false,
            expose_sensitive_info: false,
        },
        EnvironmentType::Production => EndpointSecurityConfig {
            public_health: true,
            expose_health_details: true,
            public_detailed_health: false,
            public_metrics: false,
            expose_sensitive_info: false,
        },
    }
}

fn default_health_details() -> bool {
    true
}

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Server configuration
    #[serde(default)]
    pub server: ServerConfig,

    /// API configuration for upstream services
    #[serde(default)]
    pub api: ApiConfig,

    /// Authentication configuration
    #[serde(default)]
    pub auth: AuthConfig,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Reliability configuration
    #[serde(default)]
    pub reliability: ReliabilityConfig,

    /// OpenAPI configuration
    #[serde(default)]
    pub openapi: OpenApiConfig,

    /// Cache configuration
    #[serde(default)]
    pub cache: CacheConfig,

    /// Database configuration
    #[serde(default)]
    pub database: DatabaseConfig,

    /// Environment type (development, testing, staging, production)
    #[serde(default)]
    pub environment: EnvironmentType,

    /// Endpoint security configuration
    #[serde(default)]
    pub endpoint_security: EndpointSecurityConfig,
}

/// Application metadata configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub name: String,
    pub version: String,
    pub log_level: String,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        // Use the load_config function which implements the layered configuration approach
        load_config()
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    pub fn cache_ttl(&self) -> Duration {
        Duration::from_secs(self.cache.ttl_seconds)
    }

    /// Get the full Petstore API URL
    pub fn petstore_api_url(&self) -> String {
        self.api.petstore_url.trim_end_matches('/').to_string()
    }

    /// Get the OpenAPI spec file path
    pub fn openapi_spec_path(&self) -> String {
        // Hardcoded directory + filename from config
        format!("config/swagger/{}", self.openapi.spec_file)
    }

    /// Get the OpenAPI spec URL for Swagger UI
    pub fn openapi_spec_url(&self) -> String {
        // URL is derived from the filename, and includes the /actuator prefix
        // since docs routes are mounted under /actuator
        format!("/actuator/docs/{}", self.openapi.spec_file)
    }
}

/// Load configuration from files and environment variables
pub fn load_config() -> Result<AppConfig, ConfigError> {
    // Load .env file for secrets and overrides
    let _ = dotenv();

    // Determine the configuration directory
    let config_dir = env::var("CONFIG_DIR").unwrap_or_else(|_| "./config".to_string());

    // Determine the environment (development, production, etc.)
    let environment = env::var("RUN_ENV").unwrap_or_else(|_| "development".to_string());

    info!("Loading configuration for environment: {}", environment);

    // Build configuration with the following priority (highest to lowest):
    // 1. Environment variables (for secrets and CI/CD overrides)
    // 2. Environment-specific local overrides (local-{env}.yaml - not in version control)
    // 3. Environment-specific config ({env}.yaml)
    // 4. Local overrides (local.yaml - not in version control)
    // 5. Default config (default.yaml)
    let config = Config::builder()
        // 5. Start with default settings
        .add_source(File::from(Path::new(&config_dir).join("default.yaml")).required(false))
        // 4. Add local settings (not in version control)
        .add_source(File::from(Path::new(&config_dir).join("local.yaml")).required(false))
        // 3. Add environment-specific settings
        .add_source(
            File::from(Path::new(&config_dir).join(format!("{}.yaml", environment)))
                .required(false),
        )
        // 2. Add environment-specific local overrides (not in version control)
        .add_source(
            File::from(Path::new(&config_dir).join(format!("local-{}.yaml", environment)))
                .required(false),
        )
        // 1. Add environment variables (highest priority, for secrets and CI/CD)
        .add_source(Environment::with_prefix("SERVER").separator("_"))
        .add_source(Environment::with_prefix("API").separator("_"))
        .add_source(Environment::with_prefix("APP").separator("_"))
        .add_source(Environment::with_prefix("CACHE").separator("_"))
        .add_source(Environment::with_prefix("AUTH").separator("_"))
        .add_source(Environment::with_prefix("RELIABILITY").separator("_"))
        // Add specific environment variables for Entra ID auth
        .add_source(Environment::with_prefix("RUST_BACKEND").separator("_"))
        // Add legacy environment variables
        .add_source(Environment::default().try_parsing(true))
        // Build the config
        .build()?;

    // Deserialize the config into our AppConfig struct
    let mut app_config: AppConfig = config.try_deserialize()?;

    // Validate critical configuration values
    if app_config.auth.enabled {
        // Check roles configuration - panic if no roles are configured
        if app_config.auth.entra.admin_roles.is_empty() {
            panic!("No admin roles configured. Please specify admin_roles in configuration.");
        }

        if app_config.auth.entra.read_only_roles.is_empty() {
            panic!(
                "No read-only roles configured. Please specify read_only_roles in configuration."
            );
        }

        if app_config.auth.entra.full_access_roles.is_empty() {
            panic!(
                "No full access roles configured. Please specify full_access_roles in configuration."
            );
        }
    }

    // Manually set Entra ID configuration from environment variables if they exist
    // This ensures the environment variables are properly mapped to the configuration
    if let Ok(tenant_id) = env::var(constants::auth::env_vars::TENANT_ID) {
        if !tenant_id.is_empty() {
            app_config.auth.entra.tenant_id = tenant_id;
        }
    }

    if let Ok(client_id) = env::var(constants::auth::env_vars::CLIENT_ID) {
        if !client_id.is_empty() {
            app_config.auth.entra.client_id = client_id;
        }
    }

    if let Ok(audience) = env::var(constants::auth::env_vars::AUDIENCE) {
        if !audience.is_empty() {
            app_config.auth.entra.audience = audience;
        }
    }

    if let Ok(scope) = env::var(constants::auth::env_vars::SCOPE) {
        if !scope.is_empty() {
            app_config.auth.entra.scope = scope;
        }
    }

    if let Ok(token_url) = env::var(constants::auth::env_vars::TOKEN_URL) {
        if !token_url.is_empty() {
            app_config.auth.entra.token_url = token_url;
        }
    }

    if let Ok(debug_auth) = env::var(constants::auth::env_vars::DEBUG_AUTH) {
        if !debug_auth.is_empty() {
            app_config.auth.debug = debug_auth.parse().unwrap_or(false);
        }
    }

    Ok(app_config)
}

fn default_reconnect_interval() -> u64 {
    30
}
