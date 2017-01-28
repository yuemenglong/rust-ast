use meta::EntityMeta;

pub fn entity_get_columns(meta: &EntityMeta) -> Vec<String> {
    meta.fields
        .iter()
        .filter(|field| !field.pkey)
        .map(|field| field.field_name.clone())
        .collect::<Vec<_>>()
}

pub fn sql_create_table(meta: &EntityMeta) -> String {
    let fields = meta.fields
        .iter()
        .map(|field| field.db_ty.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    format!("CREATE TABLE IF NOT EXISTS `{}`({})",
            meta.table_name,
            fields)
}
pub fn sql_drop_table(meta: &EntityMeta) -> String {
    format!("DROP TABLE IF EXISTS `{}`", meta.table_name)
}

pub fn sql_insert(meta: &EntityMeta) -> String {
    let fields = entity_get_columns(meta).join(", ");
    let values = entity_get_columns(meta)
        .iter()
        .map(|column| format!(":{}", column))
        .collect::<Vec<_>>()
        .join(", ");
    format!("INSERT INTO `{}`({}) VALUES ({})",
            &meta.table_name,
            &fields,
            &values)
}