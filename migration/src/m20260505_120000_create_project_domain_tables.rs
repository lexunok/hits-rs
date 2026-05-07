use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                ALTER TABLE "project"
                ADD COLUMN IF NOT EXISTS "report" text,
                ADD COLUMN IF NOT EXISTS "start_date" date NOT NULL DEFAULT CURRENT_DATE,
                ADD COLUMN IF NOT EXISTS "finish_date" date
                "#,
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProjectMember::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ProjectMember::ProjectId).uuid().not_null())
                    .col(ColumnDef::new(ProjectMember::UserId).uuid().not_null())
                    .col(ColumnDef::new(ProjectMember::TeamId).uuid())
                    .col(
                        ColumnDef::new(ProjectMember::ProjectRole)
                            .string()
                            .not_null()
                            .default("MEMBER"),
                    )
                    .col(
                        ColumnDef::new(ProjectMember::StartDate)
                            .date()
                            .not_null()
                            .default(Expr::cust("CURRENT_DATE")),
                    )
                    .col(ColumnDef::new(ProjectMember::FinishDate).date())
                    .primary_key(
                        Index::create()
                            .col(ProjectMember::ProjectId)
                            .col(ProjectMember::UserId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProjectMember::Table, ProjectMember::ProjectId)
                            .to(Project::Table, Project::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProjectMember::Table, ProjectMember::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProjectMember::Table, ProjectMember::TeamId)
                            .to(Team::Table, Team::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProjectMarks::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ProjectMarks::ProjectId).uuid().not_null())
                    .col(ColumnDef::new(ProjectMarks::UserId).uuid().not_null())
                    .col(ColumnDef::new(ProjectMarks::Mark).double())
                    .primary_key(
                        Index::create()
                            .col(ProjectMarks::ProjectId)
                            .col(ProjectMarks::UserId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProjectMarks::Table, ProjectMarks::ProjectId)
                            .to(Project::Table, Project::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProjectMarks::Table, ProjectMarks::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Tag::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Tag::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Tag::Name).text().not_null())
                    .col(ColumnDef::new(Tag::Color).text().not_null())
                    .col(
                        ColumnDef::new(Tag::Confirmed)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Tag::CreatorId).uuid())
                    .col(ColumnDef::new(Tag::UpdaterId).uuid())
                    .col(ColumnDef::new(Tag::DeleterId).uuid())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Tag::Table, Tag::CreatorId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Tag::Table, Tag::UpdaterId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Tag::Table, Tag::DeleterId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Sprint::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Sprint::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Sprint::ProjectId).uuid().not_null())
                    .col(ColumnDef::new(Sprint::Name).text().not_null())
                    .col(ColumnDef::new(Sprint::Goal).text().not_null())
                    .col(ColumnDef::new(Sprint::Report).text())
                    .col(
                        ColumnDef::new(Sprint::StartDate)
                            .date()
                            .not_null()
                            .default(Expr::cust("CURRENT_DATE")),
                    )
                    .col(ColumnDef::new(Sprint::FinishDate).date())
                    .col(ColumnDef::new(Sprint::WorkingHours).big_integer())
                    .col(
                        ColumnDef::new(Sprint::Status)
                            .string()
                            .not_null()
                            .default("ACTIVE"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Sprint::Table, Sprint::ProjectId)
                            .to(Project::Table, Project::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Task::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Task::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Task::SprintId).uuid())
                    .col(ColumnDef::new(Task::ProjectId).uuid().not_null())
                    .col(ColumnDef::new(Task::Position).integer())
                    .col(ColumnDef::new(Task::Name).text().not_null())
                    .col(ColumnDef::new(Task::Description).text())
                    .col(ColumnDef::new(Task::LeaderComment).text())
                    .col(ColumnDef::new(Task::ExecutorComment).text())
                    .col(ColumnDef::new(Task::InitiatorId).uuid())
                    .col(ColumnDef::new(Task::ExecutorId).uuid())
                    .col(ColumnDef::new(Task::WorkHour).double())
                    .col(
                        ColumnDef::new(Task::StartDate)
                            .date()
                            .not_null()
                            .default(Expr::cust("CURRENT_DATE")),
                    )
                    .col(ColumnDef::new(Task::FinishDate).date())
                    .col(ColumnDef::new(Task::Status).string())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Task::Table, Task::SprintId)
                            .to(Sprint::Table, Sprint::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Task::Table, Task::ProjectId)
                            .to(Project::Table, Project::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Task::Table, Task::InitiatorId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Task::Table, Task::ExecutorId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TaskTag::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TaskTag::TaskId).uuid().not_null())
                    .col(ColumnDef::new(TaskTag::TagId).uuid().not_null())
                    .primary_key(Index::create().col(TaskTag::TaskId).col(TaskTag::TagId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(TaskTag::Table, TaskTag::TaskId)
                            .to(Task::Table, Task::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TaskTag::Table, TaskTag::TagId)
                            .to(Tag::Table, Tag::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TaskHistory::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TaskHistory::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(TaskHistory::TaskId).uuid().not_null())
                    .col(ColumnDef::new(TaskHistory::SprintId).uuid())
                    .col(ColumnDef::new(TaskHistory::Status).string())
                    .col(ColumnDef::new(TaskHistory::ExecutorId).uuid())
                    .foreign_key(
                        ForeignKey::create()
                            .from(TaskHistory::Table, TaskHistory::TaskId)
                            .to(Task::Table, Task::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TaskHistory::Table, TaskHistory::SprintId)
                            .to(Sprint::Table, Sprint::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TaskHistory::Table, TaskHistory::ExecutorId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TaskMovementLog::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TaskMovementLog::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(TaskMovementLog::TaskId).uuid().not_null())
                    .col(ColumnDef::new(TaskMovementLog::ExecutorId).uuid())
                    .col(ColumnDef::new(TaskMovementLog::UserId).uuid())
                    .col(
                        ColumnDef::new(TaskMovementLog::StartDate)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(TaskMovementLog::EndDate).timestamp_with_time_zone())
                    .col(ColumnDef::new(TaskMovementLog::Status).string())
                    .foreign_key(
                        ForeignKey::create()
                            .from(TaskMovementLog::Table, TaskMovementLog::TaskId)
                            .to(Task::Table, Task::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TaskMovementLog::Table, TaskMovementLog::ExecutorId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TaskMovementLog::Table, TaskMovementLog::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(SprintMark::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SprintMark::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(SprintMark::ProjectId).uuid().not_null())
                    .col(ColumnDef::new(SprintMark::SprintId).uuid().not_null())
                    .col(ColumnDef::new(SprintMark::UserId).uuid().not_null())
                    .col(ColumnDef::new(SprintMark::ProjectRole).string().not_null())
                    .col(ColumnDef::new(SprintMark::Mark).double())
                    .col(ColumnDef::new(SprintMark::CountCompletedTasks).integer())
                    .foreign_key(
                        ForeignKey::create()
                            .from(SprintMark::Table, SprintMark::ProjectId)
                            .to(Project::Table, Project::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SprintMark::Table, SprintMark::SprintId)
                            .to(Sprint::Table, Sprint::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(SprintMark::Table, SprintMark::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Team {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Project {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum ProjectMember {
    Table,
    ProjectId,
    UserId,
    TeamId,
    ProjectRole,
    StartDate,
    FinishDate,
}

#[derive(DeriveIden)]
enum ProjectMarks {
    Table,
    ProjectId,
    UserId,
    Mark,
}

#[derive(DeriveIden)]
enum Tag {
    Table,
    Id,
    Name,
    Color,
    Confirmed,
    CreatorId,
    UpdaterId,
    DeleterId,
}

#[derive(DeriveIden)]
enum Sprint {
    Table,
    Id,
    ProjectId,
    Name,
    Goal,
    Report,
    StartDate,
    FinishDate,
    WorkingHours,
    Status,
}

#[derive(DeriveIden)]
enum Task {
    Table,
    Id,
    SprintId,
    ProjectId,
    Position,
    Name,
    Description,
    LeaderComment,
    ExecutorComment,
    InitiatorId,
    ExecutorId,
    WorkHour,
    StartDate,
    FinishDate,
    Status,
}

#[derive(DeriveIden)]
enum TaskTag {
    Table,
    TaskId,
    TagId,
}

#[derive(DeriveIden)]
enum TaskHistory {
    Table,
    Id,
    TaskId,
    SprintId,
    Status,
    ExecutorId,
}

#[derive(DeriveIden)]
enum TaskMovementLog {
    Table,
    Id,
    TaskId,
    ExecutorId,
    UserId,
    StartDate,
    EndDate,
    Status,
}

#[derive(DeriveIden)]
enum SprintMark {
    Table,
    Id,
    ProjectId,
    SprintId,
    UserId,
    ProjectRole,
    Mark,
    CountCompletedTasks,
}
