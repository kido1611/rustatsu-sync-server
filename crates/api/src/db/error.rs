#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error("Database error")]
    DatabaseError(sqlx::Error),
    #[error("Record not found")]
    NotFound,
}
