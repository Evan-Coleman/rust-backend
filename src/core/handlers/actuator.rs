use axum::{Json, extract::State};
use std::sync::Arc;
use tracing::{debug, info};

use crate::core::router::AppState;
use crate::models::{ActuatorEntry, InfoResponse};

/// Handler for the info endpoint
///
/// Returns basic information about the application, like version,
/// build info, etc. This is useful for verifying what version
/// of the application is running.
pub async fn info(State(state): State<Arc<AppState>>) -> Json<InfoResponse> {
    info!("📊 Getting application info");

    let mut entries = Vec::new();

    // Add application info
    entries.push(ActuatorEntry {
        name: "name".to_string(),
        value: env!("CARGO_PKG_NAME").to_string(),
    });

    entries.push(ActuatorEntry {
        name: "version".to_string(),
        value: env!("CARGO_PKG_VERSION").to_string(),
    });

    entries.push(ActuatorEntry {
        name: "description".to_string(),
        value: env!("CARGO_PKG_DESCRIPTION").to_string(),
    });

    entries.push(ActuatorEntry {
        name: "authors".to_string(),
        value: env!("CARGO_PKG_AUTHORS").to_string(),
    });

    // Add build information
    entries.push(ActuatorEntry {
        name: "build_profile".to_string(),
        value: if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
        .to_string(),
    });

    // Add environment information
    entries.push(ActuatorEntry {
        name: "environment".to_string(),
        value: state.config.environment.to_string(),
    });

    entries.push(ActuatorEntry {
        name: "rust_version".to_string(),
        value: env!("CARGO_PKG_RUST_VERSION").to_string(),
    });

    Json(InfoResponse {
        status: "UP".to_string(),
        entries,
    })
}
