use std::sync::Arc;
use tauri::State;

use crate::commands::connection::AppState;

#[tauri::command]
pub fn prepare_data_compare(
    options: sqdesk_core::data_compare::DataComparePreparationOptions,
) -> Result<sqdesk_core::data_compare::DataComparePreparation, String> {
    sqdesk_core::data_compare::prepare_data_compare(options)
}

#[tauri::command]
pub async fn prepare_data_compare_from_tables(
    state: State<'_, Arc<AppState>>,
    options: sqdesk_core::data_compare::DataCompareFromTablesOptions,
) -> Result<sqdesk_core::data_compare::DataCompareFromTablesPreparation, String> {
    sqdesk_core::data_compare::prepare_data_compare_from_tables(&state, options).await
}

#[tauri::command]
pub async fn prepare_data_compare_missing_target(
    state: State<'_, Arc<AppState>>,
    options: sqdesk_core::data_compare::DataCompareMissingTargetOptions,
) -> Result<sqdesk_core::data_compare::DataCompareFromTablesPreparation, String> {
    sqdesk_core::data_compare::prepare_data_compare_missing_target(&state, options).await
}

#[tauri::command]
pub fn build_data_compare_sync_plan(
    options: sqdesk_core::data_compare::DataCompareSyncPlanOptions,
) -> sqdesk_core::data_compare::DataCompareSyncPlan {
    sqdesk_core::data_compare::build_data_compare_sync_plan(options)
}
