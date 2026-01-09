pub struct User {
    pub id: i64,
    pub email: String,
    pub password: String,
    pub nickname: Option<String>,
    pub favourites_sync_timestamp: Option<i64>,
    pub history_sync_timestamp: Option<i64>,
    pub password_reset_token_hash: Option<String>,
    pub password_reset_token_expires_at: Option<i64>,
}

pub struct UserCreate {
    pub email: String,
    pub password: String,
    pub nickname: Option<String>,
}

pub struct UserUpdateSyncTime {
    pub user_id: i64,
    pub time: i64,
}

pub struct UserPasswordReset {
    pub user_id: i64,
    pub token: String,
    pub expires_at: i64,
}

pub struct UserUpdatePassword {
    pub user_id: i64,
    pub password: String,
}
