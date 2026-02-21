#![allow(
    clippy::wildcard_imports,
    clippy::too_many_lines,
    clippy::enum_variant_names
)]

use std::fmt::Write;

use sea_orm_migration::{prelude::*, schema::*, sea_orm::sea_query::SeaRc};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // --- Workspace ---
        manager
            .create_table(
                Table::create()
                    .table(Workspace::Table)
                    .if_not_exists()
                    .col(pk_uuid(Workspace::Id))
                    .col(string(Workspace::Name))
                    .col(text_null(Workspace::Description))
                    .col(
                        timestamp_with_time_zone(Workspace::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Workspace::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // --- Project ---
        manager
            .create_table(
                Table::create()
                    .table(Project::Table)
                    .if_not_exists()
                    .col(pk_uuid(Project::Id))
                    .col(uuid(Project::WorkspaceId))
                    .col(string(Project::Name))
                    .col(text_null(Project::Description))
                    .col(
                        timestamp_with_time_zone(Project::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Project::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Project::Table, Project::WorkspaceId)
                            .to(Workspace::Table, Workspace::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // --- Module ---
        manager
            .create_table(
                Table::create()
                    .table(Module::Table)
                    .if_not_exists()
                    .col(pk_uuid(Module::Id))
                    .col(uuid(Module::ProjectId))
                    .col(string(Module::Name))
                    .col(text_null(Module::Description))
                    .col(
                        timestamp_with_time_zone(Module::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Module::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Module::Table, Module::ProjectId)
                            .to(Project::Table, Project::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // --- AttributeDefinition ---
        manager
            .create_table(
                Table::create()
                    .table(AttributeDefinition::Table)
                    .if_not_exists()
                    .col(pk_uuid(AttributeDefinition::Id))
                    .col(uuid_null(AttributeDefinition::ModuleId))
                    .col(string(AttributeDefinition::Name))
                    .col(string(AttributeDefinition::DataType))
                    .col(text_null(AttributeDefinition::DefaultValue))
                    .col(json_binary_null(AttributeDefinition::EnumValues))
                    .col(
                        timestamp_with_time_zone(AttributeDefinition::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(AttributeDefinition::Table, AttributeDefinition::ModuleId)
                            .to(Module::Table, Module::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // --- Object ---
        manager
            .create_table(
                Table::create()
                    .table(Object::Table)
                    .if_not_exists()
                    .col(pk_uuid(Object::Id))
                    .col(uuid(Object::ModuleId))
                    .col(uuid_null(Object::ParentId))
                    .col(integer(Object::Position).default(0))
                    .col(text_null(Object::Heading))
                    .col(text_null(Object::Body))
                    .col(json_binary_null(Object::Attributes))
                    .col(integer(Object::CurrentVersion).default(1))
                    .col(
                        timestamp_with_time_zone(Object::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Object::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Object::Table, Object::ModuleId)
                            .to(Module::Table, Module::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Object::Table, Object::ParentId)
                            .to(Object::Table, Object::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // GIN index on object.attributes
        manager
            .create_index(
                Index::create()
                    .name("idx_object_attributes_gin")
                    .table(Object::Table)
                    .col(Object::Attributes)
                    .index_type(IndexType::Custom(SeaRc::new(GinIndex)))
                    .to_owned(),
            )
            .await?;

        // --- ObjectHistory ---
        manager
            .create_table(
                Table::create()
                    .table(ObjectHistory::Table)
                    .if_not_exists()
                    .col(
                        big_integer(ObjectHistory::Id)
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(uuid(ObjectHistory::ObjectId))
                    .col(uuid(ObjectHistory::ModuleId))
                    .col(integer(ObjectHistory::Version))
                    .col(json_binary_null(ObjectHistory::AttributeValues))
                    .col(text_null(ObjectHistory::Heading))
                    .col(text_null(ObjectHistory::Body))
                    .col(uuid_null(ObjectHistory::ChangedBy))
                    .col(
                        timestamp_with_time_zone(ObjectHistory::ChangedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(string(ObjectHistory::ChangeType))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ObjectHistory::Table, ObjectHistory::ObjectId)
                            .to(Object::Table, Object::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // GIN index on object_history.attribute_values
        manager
            .create_index(
                Index::create()
                    .name("idx_object_history_attrs_gin")
                    .table(ObjectHistory::Table)
                    .col(ObjectHistory::AttributeValues)
                    .index_type(IndexType::Custom(SeaRc::new(GinIndex)))
                    .to_owned(),
            )
            .await?;

        // --- LinkType ---
        manager
            .create_table(
                Table::create()
                    .table(LinkType::Table)
                    .if_not_exists()
                    .col(pk_uuid(LinkType::Id))
                    .col(string(LinkType::Name))
                    .col(text_null(LinkType::Description))
                    .col(
                        timestamp_with_time_zone(LinkType::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // --- Link ---
        manager
            .create_table(
                Table::create()
                    .table(Link::Table)
                    .if_not_exists()
                    .col(pk_uuid(Link::Id))
                    .col(uuid(Link::SourceObjectId))
                    .col(uuid(Link::TargetObjectId))
                    .col(uuid(Link::LinkTypeId))
                    .col(json_binary_null(Link::Attributes))
                    .col(boolean(Link::Suspect).default(false))
                    .col(
                        timestamp_with_time_zone(Link::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Link::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Link::Table, Link::SourceObjectId)
                            .to(Object::Table, Object::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Link::Table, Link::TargetObjectId)
                            .to(Object::Table, Object::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Link::Table, Link::LinkTypeId)
                            .to(LinkType::Table, LinkType::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        // Indexes on link source/target
        manager
            .create_index(
                Index::create()
                    .name("idx_link_source")
                    .table(Link::Table)
                    .col(Link::SourceObjectId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_link_target")
                    .table(Link::Table)
                    .col(Link::TargetObjectId)
                    .to_owned(),
            )
            .await?;

        // --- Baseline ---
        manager
            .create_table(
                Table::create()
                    .table(Baseline::Table)
                    .if_not_exists()
                    .col(pk_uuid(Baseline::Id))
                    .col(uuid(Baseline::ModuleId))
                    .col(string(Baseline::Name))
                    .col(text_null(Baseline::Description))
                    .col(uuid_null(Baseline::CreatedBy))
                    .col(
                        timestamp_with_time_zone(Baseline::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(boolean(Baseline::Locked).default(true))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Baseline::Table, Baseline::ModuleId)
                            .to(Module::Table, Module::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // --- BaselineEntry ---
        manager
            .create_table(
                Table::create()
                    .table(BaselineEntry::Table)
                    .if_not_exists()
                    .col(uuid(BaselineEntry::BaselineId))
                    .col(uuid(BaselineEntry::ObjectId))
                    .col(integer(BaselineEntry::Version))
                    .primary_key(
                        Index::create()
                            .col(BaselineEntry::BaselineId)
                            .col(BaselineEntry::ObjectId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(BaselineEntry::Table, BaselineEntry::BaselineId)
                            .to(Baseline::Table, Baseline::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(BaselineEntry::Table, BaselineEntry::ObjectId)
                            .to(Object::Table, Object::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // --- User ---
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(pk_uuid(User::Id))
                    .col(string_uniq(User::Username))
                    .col(string(User::DisplayName))
                    .col(string_null(User::Email))
                    .col(
                        timestamp_with_time_zone(User::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(User::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let tables = [
            BaselineEntry::Table.into_table_ref(),
            Baseline::Table.into_table_ref(),
            Link::Table.into_table_ref(),
            LinkType::Table.into_table_ref(),
            ObjectHistory::Table.into_table_ref(),
            Object::Table.into_table_ref(),
            AttributeDefinition::Table.into_table_ref(),
            Module::Table.into_table_ref(),
            Project::Table.into_table_ref(),
            Workspace::Table.into_table_ref(),
            User::Table.into_table_ref(),
        ];

        for table in tables {
            manager
                .drop_table(Table::drop().table(table).if_exists().to_owned())
                .await?;
        }

        Ok(())
    }
}

// --- Custom index type ---

struct GinIndex;

impl Iden for GinIndex {
    fn unquoted(&self, s: &mut dyn Write) {
        let _ = write!(s, "GIN");
    }
}

// --- Iden enums ---

#[derive(DeriveIden)]
enum Workspace {
    Table,
    Id,
    Name,
    Description,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Project {
    Table,
    Id,
    WorkspaceId,
    Name,
    Description,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Module {
    Table,
    Id,
    ProjectId,
    Name,
    Description,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum AttributeDefinition {
    Table,
    Id,
    ModuleId,
    Name,
    DataType,
    DefaultValue,
    EnumValues,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Object {
    Table,
    Id,
    ModuleId,
    ParentId,
    Position,
    Heading,
    Body,
    Attributes,
    CurrentVersion,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ObjectHistory {
    Table,
    Id,
    ObjectId,
    ModuleId,
    Version,
    AttributeValues,
    Heading,
    Body,
    ChangedBy,
    ChangedAt,
    ChangeType,
}

#[derive(DeriveIden)]
enum LinkType {
    Table,
    Id,
    Name,
    Description,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Link {
    Table,
    Id,
    SourceObjectId,
    TargetObjectId,
    LinkTypeId,
    Attributes,
    Suspect,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Baseline {
    Table,
    Id,
    ModuleId,
    Name,
    Description,
    CreatedBy,
    CreatedAt,
    Locked,
}

#[derive(DeriveIden)]
enum BaselineEntry {
    Table,
    BaselineId,
    ObjectId,
    Version,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Username,
    DisplayName,
    Email,
    CreatedAt,
    UpdatedAt,
}
