#[tauri::command]
pub fn prepare_schema_diff(
    options: sqdesk_core::schema_diff::SchemaDiffPreparationOptions,
) -> Result<sqdesk_core::schema_diff::SchemaDiffPreparation, String> {
    Ok(sqdesk_core::schema_diff::prepare_schema_diff(options))
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn generate_schema_sync_sql(
    diffs: Vec<sqdesk_core::schema_diff::TableDiff>,
    function_diffs: Option<Vec<sqdesk_core::schema_diff::FunctionDiff>>,
    sequence_diffs: Option<Vec<sqdesk_core::schema_diff::SequenceDiff>>,
    rule_diffs: Option<Vec<sqdesk_core::schema_diff::RuleDiff>>,
    owner_diffs: Option<Vec<sqdesk_core::schema_diff::OwnerDiff>>,
    database_type: sqdesk_core::models::connection::DatabaseType,
    target_schema: Option<String>,
    cascade_delete: Option<bool>,
    source_dialect: Option<sqdesk_core::sql_dialect::descriptor::DialectKind>,
    field_mappings: Option<Vec<sqdesk_core::schema_diff::FieldMapping>>,
) -> Result<String, String> {
    Ok(sqdesk_core::schema_diff::generate_schema_sync_sql(
        &diffs,
        function_diffs.as_deref().unwrap_or_default(),
        sequence_diffs.as_deref().unwrap_or_default(),
        rule_diffs.as_deref().unwrap_or_default(),
        owner_diffs.as_deref().unwrap_or_default(),
        database_type,
        target_schema.as_deref(),
        cascade_delete.unwrap_or(false),
        source_dialect,
        &field_mappings.unwrap_or_default(),
    ))
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn generate_schema_sync_plan(
    diffs: Vec<sqdesk_core::schema_diff::TableDiff>,
    function_diffs: Option<Vec<sqdesk_core::schema_diff::FunctionDiff>>,
    sequence_diffs: Option<Vec<sqdesk_core::schema_diff::SequenceDiff>>,
    rule_diffs: Option<Vec<sqdesk_core::schema_diff::RuleDiff>>,
    owner_diffs: Option<Vec<sqdesk_core::schema_diff::OwnerDiff>>,
    database_type: sqdesk_core::models::connection::DatabaseType,
    target_schema: Option<String>,
    cascade_delete: Option<bool>,
    source_dialect: Option<sqdesk_core::sql_dialect::descriptor::DialectKind>,
    field_mappings: Option<Vec<sqdesk_core::schema_diff::FieldMapping>>,
    enable_rollback: Option<bool>,
) -> Result<sqdesk_core::schema_diff::SchemaSyncSqlPlan, String> {
    Ok(sqdesk_core::schema_diff::generate_schema_sync_sql_plan(
        &diffs,
        function_diffs.as_deref().unwrap_or_default(),
        sequence_diffs.as_deref().unwrap_or_default(),
        rule_diffs.as_deref().unwrap_or_default(),
        owner_diffs.as_deref().unwrap_or_default(),
        database_type,
        target_schema.as_deref(),
        cascade_delete.unwrap_or(false),
        source_dialect,
        &field_mappings.unwrap_or_default(),
        enable_rollback.unwrap_or(false),
    ))
}
