//! Desktop (Tauri) commands for message queue admin operations.
//!
//! These are thin wrappers around `sqdesk_core::mq::service::*_core` functions,
//! with read-only protection (`ensure_connection_writable`) for mutating calls.

use std::sync::Arc;

use tauri::State;

use crate::commands::connection::{ensure_connection_writable, AppState};

// ---- Test connection ----

#[tauri::command]
pub async fn mq_test_connection(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
) -> Result<sqdesk_core::mq::MqClusterInfo, String> {
    sqdesk_core::mq::service::mq_test_connection_core(&state, &connection_id).await
}

// ---- Tenants ----

#[tauri::command]
pub async fn mq_list_tenants(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
) -> Result<Vec<sqdesk_core::mq::TenantInfo>, String> {
    sqdesk_core::mq::service::mq_list_tenants_core(&state, &connection_id).await
}

#[tauri::command]
pub async fn mq_get_tenant(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    name: String,
) -> Result<sqdesk_core::mq::TenantInfo, String> {
    sqdesk_core::mq::service::mq_get_tenant_core(&state, &connection_id, &name).await
}

#[tauri::command]
pub async fn mq_create_tenant(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    name: String,
    config: sqdesk_core::mq::TenantConfig,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Create tenant").await?;
    sqdesk_core::mq::service::mq_create_tenant_core(&state, &connection_id, &name, config).await
}

#[tauri::command]
pub async fn mq_update_tenant(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    name: String,
    config: sqdesk_core::mq::TenantConfig,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Update tenant").await?;
    sqdesk_core::mq::service::mq_update_tenant_core(&state, &connection_id, &name, config).await
}

#[tauri::command]
pub async fn mq_delete_tenant(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    name: String,
    force: bool,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Delete tenant").await?;
    sqdesk_core::mq::service::mq_delete_tenant_core(&state, &connection_id, &name, force).await
}

// ---- Namespaces ----

#[tauri::command]
pub async fn mq_list_namespaces(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    tenant: String,
) -> Result<Vec<sqdesk_core::mq::NamespaceInfo>, String> {
    sqdesk_core::mq::service::mq_list_namespaces_core(&state, &connection_id, &tenant).await
}

#[tauri::command]
pub async fn mq_create_namespace(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    ns: sqdesk_core::mq::NamespaceRef,
    config: sqdesk_core::mq::NamespaceConfig,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Create namespace").await?;
    sqdesk_core::mq::service::mq_create_namespace_core(&state, &connection_id, ns, config).await
}

#[tauri::command]
pub async fn mq_delete_namespace(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    ns: sqdesk_core::mq::NamespaceRef,
    force: bool,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Delete namespace").await?;
    sqdesk_core::mq::service::mq_delete_namespace_core(&state, &connection_id, ns, force).await
}

#[tauri::command]
pub async fn mq_get_namespace_policies(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    ns: sqdesk_core::mq::NamespaceRef,
) -> Result<serde_json::Value, String> {
    sqdesk_core::mq::service::mq_get_namespace_policies_core(&state, &connection_id, ns).await
}

// ---- Topics ----

#[tauri::command]
pub async fn mq_list_topics(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    ns: sqdesk_core::mq::NamespaceRef,
    opts: sqdesk_core::mq::ListTopicsOpts,
) -> Result<Vec<sqdesk_core::mq::TopicInfo>, String> {
    sqdesk_core::mq::service::mq_list_topics_core(&state, &connection_id, ns, opts).await
}

#[tauri::command]
pub async fn mq_list_topics_page(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    ns: sqdesk_core::mq::NamespaceRef,
    opts: sqdesk_core::mq::ListTopicsOpts,
    pagination: sqdesk_core::mq::MqListPageRequest,
) -> Result<sqdesk_core::mq::MqListPage<sqdesk_core::mq::TopicInfo>, String> {
    sqdesk_core::mq::service::mq_list_topics_page_core(&state, &connection_id, ns, opts, pagination).await
}

#[tauri::command]
pub async fn mq_create_topic(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
    partitions: Option<u32>,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Create topic").await?;
    sqdesk_core::mq::service::mq_create_topic_core(&state, &connection_id, topic, partitions).await
}

#[tauri::command]
pub async fn mq_delete_topic(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
    force: bool,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Delete topic").await?;
    sqdesk_core::mq::service::mq_delete_topic_core(&state, &connection_id, topic, force).await
}

#[tauri::command]
pub async fn mq_update_partitions(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
    partitions: u32,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Update partitions").await?;
    sqdesk_core::mq::service::mq_update_partitions_core(&state, &connection_id, topic, partitions).await
}

#[tauri::command]
pub async fn mq_get_topic_stats(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
) -> Result<sqdesk_core::mq::TopicStats, String> {
    sqdesk_core::mq::service::mq_get_topic_stats_core(&state, &connection_id, topic).await
}

#[tauri::command]
pub async fn mq_get_topic_internal_stats(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
) -> Result<serde_json::Value, String> {
    sqdesk_core::mq::service::mq_get_topic_internal_stats_core(&state, &connection_id, topic).await
}

// ---- Exchanges ----

#[tauri::command]
pub async fn mq_list_exchanges(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    ns: sqdesk_core::mq::NamespaceRef,
) -> Result<Vec<sqdesk_core::mq::MqExchangeInfo>, String> {
    sqdesk_core::mq::service::mq_list_exchanges_core(&state, &connection_id, ns).await
}

#[tauri::command]
pub async fn mq_list_exchanges_page(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    ns: sqdesk_core::mq::NamespaceRef,
    pagination: sqdesk_core::mq::MqListPageRequest,
) -> Result<sqdesk_core::mq::MqListPage<sqdesk_core::mq::MqExchangeInfo>, String> {
    sqdesk_core::mq::service::mq_list_exchanges_page_core(&state, &connection_id, ns, pagination).await
}

#[tauri::command]
pub async fn mq_create_exchange(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    ns: sqdesk_core::mq::NamespaceRef,
    name: String,
    exchange_type: String,
    durable: bool,
    auto_delete: bool,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Create exchange").await?;
    sqdesk_core::mq::service::mq_create_exchange_core(
        &state,
        &connection_id,
        ns,
        &name,
        &exchange_type,
        durable,
        auto_delete,
    )
    .await
}

#[tauri::command]
pub async fn mq_delete_exchange(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    ns: sqdesk_core::mq::NamespaceRef,
    name: String,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Delete exchange").await?;
    sqdesk_core::mq::service::mq_delete_exchange_core(&state, &connection_id, ns, &name).await
}

// ---- Bindings ----

#[tauri::command]
pub async fn mq_list_bindings(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    ns: sqdesk_core::mq::NamespaceRef,
    exchange: Option<String>,
    queue: Option<String>,
) -> Result<Vec<sqdesk_core::mq::MqBindingInfo>, String> {
    sqdesk_core::mq::service::mq_list_bindings_core(&state, &connection_id, ns, exchange, queue).await
}

#[tauri::command]
pub async fn mq_bind(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    ns: sqdesk_core::mq::NamespaceRef,
    binding: sqdesk_core::mq::MqBindingInfo,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Create binding").await?;
    sqdesk_core::mq::service::mq_bind_core(&state, &connection_id, ns, binding).await
}

#[tauri::command]
pub async fn mq_unbind(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    ns: sqdesk_core::mq::NamespaceRef,
    binding: sqdesk_core::mq::MqBindingInfo,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Delete binding").await?;
    sqdesk_core::mq::service::mq_unbind_core(&state, &connection_id, ns, binding).await
}

// ---- Subscriptions ----

#[tauri::command]
pub async fn mq_list_subscriptions(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
) -> Result<Vec<sqdesk_core::mq::SubscriptionInfo>, String> {
    sqdesk_core::mq::service::mq_list_subscriptions_core(&state, &connection_id, topic).await
}

#[tauri::command]
pub async fn mq_enrich_subscriptions(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
) -> Result<Vec<sqdesk_core::mq::SubscriptionInfo>, String> {
    sqdesk_core::mq::service::mq_enrich_subscriptions_core(&state, &connection_id, topic).await
}

#[tauri::command]
pub async fn mq_get_kafka_consumer_group_snapshot(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
) -> Result<sqdesk_core::mq::KafkaConsumerGroupSnapshot, String> {
    sqdesk_core::mq::service::mq_get_kafka_consumer_group_snapshot_core(&state, &connection_id).await
}

#[tauri::command]
pub async fn mq_create_subscription(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
    sub: String,
    pos: sqdesk_core::mq::ResetPosition,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Create subscription").await?;
    sqdesk_core::mq::service::mq_create_subscription_core(&state, &connection_id, topic, sub, pos).await
}

#[tauri::command]
pub async fn mq_delete_subscription(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
    sub: String,
    force: bool,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Delete subscription").await?;
    sqdesk_core::mq::service::mq_delete_subscription_core(&state, &connection_id, topic, sub, force).await
}

#[tauri::command]
pub async fn mq_skip_messages(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
    sub: String,
    count: sqdesk_core::mq::SkipCount,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Skip messages").await?;
    sqdesk_core::mq::service::mq_skip_messages_core(&state, &connection_id, topic, sub, count).await
}

#[tauri::command]
pub async fn mq_reset_cursor(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
    sub: String,
    pos: sqdesk_core::mq::ResetPosition,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Reset cursor").await?;
    sqdesk_core::mq::service::mq_reset_cursor_core(&state, &connection_id, topic, sub, pos).await
}

#[tauri::command]
pub async fn mq_clear_backlog(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
    sub: String,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Clear backlog").await?;
    sqdesk_core::mq::service::mq_clear_backlog_core(&state, &connection_id, topic, sub).await
}

#[tauri::command]
pub async fn mq_get_consumer_group_config(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    group_id: String,
) -> Result<serde_json::Value, String> {
    sqdesk_core::mq::service::mq_get_consumer_group_config_core(&state, &connection_id, group_id).await
}

#[tauri::command]
pub async fn mq_alter_consumer_group_config(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    group_id: String,
    config: serde_json::Value,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Alter consumer group config").await?;
    sqdesk_core::mq::service::mq_alter_consumer_group_config_core(&state, &connection_id, group_id, config).await
}

#[tauri::command]
pub async fn mq_peek_messages(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
    sub: String,
    count: u32,
    options: Option<sqdesk_core::mq::PeekMessagesOptions>,
) -> Result<sqdesk_core::mq::PeekMessagesResult, String> {
    sqdesk_core::mq::service::mq_peek_messages_core(&state, &connection_id, topic, sub, count, options).await
}

#[tauri::command]
pub async fn mq_expire_messages(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
    sub: String,
    expire_seconds: i64,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Expire messages").await?;
    sqdesk_core::mq::service::mq_expire_messages_core(&state, &connection_id, topic, sub, expire_seconds).await
}

// ---- Producers / consumers ----

#[tauri::command]
pub async fn mq_list_producers(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
) -> Result<Vec<sqdesk_core::mq::ProducerInfo>, String> {
    sqdesk_core::mq::service::mq_list_producers_core(&state, &connection_id, topic).await
}

#[tauri::command]
pub async fn mq_list_consumers(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
    sub: String,
) -> Result<Vec<sqdesk_core::mq::ConsumerInfo>, String> {
    sqdesk_core::mq::service::mq_list_consumers_core(&state, &connection_id, topic, sub).await
}

#[tauri::command]
pub async fn mq_unload_topic(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Unload topic").await?;
    sqdesk_core::mq::service::mq_unload_topic_core(&state, &connection_id, topic).await
}

// ---- Client connections / channels (RabbitMQ) ----

#[tauri::command]
pub async fn mq_list_client_connections(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    ns: sqdesk_core::mq::NamespaceRef,
) -> Result<Vec<sqdesk_core::mq::MqClientConnectionInfo>, String> {
    sqdesk_core::mq::service::mq_list_client_connections_core(&state, &connection_id, ns).await
}

#[tauri::command]
pub async fn mq_list_client_channels(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    ns: sqdesk_core::mq::NamespaceRef,
    connection: Option<String>,
) -> Result<Vec<sqdesk_core::mq::MqChannelInfo>, String> {
    sqdesk_core::mq::service::mq_list_client_channels_core(&state, &connection_id, ns, connection).await
}

#[tauri::command]
pub async fn mq_close_client_connection(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    ns: sqdesk_core::mq::NamespaceRef,
    name: String,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Close client connection").await?;
    sqdesk_core::mq::service::mq_close_client_connection_core(&state, &connection_id, ns, &name).await
}

// ---- Rate limits / quotas / retention ----

#[tauri::command]
pub async fn mq_set_publish_rate(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    scope: sqdesk_core::mq::PolicyScope,
    rate: sqdesk_core::mq::PublishRate,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Set publish rate").await?;
    sqdesk_core::mq::service::mq_set_publish_rate_core(&state, &connection_id, scope, rate).await
}

#[tauri::command]
pub async fn mq_set_dispatch_rate(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    scope: sqdesk_core::mq::PolicyScope,
    rate: sqdesk_core::mq::DispatchRate,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Set dispatch rate").await?;
    sqdesk_core::mq::service::mq_set_dispatch_rate_core(&state, &connection_id, scope, rate).await
}

#[tauri::command]
pub async fn mq_set_subscribe_rate(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    scope: sqdesk_core::mq::PolicyScope,
    rate: sqdesk_core::mq::SubscribeRate,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Set subscribe rate").await?;
    sqdesk_core::mq::service::mq_set_subscribe_rate_core(&state, &connection_id, scope, rate).await
}

#[tauri::command]
pub async fn mq_set_backlog_quota(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    scope: sqdesk_core::mq::PolicyScope,
    quota: sqdesk_core::mq::BacklogQuota,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Set backlog quota").await?;
    sqdesk_core::mq::service::mq_set_backlog_quota_core(&state, &connection_id, scope, quota).await
}

#[tauri::command]
pub async fn mq_set_retention(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    scope: sqdesk_core::mq::PolicyScope,
    retention: sqdesk_core::mq::RetentionPolicy,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Set retention").await?;
    sqdesk_core::mq::service::mq_set_retention_core(&state, &connection_id, scope, retention).await
}

#[tauri::command]
pub async fn mq_get_effective_policies(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    scope: sqdesk_core::mq::PolicyScope,
) -> Result<serde_json::Value, String> {
    sqdesk_core::mq::service::mq_get_effective_policies_core(&state, &connection_id, scope).await
}

// ---- Permissions ----

#[tauri::command]
pub async fn mq_grant_permission(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    scope: sqdesk_core::mq::PolicyScope,
    role: String,
    actions: Vec<sqdesk_core::mq::AuthAction>,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Grant permission").await?;
    sqdesk_core::mq::service::mq_grant_permission_core(&state, &connection_id, scope, role, actions).await
}

#[tauri::command]
pub async fn mq_revoke_permission(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    scope: sqdesk_core::mq::PolicyScope,
    role: String,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Revoke permission").await?;
    sqdesk_core::mq::service::mq_revoke_permission_core(&state, &connection_id, scope, role).await
}

#[tauri::command]
pub async fn mq_list_permissions(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    scope: sqdesk_core::mq::PolicyScope,
) -> Result<sqdesk_core::mq::PermissionMap, String> {
    sqdesk_core::mq::service::mq_list_permissions_core(&state, &connection_id, scope).await
}

// ---- Users / user permissions (RabbitMQ) ----

#[tauri::command]
pub async fn mq_list_users(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
) -> Result<Vec<sqdesk_core::mq::MqUserInfo>, String> {
    sqdesk_core::mq::service::mq_list_users_core(&state, &connection_id).await
}

#[tauri::command]
pub async fn mq_create_user(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    name: String,
    password: String,
    tags: Option<Vec<String>>,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Create user").await?;
    sqdesk_core::mq::service::mq_create_user_core(&state, &connection_id, &name, &password, tags.unwrap_or_default())
        .await
}

#[tauri::command]
pub async fn mq_delete_user(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    name: String,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Delete user").await?;
    sqdesk_core::mq::service::mq_delete_user_core(&state, &connection_id, &name).await
}

/// RabbitMQ user permissions live under the synthetic `_rabbitmq` tenant;
/// `*` is the all-vhosts marker and is only meaningful for listings.
fn user_permission_ns(virtual_host: Option<String>, all_vhosts: Option<bool>) -> sqdesk_core::mq::NamespaceRef {
    let namespace = match (all_vhosts.unwrap_or(false), virtual_host) {
        (true, _) => "*".to_string(),
        (false, Some(vhost)) if !vhost.trim().is_empty() => vhost,
        _ => "*".to_string(),
    };
    sqdesk_core::mq::NamespaceRef { tenant: "_rabbitmq".to_string(), namespace }
}

#[tauri::command]
pub async fn mq_list_user_permissions(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    virtual_host: Option<String>,
    user: Option<String>,
    all_vhosts: Option<bool>,
) -> Result<Vec<sqdesk_core::mq::MqVhostPermission>, String> {
    let ns = user_permission_ns(virtual_host, all_vhosts);
    let mut permissions = sqdesk_core::mq::service::mq_list_user_permissions_core(&state, &connection_id, ns).await?;
    if let Some(user) = user {
        permissions.retain(|p| p.user == user);
    }
    Ok(permissions)
}

#[tauri::command]
pub async fn mq_grant_user_permission(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    user: String,
    virtual_host: String,
    configure: Option<String>,
    write: Option<String>,
    read: Option<String>,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Grant user permission").await?;
    let ns = user_permission_ns(Some(virtual_host), None);
    let all = || ".*".to_string();
    sqdesk_core::mq::service::mq_grant_user_permission_core(
        &state,
        &connection_id,
        ns,
        &user,
        &configure.unwrap_or_else(all),
        &write.unwrap_or_else(all),
        &read.unwrap_or_else(all),
    )
    .await
}

#[tauri::command]
pub async fn mq_revoke_user_permission(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    user: String,
    virtual_host: String,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Revoke user permission").await?;
    let ns = user_permission_ns(Some(virtual_host), None);
    sqdesk_core::mq::service::mq_revoke_user_permission_core(&state, &connection_id, ns, &user).await
}

// ---- Policies & cluster monitoring (RabbitMQ) ----

#[tauri::command]
pub async fn mq_list_policies(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    virtual_host: Option<String>,
    all_vhosts: Option<bool>,
) -> Result<Vec<sqdesk_core::mq::MqPolicyInfo>, String> {
    let ns = user_permission_ns(virtual_host, all_vhosts);
    sqdesk_core::mq::service::mq_list_policies_core(&state, &connection_id, ns).await
}

#[tauri::command]
pub async fn mq_set_policy(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    virtual_host: String,
    name: String,
    pattern: String,
    apply_to: Option<String>,
    priority: Option<i32>,
    definition: std::collections::HashMap<String, serde_json::Value>,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Set policy").await?;
    let ns = user_permission_ns(Some(virtual_host.clone()), None);
    let policy = sqdesk_core::mq::MqPolicyInfo {
        name,
        vhost: virtual_host,
        pattern,
        apply_to: apply_to.unwrap_or_default(),
        priority: priority.unwrap_or(0),
        definition,
    };
    sqdesk_core::mq::service::mq_set_policy_core(&state, &connection_id, ns, policy).await
}

#[tauri::command]
pub async fn mq_delete_policy(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    virtual_host: String,
    name: String,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Delete policy").await?;
    let ns = user_permission_ns(Some(virtual_host), None);
    sqdesk_core::mq::service::mq_delete_policy_core(&state, &connection_id, ns, &name).await
}

#[tauri::command]
pub async fn mq_get_overview(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
) -> Result<sqdesk_core::mq::MqOverviewInfo, String> {
    sqdesk_core::mq::service::mq_get_overview_core(&state, &connection_id).await
}

#[tauri::command]
pub async fn mq_list_nodes(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
) -> Result<Vec<sqdesk_core::mq::MqNodeInfo>, String> {
    sqdesk_core::mq::service::mq_list_nodes_core(&state, &connection_id).await
}

// ---- Client tokens ----

#[tauri::command]
pub async fn mq_issue_token(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    req: sqdesk_core::mq::MqTokenIssueRequest,
) -> Result<sqdesk_core::mq::MqIssuedToken, String> {
    ensure_connection_writable(&state, &connection_id, "Issue MQ token").await?;
    sqdesk_core::mq::service::mq_issue_token_core(&state, &connection_id, req).await
}

#[tauri::command]
pub async fn mq_list_token_records(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    subject: Option<String>,
) -> Result<Vec<sqdesk_core::mq::MqTokenRecord>, String> {
    sqdesk_core::mq::service::mq_list_token_records_core(&state, &connection_id, subject).await
}

// ---- Monitoring ----

#[tauri::command]
pub async fn mq_get_backlog(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
    sub: Option<String>,
) -> Result<sqdesk_core::mq::BacklogStats, String> {
    sqdesk_core::mq::service::mq_get_backlog_core(&state, &connection_id, topic, sub).await
}

#[tauri::command]
pub async fn mq_get_cluster_info(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
) -> Result<sqdesk_core::mq::ClusterInfo, String> {
    sqdesk_core::mq::service::mq_get_cluster_info_core(&state, &connection_id).await
}

#[tauri::command]
pub async fn mq_get_topic_route(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
) -> Result<serde_json::Value, String> {
    sqdesk_core::mq::service::mq_get_topic_route_core(&state, &connection_id, topic).await
}

#[tauri::command]
pub async fn mq_alter_topic_config(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
    configs: serde_json::Value,
) -> Result<(), String> {
    ensure_connection_writable(&state, &connection_id, "Alter topic config").await?;
    sqdesk_core::mq::service::mq_alter_topic_config_core(&state, &connection_id, topic, configs).await
}

#[tauri::command]
pub async fn mq_skip_topic_accumulation(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
) -> Result<serde_json::Value, String> {
    ensure_connection_writable(&state, &connection_id, "Skip topic accumulation").await?;
    sqdesk_core::mq::service::mq_skip_topic_accumulation_core(&state, &connection_id, topic).await
}

#[tauri::command]
pub async fn mq_view_message(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
    msg_id: String,
) -> Result<serde_json::Value, String> {
    sqdesk_core::mq::service::mq_view_message_core(&state, &connection_id, topic, msg_id).await
}

#[tauri::command]
pub async fn mq_query_messages_by_key(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
    key: String,
    begin: i64,
    end: i64,
    max_num: u32,
) -> Result<serde_json::Value, String> {
    sqdesk_core::mq::service::mq_query_messages_by_key_core(&state, &connection_id, topic, key, begin, end, max_num)
        .await
}

#[tauri::command]
pub async fn mq_query_messages_by_topic(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    topic: sqdesk_core::mq::TopicRef,
    begin: i64,
    end: i64,
    max_num: u32,
) -> Result<serde_json::Value, String> {
    sqdesk_core::mq::service::mq_query_messages_by_topic_core(&state, &connection_id, topic, begin, end, max_num).await
}

#[tauri::command]
pub async fn mq_query_message_trace(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    msg_id: String,
    trace_topic: Option<String>,
) -> Result<serde_json::Value, String> {
    sqdesk_core::mq::service::mq_query_message_trace_core(&state, &connection_id, msg_id, trace_topic).await
}

// ---- Raw request (escape hatch) ----

#[tauri::command]
pub async fn mq_raw_request(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    req: sqdesk_core::mq::MqRawRequest,
) -> Result<sqdesk_core::mq::MqRawResponse, String> {
    if req.is_mutating() {
        ensure_connection_writable(&state, &connection_id, "MQ admin write").await?;
    }
    sqdesk_core::mq::service::mq_raw_request_core(&state, &connection_id, req).await
}

// ---- Message production ----

#[tauri::command]
pub async fn mq_send_message(
    state: State<'_, Arc<AppState>>,
    connection_id: String,
    req: sqdesk_core::mq::SendMessageRequest,
) -> Result<sqdesk_core::mq::SendMessageResponse, String> {
    ensure_connection_writable(&state, &connection_id, "Send message").await?;
    sqdesk_core::mq::service::mq_send_message_core(&state, &connection_id, req).await
}
