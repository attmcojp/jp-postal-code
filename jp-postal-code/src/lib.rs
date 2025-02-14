pub mod config;
pub mod infra;
pub mod repo;
pub mod usecase;

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../migrations");
