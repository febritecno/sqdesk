use std::future::Future;
use std::sync::Arc;
use tauri::State;

use crate::commands::connection::{ensure_connection_writable, AppState};
use sqdesk_core::db::mongo_driver::MongoDocumentResult;

/// The MCP bridge that used to authorise these calls is gone, so a request
/// that still claims to come from it is refused rather than run unchecked.
fn reject_mcp_request(mcp_request: Option<bool>) -> Result<(), String> {
    if mcp_request == Some(true) {
        return Err("MCP access is not available in SQDesk.".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn mongo_parse_shell_command(source: String) -> Result<sqdesk_core::mongo_shell::MongoCommand, String> {
    sqdesk_core::mongo_shell::parse(&source)
}

async fn run_cancellable<T, F>(state: &Arc<AppState>, execution_id: Option<String>, future: F) -> Result<T, String>
where
    F: Future<Output = Result<T, String>>,
{
    let registered_query =
        execution_id.as_ref().filter(|id| !id.trim().is_empty()).map(|id| state.running_queries.register(id.clone()));
    if let Some(query) = registered_query.as_ref() {
        let token = query.token();
        tokio::select! {
            biased;
            _ = token.cancelled() => Err(sqdesk_core::query::canceled_error()),
            result = future => result,
        }
    } else {
        future.await
    }
}

#[tauri::command]
pub async fn mongo_list_databases(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
) -> Result<Vec<String>, String> {
    sqdesk_core::mongo_ops::mongo_list_databases_core(&state, &connection_id).await
}

#[tauri::command]
pub async fn mongo_list_collections(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
) -> Result<Vec<sqdesk_core::document_ops::CollectionInfo>, String> {
    sqdesk_core::mongo_ops::mongo_list_collections_core(&state, &connection_id, &database).await
}

#[tauri::command]
pub async fn mongo_create_database(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Create database").await?;
    sqdesk_core::mongo_ops::mongo_create_database_core(&state, &connection_id, &database).await
}

#[tauri::command]
pub async fn mongo_drop_database(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Drop database").await?;
    sqdesk_core::mongo_ops::mongo_drop_database_core(&state, &connection_id, &database).await
}

#[tauri::command]
pub async fn mongo_drop_collection(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    collection: String,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Drop collection").await?;
    sqdesk_core::mongo_ops::mongo_drop_collection_core(&state, &connection_id, &database, &collection).await
}

#[tauri::command]
pub async fn mongo_rename_collection(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    collection: String,
    new_name: String,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Rename collection").await?;
    sqdesk_core::mongo_ops::mongo_rename_collection_core(&state, &connection_id, &database, &collection, &new_name)
        .await
}

#[tauri::command]
pub async fn mongo_clone_collection(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    source_collection: String,
    target_collection: String,
) -> Result<sqdesk_core::db::mongo_driver::MongoCloneCollectionResult, String> {
    ensure_connection_writable(&state, &connection_id, "Clone collection").await?;
    sqdesk_core::mongo_ops::mongo_clone_collection_core(
        &state,
        &connection_id,
        &database,
        &source_collection,
        &target_collection,
    )
    .await
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn mongo_find_documents(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    collection: String,
    skip: u64,
    limit: i64,
    filter: Option<String>,
    projection: Option<String>,
    sort: Option<String>,
    collation: Option<String>,
    execution_id: Option<String>,
    mcp_request: Option<bool>,
) -> Result<MongoDocumentResult, String> {
    reject_mcp_request(mcp_request)?;
    crate::commands::document_cmd::document_find_documents(
        state,
        connection_id,
        database,
        collection,
        skip,
        limit,
        filter,
        projection,
        sort,
        collation,
        None,
        None,
        execution_id,
    )
    .await
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn mongo_explain_find(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    collection: String,
    skip: u64,
    limit: i64,
    filter: Option<String>,
    projection: Option<String>,
    sort: Option<String>,
    collation: Option<String>,
    verbosity: Option<String>,
    execution_id: Option<String>,
    mcp_request: Option<bool>,
) -> Result<serde_json::Value, String> {
    let app = state.inner().clone();
    reject_mcp_request(mcp_request)?;
    run_cancellable(
        &app,
        execution_id,
        sqdesk_core::mongo_ops::mongo_explain_find_core(
            &app,
            &connection_id,
            &database,
            &collection,
            skip,
            limit,
            filter.as_deref(),
            projection.as_deref(),
            sort.as_deref(),
            collation.as_deref(),
            verbosity.as_deref().unwrap_or("queryPlanner"),
        ),
    )
    .await
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn mongo_find_one(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    collection: String,
    filter: Option<String>,
    projection: Option<String>,
    options: Option<String>,
    execution_id: Option<String>,
    mcp_request: Option<bool>,
) -> Result<MongoDocumentResult, String> {
    let app = state.inner().clone();
    reject_mcp_request(mcp_request)?;
    run_cancellable(
        &app,
        execution_id,
        sqdesk_core::mongo_ops::mongo_find_one_core(
            &app,
            &connection_id,
            &database,
            &collection,
            filter.as_deref(),
            projection.as_deref(),
            options.as_deref(),
        ),
    )
    .await
}

#[tauri::command]
pub async fn mongo_count_documents(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    collection: String,
    filter: Option<String>,
    mode: Option<String>,
    execution_id: Option<String>,
    mcp_request: Option<bool>,
) -> Result<u64, String> {
    let app = state.inner().clone();
    reject_mcp_request(mcp_request)?;
    crate::commands::document_cmd::run_cancellable(
        &app,
        execution_id,
        sqdesk_core::mongo_ops::mongo_count_documents_core(
            &app,
            &connection_id,
            &database,
            &collection,
            filter.as_deref(),
            mode.as_deref(),
        ),
    )
    .await
}

#[tauri::command]
pub async fn mongo_server_version(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    execution_id: Option<String>,
    mcp_request: Option<bool>,
) -> Result<String, String> {
    let app = state.inner().clone();
    reject_mcp_request(mcp_request)?;
    run_cancellable(
        &app,
        execution_id,
        sqdesk_core::mongo_ops::mongo_server_version_core(&app, &connection_id, &database),
    )
    .await
}

#[tauri::command]
pub async fn mongo_collection_stats(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    collection: String,
    scale: Option<serde_json::Number>,
    execution_id: Option<String>,
    mcp_request: Option<bool>,
) -> Result<sqdesk_core::db::mongo_driver::MongoCollectionStatsResult, String> {
    let app = state.inner().clone();
    reject_mcp_request(mcp_request)?;
    run_cancellable(
        &app,
        execution_id,
        sqdesk_core::mongo_ops::mongo_collection_stats_core(&app, &connection_id, &database, &collection, scale),
    )
    .await
}

#[tauri::command]
pub async fn mongo_aggregate_documents(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    collection: String,
    pipeline_json: String,
    max_rows: Option<usize>,
    options_json: Option<String>,
    execution_id: Option<String>,
    mcp_request: Option<bool>,
) -> Result<MongoDocumentResult, String> {
    let app = state.inner().clone();
    reject_mcp_request(mcp_request)?;
    run_cancellable(
        &app,
        execution_id,
        sqdesk_core::mongo_ops::mongo_aggregate_documents_core(
            &app,
            &connection_id,
            &database,
            &collection,
            &pipeline_json,
            max_rows,
            options_json.as_deref(),
        ),
    )
    .await
}

#[tauri::command]
pub async fn mongo_distinct(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    collection: String,
    field: String,
    filter: Option<String>,
    execution_id: Option<String>,
    mcp_request: Option<bool>,
) -> Result<MongoDocumentResult, String> {
    let app = state.inner().clone();
    reject_mcp_request(mcp_request)?;
    run_cancellable(
        &app,
        execution_id,
        sqdesk_core::mongo_ops::mongo_distinct_core(
            &app,
            &connection_id,
            &database,
            &collection,
            &field,
            filter.as_deref(),
        ),
    )
    .await
}

/// Read-only listing of a collection's indexes with their full MongoDB options.
#[tauri::command]
pub async fn mongo_list_index_specs(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    collection: String,
) -> Result<Vec<sqdesk_core::db::mongo_driver::MongoIndexSpec>, String> {
    sqdesk_core::mongo_ops::mongo_list_index_specs_core(&state, &connection_id, &database, &collection).await
}

#[tauri::command]
pub async fn mongo_create_index(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    collection: String,
    keys_json: String,
    options_json: Option<String>,
    mcp_request: Option<bool>,
) -> Result<serde_json::Value, String> {
    reject_mcp_request(mcp_request)?;
    ensure_connection_writable(&state, &connection_id, "Create index").await?;
    let name = sqdesk_core::mongo_ops::mongo_create_index_core(
        &state,
        &connection_id,
        &database,
        &collection,
        &keys_json,
        options_json.as_deref(),
    )
    .await?;
    Ok(serde_json::json!({ "name": name }))
}

#[tauri::command]
pub async fn mongo_create_user(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    user_json: String,
    write_concern_json: Option<String>,
    mcp_request: Option<bool>,
) -> Result<serde_json::Value, String> {
    reject_mcp_request(mcp_request)?;
    ensure_connection_writable(&state, &connection_id, "Create user").await?;
    let affected_rows = sqdesk_core::mongo_ops::mongo_create_user_core(
        &state,
        &connection_id,
        &database,
        &user_json,
        write_concern_json.as_deref(),
    )
    .await?;
    Ok(serde_json::json!({ "affected_rows": affected_rows }))
}

#[tauri::command]
pub async fn mongo_run_command(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    command_json: String,
    execution_id: Option<String>,
    mcp_request: Option<bool>,
) -> Result<MongoDocumentResult, String> {
    reject_mcp_request(mcp_request)?;
    ensure_connection_writable(&state, &connection_id, "Run MongoDB command").await?;
    let app = state.inner().clone();
    run_cancellable(
        &app,
        execution_id,
        sqdesk_core::mongo_ops::mongo_run_command_core(&app, &connection_id, &database, &command_json),
    )
    .await
}

#[tauri::command]
pub async fn mongo_drop_indexes(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    collection: String,
    indexes_json: Option<String>,
    single: bool,
    mcp_request: Option<bool>,
) -> Result<sqdesk_core::db::mongo_driver::MongoDropIndexesResult, String> {
    reject_mcp_request(mcp_request)?;
    ensure_connection_writable(&state, &connection_id, "Drop indexes").await?;
    sqdesk_core::mongo_ops::mongo_drop_indexes_core(
        &state,
        &connection_id,
        &database,
        &collection,
        indexes_json.as_deref(),
        single,
    )
    .await
}

#[tauri::command]
pub async fn mongo_insert_document(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    collection: String,
    doc_json: String,
    routing: Option<String>,
) -> Result<String, String> {
    crate::commands::document_cmd::document_insert_document(
        state,
        connection_id,
        database,
        collection,
        doc_json,
        routing,
        None,
    )
    .await
}

#[tauri::command]
pub async fn mongo_insert_documents(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    collection: String,
    docs_json: String,
    mcp_request: Option<bool>,
) -> Result<u64, String> {
    reject_mcp_request(mcp_request)?;
    ensure_connection_writable(&state, &connection_id, "Insert").await?;
    sqdesk_core::mongo_ops::mongo_insert_documents_core(&state, &connection_id, &database, &collection, &docs_json)
        .await
}

#[tauri::command]
pub async fn mongo_update_document(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    collection: String,
    id: String,
    doc_json: String,
    routing: Option<String>,
) -> Result<u64, String> {
    crate::commands::document_cmd::document_update_document(
        state,
        connection_id,
        database,
        collection,
        id,
        doc_json,
        routing,
    )
    .await
}

#[tauri::command]
pub async fn mongo_update_documents(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    collection: String,
    filter_json: String,
    update_json: String,
    many: bool,
    options_json: Option<String>,
    mcp_request: Option<bool>,
) -> Result<u64, String> {
    reject_mcp_request(mcp_request)?;
    ensure_connection_writable(&state, &connection_id, "Update").await?;
    sqdesk_core::mongo_ops::mongo_update_documents_core(
        &state,
        &connection_id,
        &database,
        &collection,
        &filter_json,
        &update_json,
        many,
        options_json.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn mongo_replace_document(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    collection: String,
    filter_json: String,
    replacement_json: String,
    options_json: Option<String>,
    mcp_request: Option<bool>,
) -> Result<u64, String> {
    reject_mcp_request(mcp_request)?;
    ensure_connection_writable(&state, &connection_id, "Replace").await?;
    sqdesk_core::mongo_ops::mongo_replace_document_core(
        &state,
        &connection_id,
        &database,
        &collection,
        &filter_json,
        &replacement_json,
        options_json.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn mongo_bulk_write(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    collection: String,
    operations_json: String,
    options_json: Option<String>,
    mcp_request: Option<bool>,
) -> Result<sqdesk_core::db::mongo_driver::MongoBulkWriteResult, String> {
    reject_mcp_request(mcp_request)?;
    ensure_connection_writable(&state, &connection_id, "BulkWrite").await?;
    sqdesk_core::mongo_ops::mongo_bulk_write_core(
        &state,
        &connection_id,
        &database,
        &collection,
        &operations_json,
        options_json.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn mongo_delete_document(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    collection: String,
    id: String,
    routing: Option<String>,
) -> Result<u64, String> {
    crate::commands::document_cmd::document_delete_document(
        state,
        connection_id,
        database,
        collection,
        id,
        routing,
        None,
    )
    .await
}

#[tauri::command]
pub async fn mongo_delete_documents(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    collection: String,
    filter_json: String,
    many: bool,
    mcp_request: Option<bool>,
) -> Result<u64, String> {
    reject_mcp_request(mcp_request)?;
    ensure_connection_writable(&state, &connection_id, "Delete").await?;
    sqdesk_core::mongo_ops::mongo_delete_documents_core(
        &state,
        &connection_id,
        &database,
        &collection,
        &filter_json,
        many,
    )
    .await
}

#[tauri::command]
pub async fn mongo_find_one_and_update(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    collection: String,
    filter_json: String,
    update_json: String,
    options_json: Option<String>,
    mcp_request: Option<bool>,
) -> Result<MongoDocumentResult, String> {
    reject_mcp_request(mcp_request)?;
    ensure_connection_writable(&state, &connection_id, "Update").await?;
    sqdesk_core::mongo_ops::mongo_find_one_and_update_core(
        &state,
        &connection_id,
        &database,
        &collection,
        &filter_json,
        &update_json,
        options_json.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn mongo_find_one_and_replace(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    collection: String,
    filter_json: String,
    replacement_json: String,
    options_json: Option<String>,
    mcp_request: Option<bool>,
) -> Result<MongoDocumentResult, String> {
    reject_mcp_request(mcp_request)?;
    ensure_connection_writable(&state, &connection_id, "Update").await?;
    sqdesk_core::mongo_ops::mongo_find_one_and_replace_core(
        &state,
        &connection_id,
        &database,
        &collection,
        &filter_json,
        &replacement_json,
        options_json.as_deref(),
    )
    .await
}

#[tauri::command]
pub async fn mongo_find_one_and_delete(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    database: String,
    collection: String,
    filter_json: String,
    options_json: Option<String>,
    mcp_request: Option<bool>,
) -> Result<MongoDocumentResult, String> {
    reject_mcp_request(mcp_request)?;
    ensure_connection_writable(&state, &connection_id, "Delete").await?;
    sqdesk_core::mongo_ops::mongo_find_one_and_delete_core(
        &state,
        &connection_id,
        &database,
        &collection,
        &filter_json,
        options_json.as_deref(),
    )
    .await
}
