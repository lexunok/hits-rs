use sea_orm_migration::prelude::*;

use crate::m20251202_065032_create_user_table::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Group::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Group::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Group::Name).string().not_null())
                    .col(
                        ColumnDef::new(Group::Roles)
                            .array(ColumnType::String(StringLen::None))
                            .not_null()
                    )                    
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(GroupMember::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GroupMember::GroupId).uuid().not_null())
                    .col(ColumnDef::new(GroupMember::UserId).uuid().not_null())
                    .primary_key(
                        Index::create()
                            .col(GroupMember::GroupId)
                            .col(GroupMember::UserId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(GroupMember::Table, GroupMember::GroupId)
                            .to(Group::Table, Group::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(GroupMember::Table, GroupMember::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Group {
    Table,
    Id,
    Name,
    Roles
}

#[derive(Iden)]
enum GroupMember {
    Table,
    GroupId,
    UserId,
}