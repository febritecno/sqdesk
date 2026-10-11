#![recursion_limit = "256"]

pub use sqdesk_sql_data::query_result_sql;

pub use sqdesk_driver_support::{runtime_config, ssh_config};
pub use sqdesk_platform::download::DownloadSource;
pub use sqdesk_platform::{path_utils, process};
pub use sqdesk_sql_core::{mysql_ddl_normalize, mysql_event_sql, sql, sql_error_position, sqlserver_temporal};
pub use sqdesk_sql_dialect::sql_dialect;
pub use sqdesk_types::{database_manifest, models, types};

pub use sqdesk_driver_agent::{
    agent_catalog, agent_connection, agent_manager, agent_offline_export, agent_recovery, agent_runtime, agent_service,
    backend_error, database_capabilities, oracle_oci,
};
pub mod db;
pub use sqdesk_driver_support::execution;
pub mod metadata;
pub use sqdesk_driver_mongodb::{mongo_oidc, mongo_shell};
pub mod salesforce_oauth;
