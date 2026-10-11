import type { ColumnInfo, ForeignKeyInfo, IndexInfo } from "@/types/database";

/**
 * Prompt building and answer clean-up for the editor's inline AI command
 * (generate SQL at the cursor, or rewrite the selection). Kept free of Vue and
 * backend calls so the rules that decide what the model sees are testable.
 */

export interface InlineSqlTableRef {
  schema?: string;
  name: string;
}

export interface InlineSqlTable extends InlineSqlTableRef {
  tableType?: string;
  columns: ColumnInfo[];
  indexes?: IndexInfo[];
  foreignKeys?: ForeignKeyInfo[];
}

export interface InlineSqlRequestInput {
  request: string;
  selectedSql: string;
  /** Editor text around the cursor or selection, for context only. */
  textBefore?: string;
  textAfter?: string;
  databaseType: string;
  serverVersion?: string;
  database?: string;
  schema?: string;
  /** Tables sent with columns, indexes and foreign keys. */
  tables: InlineSqlTable[];
  /** Tables the model may know exist but gets no columns for. */
  otherTableNames: string[];
  /** True when the table list itself was cut short. */
  truncated?: boolean;
  /**
   * Earlier requests of this panel with the SQL that was shown for each,
   * oldest first. Present for a follow-up, which revises the last of them.
   */
  conversation?: { request: string; sql: string }[];
}

/** How many alternative answers the model may offer for one request. */
export const MAX_INLINE_SQL_OPTIONS = 3;

const WORD_BREAK = /[^a-z0-9]+/;
const STOP_WORDS = new Set(["the", "and", "for", "per", "with", "from", "into", "that", "this", "all", "any", "each", "show", "list", "get", "select", "where", "by", "of", "in", "on", "to", "is", "are", "data", "table", "tables", "rows", "row", "count", "total", "top", "last", "first"]);

function singular(word: string): string {
  if (word.length > 4 && word.endsWith("ies")) return `${word.slice(0, -3)}y`;
  if (word.length > 3 && word.endsWith("ses")) return word.slice(0, -2);
  if (word.length > 3 && word.endsWith("s") && !word.endsWith("ss")) return word.slice(0, -1);
  return word;
}

function words(text: string): string[] {
  return text
    .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
    .toLowerCase()
    .split(WORD_BREAK)
    .filter((word) => word.length > 2);
}

/**
 * Scores how likely a table is what the user is talking about: an exact name
 * mention beats a singular/plural match, which beats sharing name parts with
 * the request ("orders per customer" still finds `customer_orders`).
 */
export function scoreInlineSqlTable(tableName: string, text: string): number {
  const lowerText = text.toLowerCase();
  const lowerName = tableName.toLowerCase();
  const escaped = lowerName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  if (new RegExp(`(^|[^a-z0-9_])${escaped}([^a-z0-9_]|$)`).test(lowerText)) return 100;
  const textWords = new Set(words(text).map(singular));
  const nameWords = words(tableName).map(singular);
  if (nameWords.length === 0) return 0;
  if (textWords.has(singular(lowerName.replace(/_/g, "")))) return 60;
  const hits = nameWords.filter((word) => !STOP_WORDS.has(word) && textWords.has(word)).length;
  if (hits === 0) return 0;
  // All parts of the name mentioned is a stronger signal than one shared word.
  return hits === nameWords.length ? 50 : 10 * hits;
}

/** Picks which tables get full detail. A small database is sent whole. */
export function pickInlineSqlTables<T extends InlineSqlTableRef>(tables: readonly T[], relevantText: string, maxDetailed: number): T[] {
  if (tables.length <= maxDetailed) return [...tables];
  return tables
    .map((table, index) => ({ table, index, score: scoreInlineSqlTable(table.name, relevantText) }))
    .filter((entry) => entry.score > 0)
    .sort((left, right) => right.score - left.score || left.index - right.index)
    .slice(0, maxDetailed)
    .map((entry) => entry.table);
}

function qualifiedName(table: InlineSqlTableRef): string {
  return table.schema ? `${table.schema}.${table.name}` : table.name;
}

function formatColumn(column: ColumnInfo, foreignKeys: readonly ForeignKeyInfo[]): string {
  const parts = [column.name, column.data_type || "unknown"];
  if (column.is_primary_key) parts.push("PK");
  else if (column.is_unique) parts.push("UNIQUE");
  parts.push(column.is_nullable ? "NULL" : "NOT NULL");
  const reference = foreignKeys.find((foreignKey) => foreignKey.column === column.name);
  if (reference) parts.push(`-> ${reference.ref_schema ? `${reference.ref_schema}.` : ""}${reference.ref_table}.${reference.ref_column}`);
  if (column.enum_values?.length) parts.push(`one of (${column.enum_values.slice(0, 12).join(", ")})`);
  const comment = column.comment?.trim();
  return `  ${parts.join(" ")}${comment ? `  -- ${comment.replace(/\s+/g, " ").slice(0, 80)}` : ""}`;
}

function formatIndex(index: IndexInfo): string {
  const kind = index.is_primary ? "PRIMARY" : index.is_unique ? "UNIQUE" : "INDEX";
  const filter = index.filter ? ` WHERE ${index.filter}` : "";
  return `${kind} ${index.name} (${index.columns.join(", ")})${filter}`;
}

export function formatInlineSqlTable(table: InlineSqlTable): string {
  const foreignKeys = table.foreignKeys ?? [];
  const kind = table.tableType && !/^(base )?table$/i.test(table.tableType) ? table.tableType.toLowerCase() : "table";
  const lines = [`${kind} ${qualifiedName(table)} (`, ...table.columns.map((column) => formatColumn(column, foreignKeys)), ")"];
  const indexes = (table.indexes ?? []).filter((index) => index.columns.length > 0);
  if (indexes.length) lines.push(`  indexes: ${indexes.map(formatIndex).join("; ")}`);
  return lines.join("\n");
}

function keywordCase(selectedSql: string): "uppercase" | "lowercase" {
  const keywords = selectedSql.match(/\b(select|from|where|join|group by|order by|insert|update|delete)\b/gi) ?? [];
  if (keywords.length === 0) return "uppercase";
  return keywords.filter((keyword) => keyword === keyword.toLowerCase()).length > keywords.length / 2 ? "lowercase" : "uppercase";
}

function fenced(sql: string): string {
  return `\`\`\`sql\n${sql.trim()}\n\`\`\``;
}

export function buildInlineSqlSystemPrompt(input: Pick<InlineSqlRequestInput, "databaseType" | "serverVersion" | "selectedSql">): string {
  const dialect = input.serverVersion ? `${input.databaseType} ${input.serverVersion}` : input.databaseType;
  return [
    `You are an expert ${input.databaseType} engineer working inside a SQL editor. The SQL you return is inserted into the editor as-is.`,
    "",
    "Output contract:",
    "- Reply with exactly one ```sql fenced block and nothing else. No prose, notes or explanations before or after it.",
    `- Alternatives: only when the request can honestly be read in more than one way, or there are clearly different useful answers, reply with up to ${MAX_INLINE_SQL_OPTIONS} fenced blocks instead, best first. Put one label line starting with "### " (six words at most) directly above each block, and nothing else. A clear request gets one block and no label.`,
    "- The block contains SQL only. Do not add comments of any kind, and do not explain or justify the query, unless the request asks for comments. Keep comments the user already wrote.",
    `- The SQL must be complete and runnable on ${dialect}. Use only this dialect's syntax, functions, identifier quoting and pagination.`,
    "- Use only tables and columns that appear in the schema below or in the user's own SQL. Never invent names.",
    '- Refuse only when a table or column the request depends on does not exist. In that case alone, reply with one SQL comment line starting with "-- " that names what is missing, and nothing else.',
    "",
    "Write efficient SQL by default:",
    "- Select only the columns the request needs. No SELECT * unless asked for.",
    "- Keep predicates sargable: compare indexed columns directly, never wrap them in functions or casts, avoid a leading wildcard in LIKE unless required, and use half-open ranges (>= start AND < end) for dates and times.",
    "- Join on the listed foreign keys, and prefer the listed indexes for filters, joins and ordering.",
    "- Prefer EXISTS / NOT EXISTS over IN / NOT IN with a subquery. Replace a correlated subquery that runs per row with a join or a window function when one pass can do it.",
    "- Filter before aggregating. Use UNION ALL unless duplicates must be removed. Do not use DISTINCT to hide a wrong join.",
    "- Do not add ORDER BY, hints or new indexes unless asked. Exception: an exploratory request (show, list, sample, preview) with no stated size gets a row limit of 100, written in this dialect's syntax.",
    "- For INSERT, UPDATE and DELETE be exact: never write an UPDATE or DELETE without a WHERE clause unless the user explicitly asks for every row.",
    "",
    "When rewriting selected SQL:",
    "- Always return a rewritten query. Never answer with commentary about the query, a list of suggestions, or the unchanged query plus remarks.",
    "- Keep its result (columns, rows, order) unless the request changes it. Change only what was asked.",
    "- For an open-ended request (tidy, clean up, improve, optimize), apply the formatting and efficiency rules above and return the result. If nothing can be improved, return the query formatted, with no remark.",
    "- When the request is ambiguous, write the query for the most likely meaning first; other readings may follow as alternatives. Never ask a question.",
    "- Do not add a row limit, ORDER BY or DISTINCT that the selection does not have, unless the request asks for it. The exploratory row limit above applies to new queries only.",
    "- Keep the statement type. A SELECT stays a SELECT: never answer with DDL (ALTER, CREATE, DROP) or a different kind of statement unless the request asks for one.",
    "- Keep the user's names, aliases and style.",
    "- Return the full replacement for the selection, never a fragment or a diff.",
    "",
    "When revising a proposed SQL from the conversation:",
    "- Start from that proposal and apply the new request to it. Keep everything the request does not mention.",
    "- Return the complete revised query, following every rule above.",
    "",
    `Format: one clause per line (SELECT, FROM, JOIN, WHERE, GROUP BY, HAVING, ORDER BY), ${keywordCase(input.selectedSql)} keywords, two-space indentation.`,
  ].join("\n");
}

export function buildInlineSqlUserPrompt(input: InlineSqlRequestInput): string {
  const sections: string[] = [];
  const location = [`Database type: ${input.serverVersion ? `${input.databaseType} ${input.serverVersion}` : input.databaseType}`];
  if (input.database) location.push(`database: ${input.database}`);
  if (input.schema) location.push(`default schema: ${input.schema}`);
  sections.push(location.join("; "));

  if (input.tables.length) sections.push(`Schema:\n${input.tables.map(formatInlineSqlTable).join("\n\n")}`);
  else sections.push("Schema: no table details are available. Use only names that appear in the user's SQL or request.");
  if (input.otherTableNames.length) sections.push(`Other tables that exist (names only, columns not shown): ${input.otherTableNames.join(", ")}${input.truncated ? ", …" : ""}`);

  const before = input.textBefore?.trim();
  const after = input.textAfter?.trim();
  if (before) sections.push(`Editor text before ${input.selectedSql.trim() ? "the selection" : "the cursor"} (context only, do not repeat it):\n${fenced(before)}`);
  if (input.selectedSql.trim()) sections.push(`Selected SQL to rewrite:\n${fenced(input.selectedSql)}`);
  if (after) sections.push(`Editor text after (context only, do not repeat it):\n${fenced(after)}`);

  const conversation = (input.conversation ?? []).filter((turn) => turn.sql.trim());
  if (conversation.length) {
    // The editor still holds what is shown above; the SQL below was only
    // proposed. Saying so keeps the model from treating a proposal as the
    // user's own text, while still letting it build on the last one.
    const turns = conversation.map((turn, index) => `Request ${index + 1}: ${turn.request.trim()}\nProposed SQL ${index + 1}:\n${fenced(turn.sql)}`);
    sections.push(`Conversation so far (proposals the user has seen; none is in the editor yet):\n${turns.join("\n\n")}`);
    sections.push(`Revise proposed SQL ${conversation.length} as follows: ${input.request.trim()}`);
    return sections.join("\n\n");
  }

  sections.push(`${input.selectedSql.trim() ? "Rewrite the selected SQL as follows" : "Write SQL for this request"}: ${input.request.trim()}`);
  return sections.join("\n\n");
}

export interface InlineSqlAnswer {
  /** SQL to insert. Empty when the model answered with a note instead. */
  sql: string;
  /** What the model said it could not do, shown to the user instead of inserting. */
  note: string;
  /** The reply stopped inside its code block, so the SQL is incomplete and must not be inserted. */
  truncated?: boolean;
  /** Statement kind (e.g. "ALTER") the rewrite introduced without being asked; the SQL was held back. */
  unrequestedStatement?: string;
}

const SQL_START = /^\s*(--|\/\*|\(|with\b|select\b|insert\b|update\b|delete\b|merge\b|create\b|alter\b|drop\b|truncate\b|explain\b|show\b|describe\b|desc\b|call\b|exec\b|execute\b|begin\b|set\b|use\b|grant\b|revoke\b|values\b|declare\b|pragma\b|replace\b|analyze\b|vacuum\b|copy\b)/i;

/** `--` everywhere, plus MySQL's `#` (never `#>` / `#-`, which are PostgreSQL JSON operators mid-expression). */
function isLineComment(trimmedLine: string): boolean {
  return trimmedLine.startsWith("--") || /^#(\s|$)/.test(trimmedLine);
}

const SQL_CONTINUATION = /^\s*(\)|,|from\b|where\b|and\b|or\b|on\b|join\b|inner\b|left\b|right\b|full\b|cross\b|group\b|order\b|having\b|limit\b|offset\b|union\b|intersect\b|except\b|returning\b|when\b|then\b|else\b|end\b|fetch\b|window\b|into\b|using\b)/i;

/**
 * Cuts the explanation a model appends after unfenced SQL. A paragraph that
 * follows a blank line and neither starts a statement nor continues one is
 * prose, and so is everything after it.
 */
function dropTrailingProse(sql: string): string {
  const paragraphs = sql.split(/\n[ \t]*\n/);
  const kept = [paragraphs[0]];
  for (const paragraph of paragraphs.slice(1)) {
    const firstLine = paragraph.split("\n")[0] ?? "";
    if (!SQL_START.test(firstLine) && !SQL_CONTINUATION.test(firstLine)) break;
    kept.push(paragraph);
  }
  return kept.join("\n\n").trim();
}

const WRITE_STATEMENT = /(?:^|;)\s*(alter|create|drop|truncate|insert|update|delete|merge|replace|grant|revoke)\b/gi;

function statementKinds(sql: string): Set<string> {
  // Comments and string literals can contain these words without being statements.
  const bare = sql
    .replace(/\/\*[\s\S]*?\*\//g, " ")
    .replace(/--[^\n]*/g, " ")
    .replace(/'(?:[^']|'')*'/g, "''")
    .replace(/"(?:[^"]|"")*"/g, '""');
  return new Set([...bare.matchAll(WRITE_STATEMENT)].map((match) => match[1].toUpperCase()));
}

/**
 * A rewrite must not turn a query into something that changes data or schema.
 * Returns the first write statement kind the reply introduced that is neither
 * in the selection nor named in the request; models do this when asked for a
 * column that does not exist ("ALTER TABLE ... ADD COLUMN").
 */
export function unrequestedWriteStatement(selectedSql: string, sql: string, request: string): string | undefined {
  if (!selectedSql.trim()) return undefined;
  const allowed = statementKinds(selectedSql);
  const lowerRequest = request.toLowerCase();
  return [...statementKinds(sql)].find((kind) => !allowed.has(kind) && !new RegExp(`\\b${kind.toLowerCase()}\\b`).test(lowerRequest));
}

function isCommentOnly(sql: string): boolean {
  const withoutBlockComments = sql.replace(/\/\*[\s\S]*?\*\//g, "");
  return withoutBlockComments.split("\n").every((line) => line.trim() === "" || isLineComment(line.trim()));
}

/**
 * Turns a model reply into the text to insert. Models sometimes ignore the
 * "fenced block only" rule, so this tolerates a missing fence and a sentence
 * of preamble, drops comments the user did not ask for, and reports a
 * comment-only reply as a note rather than writing a comment over the user's
 * selection.
 */
const ASKS_FOR_COMMENTS = /\b(comments?|commented|annotat\w*|document\w*|explain\w*|describe|komentar\w*|jelaskan|penjelasan|keterangan|catatan|dokumentasi\w*)\b/i;

/** Whether the user asked for the SQL itself to carry comments or explanations. */
export function inlineSqlRequestWantsComments(request: string): boolean {
  return ASKS_FOR_COMMENTS.test(request);
}

/**
 * Drops whole-line comments the model added on its own. A comment line that
 * was already in the user's selection stays, so rewriting never loses their
 * notes. Trailing comments on a code line are left alone: telling them apart
 * from "--" inside a string literal is not worth the risk of cutting SQL.
 */
function stripAddedCommentLines(sql: string, selectedSql: string): string {
  const original = new Set(
    selectedSql
      .split("\n")
      .map((line) => line.trim())
      .filter(Boolean),
  );
  const kept: string[] = [];
  let inBlock = false;
  for (const line of sql.split("\n")) {
    const trimmed = line.trim();
    if (original.has(trimmed)) {
      kept.push(line);
      continue;
    }
    if (inBlock) {
      if (trimmed.includes("*/")) inBlock = false;
      continue;
    }
    if (isLineComment(trimmed)) continue;
    if (trimmed.startsWith("/*")) {
      const end = trimmed.indexOf("*/");
      if (end < 0) {
        inBlock = true;
        continue;
      }
      // An optimizer hint such as /*+ INDEX(...) */ is part of the statement.
      if (!trimmed.startsWith("/*+") && end === trimmed.length - 2) continue;
    }
    kept.push(line);
  }
  return kept
    .join("\n")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

export function cleanInlineSqlAnswer(answer: string, options: { selectedSql?: string; request?: string } = {}): InlineSqlAnswer {
  const fencedMatch = answer.match(/```[a-zA-Z0-9_+-]*[ \t]*\r?\n([\s\S]*?)```/);
  // An opening fence that never closes means the reply ran out of tokens
  // mid-statement. Half a query must not land in the editor.
  if (!fencedMatch && (answer.match(/```/g)?.length ?? 0) % 2 === 1) return { sql: "", note: "", truncated: true };
  let sql = (fencedMatch ? fencedMatch[1] : answer.replace(/```[a-zA-Z0-9_+-]*/g, "")).replace(/\r\n/g, "\n").trim();
  if (!fencedMatch) {
    const lines = sql.split("\n");
    const start = lines.findIndex((line) => SQL_START.test(line));
    if (start > 0) sql = lines.slice(start).join("\n").trim();
    sql = dropTrailingProse(sql);
  }
  if (!sql) return { sql: "", note: "" };
  if (isCommentOnly(sql)) {
    const note = sql
      .replace(/\/\*|\*\//g, "")
      .split("\n")
      .map((line) => line.replace(/^\s*(--|#)\s?/, "").trim())
      .filter(Boolean)
      .join(" ");
    return { sql: "", note };
  }
  // The editor gets the query only: unrequested commentary is removed here
  // even when the model ignores that part of the output contract.
  if (!inlineSqlRequestWantsComments(options.request ?? "")) sql = stripAddedCommentLines(sql, options.selectedSql ?? "");
  const unrequestedStatement = unrequestedWriteStatement(options.selectedSql ?? "", sql, options.request ?? "");
  if (unrequestedStatement) return { sql: "", note: "", unrequestedStatement };
  const selected = options.selectedSql?.trim();
  if (selected) {
    // Match the selection's statement terminator so a rewrite drops into place.
    const selectionEndsStatement = selected.endsWith(";");
    if (selectionEndsStatement && !sql.endsWith(";")) sql += ";";
    if (!selectionEndsStatement && sql.endsWith(";") && sql.indexOf(";") === sql.length - 1) sql = sql.slice(0, -1).trimEnd();
  }
  return { sql, note: "" };
}

/**
 * Text to insert when generating at the cursor. SQL dropped into the middle of
 * a line would fuse with what is already there ("SELECT 1SELECT 2"), so it is
 * moved onto its own line whenever the cursor's line has other content.
 */
export function inlineSqlInsertion(sql: string, lineBeforeCursor: string, lineAfterCursor: string): string {
  const prefix = lineBeforeCursor.trim() ? "\n" : "";
  const suffix = lineAfterCursor.trim() ? "\n" : "";
  return `${prefix}${sql}${suffix}`;
}

export interface InlineSqlOption {
  sql: string;
  /** Short name the model gave this alternative; empty when it gave none. */
  label: string;
}

export interface InlineSqlOptions extends Omit<InlineSqlAnswer, "sql"> {
  /** Usable answers, best first. Empty when the reply held nothing to insert. */
  options: InlineSqlOption[];
}

const FENCED_BLOCK = /```[a-zA-Z0-9_+-]*[ \t]*\r?\n([\s\S]*?)```/g;

function optionLabel(textBeforeBlock: string): string {
  const line = textBeforeBlock.trimEnd().split("\n").pop()?.trim() ?? "";
  const label = line
    .replace(/^#+\s*/, "")
    .replace(/^(option|opsi|alternative|alternatif|variant|varian)\s*\d*\s*[:.)-]?\s*/i, "")
    .replace(/[*_`:]+$/g, "")
    .trim();
  return label.length <= 60 ? label : "";
}

/**
 * Reads every alternative out of a model reply. Each fenced block is cleaned
 * by the same rules as a single answer, duplicates are dropped, and a reply
 * with no usable block reports why through the first block's outcome.
 */
export function cleanInlineSqlOptions(answer: string, options: { selectedSql?: string; request?: string } = {}): InlineSqlOptions {
  const blocks = [...answer.matchAll(FENCED_BLOCK)];
  if (blocks.length <= 1) {
    const { sql, ...rest } = cleanInlineSqlAnswer(answer, options);
    return { ...rest, options: sql ? [{ sql, label: "" }] : [] };
  }
  const results = blocks.map((block, index) => {
    const previousEnd = index === 0 ? 0 : (blocks[index - 1].index ?? 0) + blocks[index - 1][0].length;
    return { label: optionLabel(answer.slice(previousEnd, block.index ?? 0)), ...cleanInlineSqlAnswer(block[0], options) };
  });
  const usable: InlineSqlOption[] = [];
  for (const result of results) {
    if (result.sql && !usable.some((option) => option.sql === result.sql)) usable.push({ sql: result.sql, label: result.label });
  }
  if (usable.length === 0) {
    const { sql: _sql, label: _label, ...first } = results[0];
    return { ...first, options: [] };
  }
  return { note: "", options: usable.slice(0, MAX_INLINE_SQL_OPTIONS) };
}
