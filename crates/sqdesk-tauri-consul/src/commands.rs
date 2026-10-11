use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use sqdesk_core::agent_kv::{
    KvDeleteOptions, KvDeleteResponse, KvGetResponse, KvListPrefixResponse, KvPutOptions, KvPutResponse, KvValue,
};
use sqdesk_core::connection::AppState;

async fn ensure_connection_writable(state: &Arc<AppState>, connection_id: &str, action: &str) -> Result<(), String> {
    if let Some(name) = sqdesk_core::query::connection_readonly_name(state, connection_id).await {
        return Err(format!(
            "Read-only mode: connection '{}' has read-only protection enabled. {} blocked.",
            name, action
        ));
    }
    Ok(())
}

#[tauri::command]
pub async fn consul_capabilities(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
) -> Result<sqdesk_core::consul::ConsulCapabilities, String> {
    sqdesk_core::consul::consul_capabilities_core(&state, &connection_id).await
}

#[tauri::command]
pub async fn consul_list_prefix(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    prefix: String,
    limit: usize,
    continuation: Option<String>,
) -> Result<KvListPrefixResponse, String> {
    sqdesk_core::consul::consul_list_prefix_core(&state, &connection_id, &prefix, limit, continuation.as_deref()).await
}

#[tauri::command]
pub async fn consul_list_recursive(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    prefix: String,
    max_entries: usize,
    max_value_bytes: usize,
) -> Result<sqdesk_core::consul::ConsulRecursiveListResponse, String> {
    sqdesk_core::consul::consul_list_recursive_core(&state, &connection_id, &prefix, max_entries, max_value_bytes).await
}

#[tauri::command]
pub async fn consul_search(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::ConsulSearchRequest,
) -> Result<sqdesk_core::consul::ConsulSearchResponse, String> {
    sqdesk_core::consul::consul_search_core(&state, &connection_id, request).await
}

#[tauri::command]
pub async fn consul_search_progress(
    connection_id: String,
    request_id: String,
    scope: sqdesk_core::consul::ConsulScope,
    generation: u64,
) -> Result<sqdesk_core::consul::ConsulSearchProgress, String> {
    Ok(sqdesk_core::consul::consul_search_progress_core(&connection_id, &scope, generation, &request_id))
}

#[tauri::command]
pub async fn consul_cancel_search(
    connection_id: String,
    request_id: String,
    scope: sqdesk_core::consul::ConsulScope,
    generation: u64,
) -> Result<bool, String> {
    Ok(sqdesk_core::consul::consul_cancel_search_core(&connection_id, &scope, generation, &request_id))
}

#[tauri::command]
pub async fn consul_txn(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::ConsulTxnRequest,
) -> Result<sqdesk_core::consul::ConsulTxnResult, String> {
    ensure_connection_writable(&state, &connection_id, "Execute Consul transaction").await?;
    sqdesk_core::consul::consul_txn_core(&state, &connection_id, request).await
}

#[tauri::command]
pub async fn consul_rename_key(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    source: String,
    target: String,
    expected_modify_index: sqdesk_core::agent_kv::KvInt64,
    copy: bool,
) -> Result<sqdesk_core::consul::ConsulTxnResult, String> {
    sqdesk_core::consul::consul_rename_key_core(&state, &connection_id, &source, &target, expected_modify_index, copy)
        .await
}

#[tauri::command]
pub async fn consul_blocking_query(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::ConsulBlockingRequest,
) -> Result<sqdesk_core::consul::ConsulBlockingResponse, String> {
    sqdesk_core::consul::consul_blocking_query_core(&state, &connection_id, request).await
}

#[tauri::command]
pub async fn consul_domain_watch(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::ConsulDomainWatchRequest,
) -> Result<sqdesk_core::consul::ConsulDomainWatchResponse, String> {
    sqdesk_core::consul::consul_domain_watch_core(&state, &connection_id, request).await
}

#[tauri::command]
pub async fn consul_cancel_blocking(
    connection_id: String,
    scope: sqdesk_core::consul::ConsulScope,
    generation: u64,
    operation_id: String,
) -> Result<bool, String> {
    Ok(sqdesk_core::consul::consul_cancel_blocking_core(&connection_id, &scope, generation, &operation_id))
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConsulWatchEvent {
    connection_id: String,
    operation_id: String,
    generation: u64,
    result: Option<sqdesk_core::consul::ConsulBlockingResponse>,
    error: Option<String>,
}

#[tauri::command]
pub async fn consul_watch_start(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::ConsulBlockingRequest,
) -> Result<String, String> {
    let operation_id = request.operation_id.clone();
    let event_operation_id = operation_id.clone();
    let generation = request.generation;
    let app_state = Arc::clone(state.inner());
    tauri::async_runtime::spawn(async move {
        let response = sqdesk_core::consul::consul_blocking_query_core(&app_state, &connection_id, request).await;
        let event = match response {
            Ok(result) => ConsulWatchEvent {
                connection_id,
                operation_id: event_operation_id,
                generation,
                result: Some(result),
                error: None,
            },
            Err(error) => ConsulWatchEvent {
                connection_id,
                operation_id: event_operation_id,
                generation,
                result: None,
                error: Some(error),
            },
        };
        let _ = app.emit("consul-watch", event);
    });
    Ok(operation_id)
}

#[tauri::command]
pub async fn consul_export_bundle(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::ConsulExportRequest,
) -> Result<sqdesk_core::consul::ConsulKvBundle, String> {
    sqdesk_core::consul::consul_export_bundle_core(&state, &connection_id, request).await
}

#[tauri::command]
pub async fn consul_import_preview(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::ConsulImportRequest,
) -> Result<sqdesk_core::consul::ConsulImportPreview, String> {
    sqdesk_core::consul::consul_import_preview_core(&state, &connection_id, request).await
}

#[tauri::command]
pub async fn consul_import_execute(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::ConsulImportRequest,
) -> Result<sqdesk_core::consul::ConsulImportReport, String> {
    ensure_connection_writable(&state, &connection_id, "Import Consul KV bundle").await?;
    sqdesk_core::consul::consul_import_execute_core(&state, &connection_id, request).await
}

#[tauri::command]
pub async fn consul_delete_prefix_preview(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    prefix: String,
) -> Result<sqdesk_core::consul::ConsulDeletePrefixPreview, String> {
    sqdesk_core::consul::consul_delete_prefix_preview_core(&state, &connection_id, &prefix).await
}

#[tauri::command]
pub async fn consul_delete_prefix_execute(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::ConsulDeletePrefixRequest,
) -> Result<sqdesk_core::consul::ConsulDeletePrefixReport, String> {
    ensure_connection_writable(&state, &connection_id, "Delete Consul KV prefix").await?;
    sqdesk_core::consul::consul_delete_prefix_execute_core(&state, &connection_id, request).await
}

#[tauri::command]
pub async fn consul_get(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    key: String,
) -> Result<KvGetResponse, String> {
    sqdesk_core::consul::consul_get_core(&state, &connection_id, &key).await
}

#[tauri::command]
pub async fn consul_put(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    key: String,
    value: KvValue,
    options: Option<KvPutOptions>,
) -> Result<KvPutResponse, String> {
    ensure_connection_writable(&state, &connection_id, "Put Consul KV key").await?;
    sqdesk_core::consul::consul_put_core(&state, &connection_id, &key, value, options.unwrap_or_default()).await
}

#[tauri::command]
pub async fn consul_delete(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    key: String,
    options: Option<KvDeleteOptions>,
) -> Result<KvDeleteResponse, String> {
    ensure_connection_writable(&state, &connection_id, "Delete Consul KV key").await?;
    sqdesk_core::consul::consul_delete_core(&state, &connection_id, &key, options.unwrap_or_default()).await
}

#[tauri::command]
pub async fn consul_prepared_query_list(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
) -> Result<Vec<sqdesk_core::consul::ConsulPreparedQuery>, String> {
    sqdesk_core::consul::consul_prepared_query_list_core(&state, &connection_id).await
}

#[tauri::command]
pub async fn consul_prepared_query_read(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    id: String,
) -> Result<sqdesk_core::consul::ConsulPreparedQuery, String> {
    sqdesk_core::consul::consul_prepared_query_read_core(&state, &connection_id, &id).await
}

#[tauri::command]
pub async fn consul_prepared_query_create(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    input: sqdesk_core::consul::ConsulPreparedQueryInput,
) -> Result<String, String> {
    ensure_connection_writable(&state, &connection_id, "Create Consul prepared query").await?;
    sqdesk_core::consul::consul_prepared_query_create_core(&state, &connection_id, input).await
}

#[tauri::command]
pub async fn consul_prepared_query_update(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    id: String,
    input: sqdesk_core::consul::ConsulPreparedQueryInput,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Update Consul prepared query").await?;
    sqdesk_core::consul::consul_prepared_query_update_core(&state, &connection_id, &id, input).await
}

#[tauri::command]
pub async fn consul_prepared_query_delete(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    id: String,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Delete Consul prepared query").await?;
    sqdesk_core::consul::consul_prepared_query_delete_core(&state, &connection_id, &id).await
}

#[tauri::command]
pub async fn consul_prepared_query_execute(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::ConsulPreparedQueryExecuteRequest,
) -> Result<sqdesk_core::consul::ConsulPreparedQueryExecuteResponse, String> {
    sqdesk_core::consul::consul_prepared_query_execute_core(&state, &connection_id, request).await
}

#[tauri::command]
pub async fn consul_prepared_query_explain(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    query: String,
) -> Result<serde_json::Value, String> {
    sqdesk_core::consul::consul_prepared_query_explain_core(&state, &connection_id, &query).await
}

#[tauri::command]
pub async fn consul_event_list(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    name: Option<String>,
) -> Result<Vec<sqdesk_core::consul::ConsulEvent>, String> {
    sqdesk_core::consul::consul_event_list_core(&state, &connection_id, name.as_deref()).await
}

#[tauri::command]
pub async fn consul_event_fire(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::ConsulEventFireRequest,
) -> Result<sqdesk_core::consul::ConsulEvent, String> {
    ensure_connection_writable(&state, &connection_id, "Fire Consul event").await?;
    sqdesk_core::consul::consul_event_fire_core(&state, &connection_id, request).await
}

#[tauri::command]
pub async fn consul_coordinate_nodes(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
) -> Result<Vec<sqdesk_core::consul::ConsulCoordinate>, String> {
    sqdesk_core::consul::consul_coordinate_nodes_core(&state, &connection_id).await
}

#[tauri::command]
pub async fn consul_operator_read(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    kind: sqdesk_core::consul::ConsulOperatorReadKind,
) -> Result<sqdesk_core::consul::ConsulOperatorDocument, String> {
    sqdesk_core::consul::consul_operator_read_core(&state, &connection_id, kind).await
}

#[tauri::command]
pub async fn consul_snapshot_generate(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
) -> Result<sqdesk_core::consul::ConsulSnapshot, String> {
    sqdesk_core::consul::consul_snapshot_generate_core(&state, &connection_id).await
}

#[tauri::command]
pub async fn consul_snapshot_restore(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::ConsulSnapshotRestoreRequest,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Restore Consul snapshot").await?;
    sqdesk_core::consul::consul_snapshot_restore_core(&state, &connection_id, request).await
}

#[tauri::command]
pub async fn consul_autopilot_update(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    update: sqdesk_core::consul::ConsulAutopilotUpdate,
    confirmation: String,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Update Consul Autopilot").await?;
    sqdesk_core::consul::consul_autopilot_update_core(&state, &connection_id, update, &confirmation).await
}

#[tauri::command]
pub async fn consul_raft_transfer(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::ConsulRaftWriteRequest,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Transfer Consul Raft leadership").await?;
    sqdesk_core::consul::consul_raft_transfer_core(&state, &connection_id, request).await
}

#[tauri::command]
pub async fn consul_raft_remove(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::ConsulRaftWriteRequest,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Remove Consul Raft peer").await?;
    sqdesk_core::consul::consul_raft_remove_core(&state, &connection_id, request).await
}

#[tauri::command]
pub async fn consul_keyring_write(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::ConsulKeyringWriteRequest,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Modify Consul gossip keyring").await?;
    sqdesk_core::consul::consul_keyring_write_core(&state, &connection_id, request).await
}

#[tauri::command]
pub async fn consul_license_write(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::ConsulLicenseWriteRequest,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Update Consul license").await?;
    sqdesk_core::consul::consul_license_write_core(&state, &connection_id, request).await
}

#[tauri::command]
pub async fn consul_status_leader(state: State<'_, Arc<AppState>>, connection_id: String) -> Result<String, String> {
    sqdesk_core::consul::consul_status_leader_core(&state, &connection_id).await
}

#[tauri::command]
pub async fn consul_status_peers(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
) -> Result<Vec<String>, String> {
    sqdesk_core::consul::consul_status_peers_core(&state, &connection_id).await
}

#[tauri::command]
pub async fn consul_agent_self(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
) -> Result<sqdesk_core::consul::ConsulAgentIdentity, String> {
    sqdesk_core::consul::consul_agent_self_core(&state, &connection_id).await
}

#[tauri::command]
pub async fn consul_agent_members(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    wan: bool,
    segment: Option<String>,
) -> Result<Vec<sqdesk_core::consul::ConsulAgentMember>, String> {
    sqdesk_core::consul::consul_agent_members_core(&state, &connection_id, wan, segment.as_deref()).await
}

#[tauri::command]
pub async fn consul_agent_metrics(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
) -> Result<sqdesk_core::consul::ConsulAgentMetrics, String> {
    sqdesk_core::consul::consul_agent_metrics_core(&state, &connection_id).await
}

#[tauri::command]
pub async fn consul_catalog_datacenters(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
) -> Result<Vec<String>, String> {
    sqdesk_core::consul::consul_catalog_datacenters_core(&state, &connection_id).await
}

#[tauri::command]
pub async fn consul_catalog_nodes(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    options: sqdesk_core::consul::ConsulReadOptions,
) -> Result<sqdesk_core::consul::ConsulListResponse<Vec<sqdesk_core::consul::ConsulCatalogNode>>, String> {
    sqdesk_core::consul::consul_catalog_nodes_core(&state, &connection_id, options).await
}

#[tauri::command]
pub async fn consul_catalog_services(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    options: sqdesk_core::consul::ConsulReadOptions,
) -> Result<sqdesk_core::consul::ConsulListResponse<std::collections::BTreeMap<String, Vec<String>>>, String> {
    sqdesk_core::consul::consul_catalog_services_core(&state, &connection_id, options).await
}

#[tauri::command]
pub async fn consul_catalog_service_nodes(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    service: String,
    options: sqdesk_core::consul::ConsulReadOptions,
) -> Result<sqdesk_core::consul::ConsulListResponse<Vec<sqdesk_core::consul::ConsulCatalogServiceNode>>, String> {
    sqdesk_core::consul::consul_catalog_service_nodes_core(&state, &connection_id, &service, options).await
}

#[tauri::command]
pub async fn consul_catalog_node_services(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    node: String,
    options: sqdesk_core::consul::ConsulReadOptions,
) -> Result<sqdesk_core::consul::ConsulListResponse<sqdesk_core::consul::ConsulNodeServices>, String> {
    sqdesk_core::consul::consul_catalog_node_services_core(&state, &connection_id, &node, options).await
}

#[tauri::command]
pub async fn consul_health_node(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    node: String,
    options: sqdesk_core::consul::ConsulReadOptions,
) -> Result<sqdesk_core::consul::ConsulListResponse<Vec<sqdesk_core::consul::ConsulHealthCheck>>, String> {
    sqdesk_core::consul::consul_health_node_core(&state, &connection_id, &node, options).await
}

#[tauri::command]
pub async fn consul_health_checks(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    service: String,
    options: sqdesk_core::consul::ConsulReadOptions,
) -> Result<sqdesk_core::consul::ConsulListResponse<Vec<sqdesk_core::consul::ConsulHealthCheck>>, String> {
    sqdesk_core::consul::consul_health_checks_core(&state, &connection_id, &service, options).await
}

#[tauri::command]
pub async fn consul_health_service(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    service: String,
    passing: Option<bool>,
    options: sqdesk_core::consul::ConsulReadOptions,
) -> Result<sqdesk_core::consul::ConsulListResponse<Vec<sqdesk_core::consul::ConsulServiceInstance>>, String> {
    sqdesk_core::consul::consul_health_service_core(&state, &connection_id, &service, passing, options).await
}

#[tauri::command]
pub async fn consul_health_state(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    health_state: String,
    options: sqdesk_core::consul::ConsulReadOptions,
) -> Result<sqdesk_core::consul::ConsulListResponse<Vec<sqdesk_core::consul::ConsulHealthCheck>>, String> {
    sqdesk_core::consul::consul_health_state_core(&state, &connection_id, &health_state, options).await
}

#[tauri::command]
pub async fn consul_agent_services(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
) -> Result<std::collections::BTreeMap<String, sqdesk_core::consul::ConsulAgentService>, String> {
    sqdesk_core::consul::consul_agent_services_core(&state, &connection_id).await
}

#[tauri::command]
pub async fn consul_agent_service(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    id: String,
) -> Result<sqdesk_core::consul::ConsulAgentService, String> {
    sqdesk_core::consul::consul_agent_service_core(&state, &connection_id, &id).await
}

#[tauri::command]
pub async fn consul_agent_checks(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
) -> Result<std::collections::BTreeMap<String, sqdesk_core::consul::ConsulHealthCheck>, String> {
    sqdesk_core::consul::consul_agent_checks_core(&state, &connection_id).await
}

#[tauri::command]
pub async fn consul_agent_register_service(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    registration: sqdesk_core::consul::ConsulAgentServiceRegistration,
) -> Result<sqdesk_core::consul::ConsulAgentWriteResult, String> {
    ensure_connection_writable(&state, &connection_id, "Register Consul Agent service").await?;
    sqdesk_core::consul::consul_agent_register_service_core(&state, &connection_id, registration).await
}

#[tauri::command]
pub async fn consul_agent_deregister_service(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    id: String,
) -> Result<sqdesk_core::consul::ConsulAgentWriteResult, String> {
    ensure_connection_writable(&state, &connection_id, "Deregister Consul Agent service").await?;
    sqdesk_core::consul::consul_agent_deregister_service_core(&state, &connection_id, &id).await
}

#[tauri::command]
pub async fn consul_agent_service_maintenance(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    id: String,
    enable: bool,
    reason: Option<String>,
) -> Result<sqdesk_core::consul::ConsulAgentWriteResult, String> {
    ensure_connection_writable(&state, &connection_id, "Change Consul Agent service maintenance").await?;
    sqdesk_core::consul::consul_agent_service_maintenance_core(&state, &connection_id, &id, enable, reason.as_deref())
        .await
}

#[tauri::command]
pub async fn consul_agent_register_check(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    registration: sqdesk_core::consul::ConsulAgentCheckRegistration,
) -> Result<sqdesk_core::consul::ConsulAgentWriteResult, String> {
    ensure_connection_writable(&state, &connection_id, "Register Consul Agent check").await?;
    sqdesk_core::consul::consul_agent_register_check_core(&state, &connection_id, registration).await
}

#[tauri::command]
pub async fn consul_agent_deregister_check(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    id: String,
) -> Result<sqdesk_core::consul::ConsulAgentWriteResult, String> {
    ensure_connection_writable(&state, &connection_id, "Deregister Consul Agent check").await?;
    sqdesk_core::consul::consul_agent_deregister_check_core(&state, &connection_id, &id).await
}

#[tauri::command]
pub async fn consul_agent_update_ttl(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    id: String,
    status: sqdesk_core::consul::ConsulCheckStatus,
    output: Option<String>,
) -> Result<sqdesk_core::consul::ConsulAgentWriteResult, String> {
    ensure_connection_writable(&state, &connection_id, "Update Consul Agent TTL check").await?;
    sqdesk_core::consul::consul_agent_update_ttl_core(&state, &connection_id, &id, status, output.as_deref()).await
}

#[tauri::command]
pub async fn consul_sessions(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    options: sqdesk_core::consul::ConsulReadOptions,
) -> Result<sqdesk_core::consul::ConsulListResponse<Vec<sqdesk_core::consul::ConsulSession>>, String> {
    sqdesk_core::consul::consul_sessions_core(&state, &connection_id, options).await
}

#[tauri::command]
pub async fn consul_node_sessions(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    node: String,
    options: sqdesk_core::consul::ConsulReadOptions,
) -> Result<sqdesk_core::consul::ConsulListResponse<Vec<sqdesk_core::consul::ConsulSession>>, String> {
    sqdesk_core::consul::consul_node_sessions_core(&state, &connection_id, &node, options).await
}

#[tauri::command]
pub async fn consul_session(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    id: String,
) -> Result<Option<sqdesk_core::consul::ConsulSession>, String> {
    sqdesk_core::consul::consul_session_core(&state, &connection_id, &id).await
}

#[tauri::command]
pub async fn consul_session_keys(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    id: String,
) -> Result<sqdesk_core::consul::ConsulSessionKeysResponse, String> {
    sqdesk_core::consul::consul_session_keys_core(&state, &connection_id, &id).await
}

#[tauri::command]
pub async fn consul_session_destroy_impact(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    id: String,
) -> Result<sqdesk_core::consul::ConsulSessionDestroyImpact, String> {
    sqdesk_core::consul::consul_session_destroy_impact_core(&state, &connection_id, &id).await
}

#[tauri::command]
pub async fn consul_create_session(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::ConsulSessionCreateRequest,
) -> Result<sqdesk_core::consul::ConsulSession, String> {
    ensure_connection_writable(&state, &connection_id, "Create Consul Session").await?;
    sqdesk_core::consul::consul_create_session_core(&state, &connection_id, request).await
}

#[tauri::command]
pub async fn consul_renew_session(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    id: String,
) -> Result<sqdesk_core::consul::ConsulSession, String> {
    ensure_connection_writable(&state, &connection_id, "Renew Consul Session").await?;
    sqdesk_core::consul::consul_renew_session_core(&state, &connection_id, &id).await
}

#[tauri::command]
pub async fn consul_destroy_session(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::ConsulSessionDestroyRequest,
) -> Result<bool, String> {
    ensure_connection_writable(&state, &connection_id, "Destroy Consul Session").await?;
    sqdesk_core::consul::consul_destroy_session_core(&state, &connection_id, request).await
}

#[tauri::command]
pub async fn consul_acquire_lock(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::ConsulLockRequest,
) -> Result<sqdesk_core::consul::ConsulLockResponse, String> {
    ensure_connection_writable(&state, &connection_id, "Acquire Consul KV lock").await?;
    sqdesk_core::consul::consul_acquire_lock_core(&state, &connection_id, request).await
}

#[tauri::command]
pub async fn consul_release_lock(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    key: String,
    session: String,
) -> Result<sqdesk_core::consul::ConsulLockResponse, String> {
    ensure_connection_writable(&state, &connection_id, "Release Consul KV lock").await?;
    sqdesk_core::consul::consul_release_lock_core(&state, &connection_id, &key, &session).await
}

#[tauri::command]
pub async fn consul_acl_list(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    kind: sqdesk_core::consul::ConsulAclKind,
) -> Result<sqdesk_core::consul::ConsulAclList, String> {
    sqdesk_core::consul::consul_acl_list_core(&state, &connection_id, kind).await
}
#[tauri::command]
pub async fn consul_acl_token_self(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
) -> Result<sqdesk_core::consul::ConsulAclToken, String> {
    sqdesk_core::consul::consul_acl_token_self_core(&state, &connection_id).await
}
#[tauri::command]
pub async fn consul_acl_token_clone(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    accessor_id: String,
    request: sqdesk_core::consul::ConsulAclTokenClone,
) -> Result<sqdesk_core::consul::ConsulAclToken, String> {
    ensure_connection_writable(&state, &connection_id, "Clone Consul ACL token").await?;
    sqdesk_core::consul::consul_acl_token_clone_core(&state, &connection_id, &accessor_id, request).await
}
#[tauri::command]
pub async fn consul_acl_get(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    kind: sqdesk_core::consul::ConsulAclKind,
    id: String,
) -> Result<sqdesk_core::consul::ConsulAclItem, String> {
    sqdesk_core::consul::consul_acl_get_core(&state, &connection_id, kind, &id).await
}
#[tauri::command]
pub async fn consul_acl_apply(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    id: Option<String>,
    value: sqdesk_core::consul::ConsulAclWrite,
) -> Result<sqdesk_core::consul::ConsulAclItem, String> {
    ensure_connection_writable(&state, &connection_id, "Write Consul ACL resource").await?;
    sqdesk_core::consul::consul_acl_apply_core(&state, &connection_id, id.as_deref(), value).await
}
#[tauri::command]
pub async fn consul_acl_references(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    kind: sqdesk_core::consul::ConsulAclKind,
    id: String,
) -> Result<sqdesk_core::consul::ConsulAclReferences, String> {
    sqdesk_core::consul::consul_acl_references_core(&state, &connection_id, kind, &id).await
}
#[tauri::command]
pub async fn consul_acl_delete(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    kind: sqdesk_core::consul::ConsulAclKind,
    id: String,
) -> Result<sqdesk_core::consul::ConsulAclReferences, String> {
    ensure_connection_writable(&state, &connection_id, "Delete Consul ACL resource").await?;
    sqdesk_core::consul::consul_acl_delete_core(&state, &connection_id, kind, &id).await
}

#[tauri::command]
pub async fn consul_enterprise_list(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    kind: sqdesk_core::consul::ConsulEnterpriseKind,
) -> Result<sqdesk_core::consul::ConsulEnterpriseList, String> {
    sqdesk_core::consul::consul_enterprise_list_core(&state, &connection_id, kind).await
}
#[tauri::command]
pub async fn consul_enterprise_get(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    kind: sqdesk_core::consul::ConsulEnterpriseKind,
    name: String,
) -> Result<sqdesk_core::consul::ConsulEnterpriseItem, String> {
    sqdesk_core::consul::consul_enterprise_get_core(&state, &connection_id, kind, &name).await
}
#[tauri::command]
pub async fn consul_enterprise_apply(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    existing_name: Option<String>,
    item: sqdesk_core::consul::ConsulEnterpriseWrite,
) -> Result<sqdesk_core::consul::ConsulEnterpriseItem, String> {
    ensure_connection_writable(&state, &connection_id, "Write Consul Enterprise scope").await?;
    sqdesk_core::consul::consul_enterprise_apply_core(&state, &connection_id, existing_name.as_deref(), item).await
}
#[tauri::command]
pub async fn consul_enterprise_impact(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    kind: sqdesk_core::consul::ConsulEnterpriseKind,
    name: String,
) -> Result<sqdesk_core::consul::ConsulScopeImpact, String> {
    sqdesk_core::consul::consul_enterprise_impact_core(&state, &connection_id, kind, &name).await
}
#[tauri::command]
pub async fn consul_enterprise_delete(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    kind: sqdesk_core::consul::ConsulEnterpriseKind,
    name: String,
) -> Result<sqdesk_core::consul::ConsulScopeImpact, String> {
    ensure_connection_writable(&state, &connection_id, "Delete Consul Enterprise scope").await?;
    sqdesk_core::consul::consul_enterprise_delete_core(&state, &connection_id, kind, &name).await
}

#[tauri::command]
pub async fn consul_mesh_config_list(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    kind: String,
) -> Result<Vec<sqdesk_core::consul::mesh::ConsulConfigEntry>, String> {
    sqdesk_core::consul::mesh::consul_mesh_config_list_core(&state, &connection_id, &kind).await
}
#[tauri::command]
pub async fn consul_mesh_config_get(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    kind: String,
    name: String,
) -> Result<sqdesk_core::consul::mesh::ConsulConfigEntry, String> {
    sqdesk_core::consul::mesh::consul_mesh_config_get_core(&state, &connection_id, &kind, &name).await
}
#[tauri::command]
pub async fn consul_mesh_config_apply(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::mesh::ConsulConfigEntryApply,
) -> Result<sqdesk_core::consul::mesh::ConsulConfigEntry, String> {
    ensure_connection_writable(&state, &connection_id, "Write Consul Service Mesh config entry").await?;
    sqdesk_core::consul::mesh::consul_mesh_config_apply_core(&state, &connection_id, request).await
}
#[tauri::command]
pub async fn consul_mesh_config_delete(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    kind: String,
    name: String,
    expected_modify_index: u64,
) -> Result<bool, String> {
    ensure_connection_writable(&state, &connection_id, "Delete Consul Service Mesh config entry").await?;
    sqdesk_core::consul::mesh::consul_mesh_config_delete_core(&state, &connection_id, &kind, &name, expected_modify_index)
        .await
}
#[tauri::command]
pub async fn consul_mesh_intentions_list(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
) -> Result<Vec<sqdesk_core::consul::mesh::ConsulIntention>, String> {
    sqdesk_core::consul::mesh::consul_mesh_intentions_list_core(&state, &connection_id).await
}
#[tauri::command]
pub async fn consul_mesh_intention_get(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    id: String,
) -> Result<sqdesk_core::consul::mesh::ConsulIntention, String> {
    sqdesk_core::consul::mesh::consul_mesh_intention_get_core(&state, &connection_id, &id).await
}
#[tauri::command]
pub async fn consul_mesh_intention_get_exact(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::mesh::ConsulIntentionExactRequest,
) -> Result<sqdesk_core::consul::mesh::ConsulIntention, String> {
    sqdesk_core::consul::mesh::consul_mesh_intention_get_exact_core(&state, &connection_id, request).await
}
#[tauri::command]
pub async fn consul_mesh_intention_upsert(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    item: sqdesk_core::consul::mesh::ConsulIntention,
) -> Result<sqdesk_core::consul::mesh::ConsulIntention, String> {
    ensure_connection_writable(&state, &connection_id, "Write Consul intention").await?;
    sqdesk_core::consul::mesh::consul_mesh_intention_upsert_core(&state, &connection_id, item).await
}
#[tauri::command]
pub async fn consul_mesh_intention_delete(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    id: String,
) -> Result<bool, String> {
    ensure_connection_writable(&state, &connection_id, "Delete Consul intention").await?;
    sqdesk_core::consul::mesh::consul_mesh_intention_delete_core(&state, &connection_id, &id).await
}
#[tauri::command]
pub async fn consul_mesh_intention_delete_exact(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::mesh::ConsulIntentionExactRequest,
) -> Result<bool, String> {
    ensure_connection_writable(&state, &connection_id, "Delete exact Consul intention").await?;
    sqdesk_core::consul::mesh::consul_mesh_intention_delete_exact_core(&state, &connection_id, request).await
}
#[tauri::command]
pub async fn consul_mesh_intention_match(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::mesh::ConsulIntentionMatchRequest,
) -> Result<Vec<sqdesk_core::consul::mesh::ConsulIntention>, String> {
    sqdesk_core::consul::mesh::consul_mesh_intention_match_core(&state, &connection_id, request).await
}
#[tauri::command]
pub async fn consul_mesh_intention_check(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::mesh::ConsulIntentionCheckRequest,
) -> Result<sqdesk_core::consul::mesh::ConsulIntentionCheckResponse, String> {
    sqdesk_core::consul::mesh::consul_mesh_intention_check_core(&state, &connection_id, request).await
}
#[tauri::command]
pub async fn consul_mesh_discovery_chain(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    service: String,
) -> Result<sqdesk_core::consul::mesh::ConsulDiscoveryChain, String> {
    sqdesk_core::consul::mesh::consul_mesh_discovery_chain_core(&state, &connection_id, &service).await
}
#[tauri::command]
pub async fn consul_mesh_peering_list(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
) -> Result<Vec<sqdesk_core::consul::mesh::ConsulPeering>, String> {
    sqdesk_core::consul::mesh::consul_mesh_peering_list_core(&state, &connection_id).await
}
#[tauri::command]
pub async fn consul_mesh_peering_get(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    name: String,
) -> Result<sqdesk_core::consul::mesh::ConsulPeering, String> {
    sqdesk_core::consul::mesh::consul_mesh_peering_get_core(&state, &connection_id, &name).await
}
#[tauri::command]
pub async fn consul_mesh_peering_generate_token(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::mesh::ConsulPeeringGenerateRequest,
) -> Result<sqdesk_core::consul::mesh::ConsulPeeringToken, String> {
    ensure_connection_writable(&state, &connection_id, "Generate Consul peering token").await?;
    sqdesk_core::consul::mesh::consul_mesh_peering_generate_token_core(&state, &connection_id, request).await
}
#[tauri::command]
pub async fn consul_mesh_peering_establish(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    request: sqdesk_core::consul::mesh::ConsulPeeringEstablishRequest,
) -> Result<sqdesk_core::consul::mesh::ConsulPeering, String> {
    ensure_connection_writable(&state, &connection_id, "Establish Consul peering").await?;
    sqdesk_core::consul::mesh::consul_mesh_peering_establish_core(&state, &connection_id, request).await
}
#[tauri::command]
pub async fn consul_mesh_peering_delete(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    name: String,
) -> Result<bool, String> {
    ensure_connection_writable(&state, &connection_id, "Delete Consul peering").await?;
    sqdesk_core::consul::mesh::consul_mesh_peering_delete_core(&state, &connection_id, &name).await
}
#[tauri::command]
pub async fn consul_mesh_exported_services_list(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
) -> Result<Vec<sqdesk_core::consul::mesh::ConsulExportedService>, String> {
    sqdesk_core::consul::mesh::consul_mesh_exported_services_list_core(&state, &connection_id).await
}
#[tauri::command]
pub async fn consul_mesh_exported_services_apply(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    name: String,
    expected_modify_index: u64,
    raw: serde_json::Value,
) -> Result<sqdesk_core::consul::mesh::ConsulConfigEntry, String> {
    ensure_connection_writable(&state, &connection_id, "Write Consul exported services").await?;
    sqdesk_core::consul::mesh::consul_mesh_exported_services_apply_core(
        &state,
        &connection_id,
        &name,
        expected_modify_index,
        raw,
    )
    .await
}
