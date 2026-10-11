use std::any::TypeId;

fn assert_same_type<Legacy: 'static, Extracted: 'static>() {
    assert_eq!(TypeId::of::<Legacy>(), TypeId::of::<Extracted>());
}

#[test]
fn extracted_crates_preserve_legacy_type_identity() {
    assert_same_type::<sqdesk_core::models::connection::ConnectionConfig, sqdesk_types::models::connection::ConnectionConfig>(
    );
    assert_same_type::<sqdesk_core::models::connection::DatabaseType, sqdesk_types::models::connection::DatabaseType>();
    assert_same_type::<sqdesk_core::types::QueryResult, sqdesk_types::types::QueryResult>();
    assert_same_type::<sqdesk_core::db::mysql::MySqlPool, sqdesk_drivers::db::mysql::MySqlPool>();
    assert_same_type::<sqdesk_core::db::agent_driver::AgentDriverClient, sqdesk_drivers::db::agent_driver::AgentDriverClient>(
    );
    assert_same_type::<sqdesk_core::schema_diff::SchemaDiffPreparation, sqdesk_sql::schema_diff::SchemaDiffPreparation>();
    assert_same_type::<
        sqdesk_core::table_structure_sql::TableStructureSqlOptions,
        sqdesk_sql::table_structure_sql::TableStructureSqlOptions,
    >();
    assert_same_type::<sqdesk_core::csv_export::CsvQuoteMode, sqdesk_formats::csv_export::CsvQuoteMode>();
    assert_same_type::<sqdesk_core::plugins::PluginManifest, sqdesk_plugin_runtime::plugins::PluginManifest>();
    assert_same_type::<sqdesk_core::ai::AiProvider, sqdesk_ai_provider::ai::AiProvider>();
    assert_same_type::<sqdesk_core::db::ssh_prompt::SshPromptRequest, sqdesk_platform::ssh_prompt::SshPromptRequest>();
    assert_same_type::<sqdesk_core::db::ssh_prompt::UserInputRequest, sqdesk_platform::ssh_prompt::UserInputRequest>();
}

#[test]
fn business_directories_preserve_legacy_namespaces() {
    assert_same_type::<sqdesk_core::storage::Storage, sqdesk_core::persistence::storage::Storage>();
    assert_same_type::<sqdesk_core::query_cancel::RunningTaskMetadata, sqdesk_core::query::query_cancel::RunningTaskMetadata>(
    );
}
