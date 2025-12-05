pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251202_065032_create_user_table::Migration),
            Box::new(m20251205_050148_create_invitation_table::Migration),
        ]
    }
}

mod m20251202_065032_create_user_table;
mod m20251205_050148_create_invitation_table;