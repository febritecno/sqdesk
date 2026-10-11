use std::sync::Arc;

use sqdesk_core::storage::{MigrationPreflight, MigrationReport};
use tauri::State;

use sqdesk_core::connection::AppState;

#[tauri::command]
pub async fn migration_status(
    state: State<'_, Arc<AppState>>,
    retry: Option<bool>,
) -> Result<MigrationPreflight, String> {
    if retry.unwrap_or(false) {
        state.storage.retry_data_migration_inspection().await
    } else {
        state.storage.inspect_data_migration().await
    }
}

#[tauri::command]
pub async fn migration_start(
    state: State<'_, Arc<AppState>>,
    gate: State<'_, Arc<crate::migration_gate::MigrationGate>>,
) -> Result<MigrationReport, String> {
    gate.set_ready(false);
    let result = state.storage.start_data_migration().await;
    let ready = state.storage.inspect_data_migration().await.map(|status| status.is_ready()).unwrap_or(false);
    gate.set_ready(ready);
    result
}

#[tauri::command]
pub async fn migration_retry(
    state: State<'_, Arc<AppState>>,
    gate: State<'_, Arc<crate::migration_gate::MigrationGate>>,
) -> Result<MigrationReport, String> {
    migration_start(state, gate).await
}

#[tauri::command]
pub async fn migration_cleanup_backups(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.storage.cleanup_migration_backups().await
}
