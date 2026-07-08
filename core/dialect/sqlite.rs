use crate::{
    pragma::PragmaVirtualTable,
    schema::{Schema, Table},
    sync::Arc,
    vtab::{VirtualTable, VirtualTableType},
};
use turso_ext::VTabKind;

/// SQLite schema dialect.
pub struct SQLiteSchemaDialect;

impl super::SchemaDialect for SQLiteSchemaDialect {
    fn parse_sql(&self, sql: &str, root_page: i64) -> crate::Result<crate::schema::BTreeTable> {
        crate::schema::BTreeTable::from_sql(sql, root_page)
    }

    fn to_sql(
        &self,
        _input: &str,
        tbl_name: &turso_parser::ast::QualifiedName,
        body: &turso_parser::ast::CreateTableBody,
    ) -> crate::Result<String> {
        Ok(format!(
            "CREATE TABLE {} {}",
            tbl_name.name.as_ident(),
            body
        ))
    }

    fn force_custom_types(&self) -> bool {
        false
    }

    fn resolve_function(&self, _name: &str, _arg_count: usize) -> bool {
        false
    }

    fn scalar_function(
        &self,
        _conn: &crate::Connection,
        name: &str,
        _args: &[crate::Value],
    ) -> crate::Result<crate::Value> {
        Err(crate::LimboError::ParseError(format!(
            "no such function: {name}"
        )))
    }
}

/// Insert the standard SQLite-style catalog tables into `schema`.
///
/// `pragma_*` virtual tables use the dedicated [`VirtualTableType::Pragma`]
/// variant (they aren't `InternalVirtualTable`), so they are inserted
/// directly. The rest go through [`Schema::register_internal_vtab`] — the
/// same path external callers use via [`crate::Database::register_internal_vtab`].
pub fn register_builtin_catalog(
    schema: &mut Schema,
    enable_custom_types: bool,
) -> crate::Result<()> {
    for vtab in pragma_vtabs() {
        schema.tables.insert(
            vtab.name.to_owned(),
            Arc::new(Table::Virtual(Arc::new((*vtab).clone()))),
        );
    }

    #[cfg(feature = "json")]
    {
        schema.register_internal_vtab(crate::json::vtab::JsonVirtualTable::json_each())?;
        schema.register_internal_vtab(crate::json::vtab::JsonVirtualTable::json_tree())?;
    }
    #[cfg(feature = "cli_only")]
    {
        schema.register_internal_vtab(crate::dbpage::DbPageTable::new())?;
        schema.register_internal_vtab(crate::btree_dump::BtreeDumpTable::new())?;
    }
    if enable_custom_types {
        schema.register_internal_vtab(crate::turso_types_vtab::TursoTypesTable::new())?;
    }
    Ok(())
}

/// Build a `VirtualTable` for each PRAGMA table-valued function.
fn pragma_vtabs() -> Vec<Arc<VirtualTable>> {
    PragmaVirtualTable::functions()
        .into_iter()
        .map(|(tab, schema_sql)| {
            Arc::new(VirtualTable {
                name: format!("pragma_{}", tab.pragma_name),
                columns: VirtualTable::resolve_columns(schema_sql)
                    .expect("pragma table-valued function schema resolution should not fail"),
                kind: VTabKind::TableValuedFunction,
                vtab_type: VirtualTableType::Pragma(tab),
                vtab_id: 0,
                innocuous: true,
            })
        })
        .collect()
}
