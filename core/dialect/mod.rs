//! Schema dialects.
//!
//! The [`SchemaDialect`] trait is the boundary between the engine and a
//! frontend's SQL surface: schema-row parsing/formatting and dialect-provided
//! scalar functions. The [`sqlite`] module owns the default dialect and the
//! catalog tables that ship with every Turso build (`pragma_*`,
//! `json_each`/`json_tree`, `sqlite_dbpage`, `btree_dump`,
//! `sqlite_turso_types`).

pub mod sqlite;

pub use sqlite::SQLiteSchemaDialect;

/// Schema dialect layered on top of the SQLite engine.
///
/// Every [`crate::Database`] carries a dialect, [`SQLiteSchemaDialect`] by
/// default. The engine goes through it whenever it parses a `sqlite_schema`
/// table row or formats the schema SQL to store there, so a frontend can
/// preserve its own schema SQL and reparse it on schema load.
pub trait SchemaDialect: Send + Sync {
    /// Parse a `sqlite_schema` SQL row into a table definition.
    fn parse_sql(&self, sql: &str, root_page: i64) -> crate::Result<crate::schema::BTreeTable>;

    /// Format the schema SQL to store in `sqlite_schema` for a `CREATE TABLE`.
    fn to_sql(
        &self,
        input: &str,
        tbl_name: &turso_parser::ast::QualifiedName,
        body: &turso_parser::ast::CreateTableBody,
    ) -> crate::Result<String>;

    /// Report whether the dialect relies on the custom-type machinery
    /// (DECODE/ENCODE, affinity metadata). When true, custom types are
    /// enabled for the database regardless of the experimental flag.
    fn force_custom_types(&self) -> bool;

    /// Report whether the dialect provides a scalar function with this name
    /// and argument count. Consulted during translation after built-in and
    /// extension function resolution.
    fn resolve_function(&self, name: &str, arg_count: usize) -> bool;

    /// Execute a dialect-provided scalar function.
    ///
    /// The bytecode interpreter routes dialect-specific scalar functions
    /// (e.g. the PostgreSQL catalog functions) here, so their implementations
    /// live with the frontend that owns them rather than in the engine.
    fn scalar_function(
        &self,
        conn: &crate::Connection,
        name: &str,
        args: &[crate::Value],
    ) -> crate::Result<crate::Value>;
}
