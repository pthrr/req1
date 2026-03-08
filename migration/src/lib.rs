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
mod m20260221_000024_webhook_table;
mod m20260221_000025_module_publish_template;
mod m20260221_000026_lifecycle;
mod m20260221_000027_test_tables;
mod m20260221_000028_dependent_enums;
mod m20260221_000029_script_priority;
mod m20260221_000030_review_comment;
mod m20260221_000031_placeholder;
mod m20260221_000032_auth_tables;
mod m20260221_000033_diagram_table;
mod m20260221_000034_script_scheduling;
mod m20260221_000035_mentions_and_notifications;
mod m20260221_000036_e_signature;
mod m20260221_000037_docx_import_tracking;
mod m20260221_000038_dashboard_tables;
mod m20260221_000039_project_template;
mod m20260221_000040_seed_admin_user;

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
            Box::new(m20260221_000024_webhook_table::Migration),
            Box::new(m20260221_000025_module_publish_template::Migration),
            Box::new(m20260221_000026_lifecycle::Migration),
            Box::new(m20260221_000027_test_tables::Migration),
            Box::new(m20260221_000028_dependent_enums::Migration),
            Box::new(m20260221_000029_script_priority::Migration),
            Box::new(m20260221_000030_review_comment::Migration),
            Box::new(m20260221_000031_placeholder::Migration),
            Box::new(m20260221_000032_auth_tables::Migration),
            Box::new(m20260221_000033_diagram_table::Migration),
            Box::new(m20260221_000034_script_scheduling::Migration),
            Box::new(m20260221_000035_mentions_and_notifications::Migration),
            Box::new(m20260221_000036_e_signature::Migration),
            Box::new(m20260221_000037_docx_import_tracking::Migration),
            Box::new(m20260221_000038_dashboard_tables::Migration),
            Box::new(m20260221_000039_project_template::Migration),
            Box::new(m20260221_000040_seed_admin_user::Migration),
        ]
    }
}
