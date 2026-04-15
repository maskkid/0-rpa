//! Database schema migration.

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240415_000001_init_tables"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create plugins table
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("plugins"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("name")).string().not_null())
                    .col(ColumnDef::new(Alias::new("version")).string().not_null())
                    .col(ColumnDef::new(Alias::new("description")).string().null())
                    .col(ColumnDef::new(Alias::new("author")).string().null())
                    .col(ColumnDef::new(Alias::new("zip_path")).string().not_null())
                    .col(ColumnDef::new(Alias::new("installed_at")).date_time().not_null())
                    .col(
                        ColumnDef::new(Alias::new("is_enabled"))
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .to_owned(),
            )
            .await?;

        // Create workflows table
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("workflows"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("plugin_id")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("name")).string().not_null())
                    .col(ColumnDef::new(Alias::new("description")).string().null())
                    .col(ColumnDef::new(Alias::new("script")).text().not_null())
                    .col(ColumnDef::new(Alias::new("created_at")).date_time().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).date_time().not_null())
                    .col(ColumnDef::new(Alias::new("last_run_at")).date_time().null())
                    .col(
                        ColumnDef::new(Alias::new("run_count"))
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("workflows"), Alias::new("plugin_id"))
                            .to(Alias::new("plugins"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create task_history table
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("task_history"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("workflow_id")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("status")).string().not_null())
                    .col(ColumnDef::new(Alias::new("started_at")).date_time().not_null())
                    .col(ColumnDef::new(Alias::new("finished_at")).date_time().null())
                    .col(ColumnDef::new(Alias::new("logs")).text().not_null())
                    .col(ColumnDef::new(Alias::new("error_message")).string().null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("task_history"), Alias::new("workflow_id"))
                            .to(Alias::new("workflows"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create settings table (key-value store)
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("settings"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("key"))
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("value")).string().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).date_time().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("task_history")).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Alias::new("workflows")).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Alias::new("plugins")).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Alias::new("settings")).to_owned())
            .await?;
        Ok(())
    }
}