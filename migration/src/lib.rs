pub use sea_orm_migration::prelude::*;

mod m20260221_000001_initial_schema;
mod m20260221_000002_seed_data;
mod m20260221_000003_fts_index;
mod m20260221_000004_history_preserve_on_delete;
mod m20260221_000005_add_level_column;
mod m20260221_000006_add_review_columns;
mod m20260221_000007_content_hash_review;
mod m20260221_000008_scoped_suspect_links;
mod m20260221_000009_object_classification;
mod m20260221_000010_script_table;
mod m20260221_000011_add_module_config;
mod m20260221_000012_object_references;
mod m20260221_000013_view_table;
mod m20260221_000014_object_type_table;
mod m20260221_000015_comment_table;
mod m20260221_000016_app_user_table;
mod m20260221_000017_review_tables;
mod m20260221_000018_change_proposal_table;
mod m20260221_000019_baseline_set;
mod m20260221_000020_attachment_table;
mod m20260221_000021_soft_delete;
mod m20260221_000022_multi_value_enum;
mod m20260221_000023_seed_layout_scripts;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260221_000001_initial_schema::Migration),
            Box::new(m20260221_000002_seed_data::Migration),
            Box::new(m20260221_000003_fts_index::Migration),
            Box::new(m20260221_000004_history_preserve_on_delete::Migration),
            Box::new(m20260221_000005_add_level_column::Migration),
            Box::new(m20260221_000006_add_review_columns::Migration),
            Box::new(m20260221_000007_content_hash_review::Migration),
            Box::new(m20260221_000008_scoped_suspect_links::Migration),
            Box::new(m20260221_000009_object_classification::Migration),
            Box::new(m20260221_000010_script_table::Migration),
            Box::new(m20260221_000011_add_module_config::Migration),
            Box::new(m20260221_000012_object_references::Migration),
            Box::new(m20260221_000013_view_table::Migration),
            Box::new(m20260221_000014_object_type_table::Migration),
            Box::new(m20260221_000015_comment_table::Migration),
            Box::new(m20260221_000016_app_user_table::Migration),
            Box::new(m20260221_000017_review_tables::Migration),
            Box::new(m20260221_000018_change_proposal_table::Migration),
            Box::new(m20260221_000019_baseline_set::Migration),
            Box::new(m20260221_000020_attachment_table::Migration),
            Box::new(m20260221_000021_soft_delete::Migration),
            Box::new(m20260221_000022_multi_value_enum::Migration),
            Box::new(m20260221_000023_seed_layout_scripts::Migration),
        ]
    }
}
