# Reduce MySQL connections held per app

**Status:** not started. Findings are measured; no code has been changed.

## Problem

SQDesk opens far more MySQL connections than one desktop client needs, and keeps some of them forever.

Measured against a local MySQL 8.0 with one saved connection, during normal browsing:

| Moment | Connections from the app |
| --- | --- |
| Peak, while opening tables and expanding 5 databases | 31 |
| About 80 seconds after the last action | 9 |
| Server `Max_used_connections` | 46 (limit 151) |

The floor of 9 was exactly one per database touched (8 databases, one of them holding 2).

Harmless on a local server. On shared or managed MySQL, where a user is often limited to 20–60 connections, one SQDesk can take most of the allowance and block other clients.

## Cause

There is one connection pool **per database**, not per connection.

| Rule | Value | Where |
| --- | --- | --- |
| Pool key | connection + database (+ session, + role) | `crates/sqdesk-core/src/connection/mod.rs` |
| Minimum connections | 1 | `PoolConstraints::new(1, max)` in `crates/sqdesk-driver-mysql/src/mysql.rs`, `create_pool` |
| Maximum connections | 10 for shared pools, 1 for session pools | `mysql_pool_max_connections_for_session` in `connection/mod.rs` |
| Metadata pool limit | 6 | `METADATA_POOL_DEFAULT_LIMIT` |
| Idle connection lifetime | `idle_timeout_secs`, default 60 (minimum honoured: 30, fallback 300) | `create_pool` |
| Keepalive | ping every `keepalive_interval_secs`, default 30 | `start_keepalive_task` |
| Closing an unused pool | never, until the connection is disconnected | no eviction for native pools |

Consequences:

- Every database ever opened keeps at least 1 connection for the life of the session: the minimum of 1 never expires, and the keepalive pings it every 30 seconds.
- Opening a table runs its query on its own connection, so opening several tables grows the pool quickly.
- Expanding a database loads metadata in parallel and took 3 connections at once.

Idle shrinking above the minimum does work: connections idle for 60–80 seconds were closed.

## Options

1. **Close pools of databases that are no longer in use.** After a few minutes without activity, drop the whole pool of a non-active database. Floor falls from "1 per database" to 1 or 2 in total. The mechanism already exists for agent pools only: `reclaim_idle_base_pool_for_session` and `PoolActivity::last_used_at_ms`.
2. **Lower the per-pool maximum** from 10 to 3 or 4. Peak drops sharply. Cost: restoring many tabs at once queues instead of running in parallel.
3. **Keep alive only the active database's pool.** Let the others expire on their own.

Recommended: 1 and 2 together.

## Things to watch

- A pool must not be closed while a query, a manual transaction or a streaming read is using it. `running_queries.is_pool_active` and `begin_pool_drain` are the existing guards.
- Session pools (max 1) hold session state such as temporary tables and must not be reclaimed behind the user's back.
- Reopening a closed pool costs a new handshake; the first query on a database that was idle will be slightly slower.
- The same per-database pooling exists for PostgreSQL (`postgres_pool_max_connections_for_session`). There it is necessary, because a PostgreSQL connection is bound to one database. MySQL can `USE` any database on one connection, which is why this is wasteful only for MySQL-family drivers.

## How to verify

Run the same measurement before and after, with the app idle and then while browsing:

```sql
SELECT IFNULL(t.PROCESSLIST_DB, '(no database)') AS db,
       COUNT(*)                                  AS connections,
       MIN(t.PROCESSLIST_TIME)                   AS min_idle_s,
       MAX(t.PROCESSLIST_TIME)                   AS max_idle_s
FROM performance_schema.threads t
WHERE t.PROCESSLIST_USER = 'root'
  AND t.PROCESSLIST_ID <> CONNECTION_ID()
GROUP BY 1
ORDER BY 1;

SHOW GLOBAL STATUS LIKE 'Max_used_connections';
```

Sample every 30 seconds for a few minutes: connection counts change with activity, and a single reading is misleading.

Targets: floor of 2 or fewer after a few idle minutes, peak under 10 during normal browsing, and no failed query when returning to a database whose pool was closed.

Core tests need a larger thread stack: `RUST_MIN_STACK=16777216 cargo test -p sqdesk-core --lib`.
