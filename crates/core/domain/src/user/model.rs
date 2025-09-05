pub struct User {
    pub id: i64,
    pub email: String,
    pub password: String,
    pub nickname: Option<String>,
    pub favourites_sync_timestamp: Option<i64>,
    pub history_sync_timestamp: Option<i64>,
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
