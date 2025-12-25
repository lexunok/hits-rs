pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251202_065032_create_user_table::Migration),
            Box::new(m20251205_050148_create_invitation_table::Migration),
            Box::new(m20251211_060650_create_verification_code_table::Migration),
            Box::new(m20251218_060739_create_company_table::Migration),
            Box::new(m20251221_103728_create_skill_table::Migration),
            Box::new(m20251222_064711_create_groups_table::Migration),
            Box::new(m20251225_134356_create_idea_table::Migration),
            Box::new(m20251225_134357_create_idea_checked_table::Migration),
        ]
    }
}
mod m20251202_065032_create_user_table;
mod m20251205_050148_create_invitation_table;
mod m20251211_060650_create_verification_code_table;
mod m20251218_060739_create_company_table;
mod m20251221_103728_create_skill_table;
mod m20251222_064711_create_groups_table;
mod m20251225_134356_create_idea_table;
mod m20251225_134357_create_idea_checked_table;
