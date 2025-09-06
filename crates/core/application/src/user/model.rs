use core_domain::user::model::User;
use secrecy::SecretString;

pub struct UserDto {
    pub id: i64,
    pub email: String,
    pub nickname: Option<String>,
    pub favourites_sync_timestamp: Option<i64>,
    pub history_sync_timestamp: Option<i64>,
}

impl From<User> for UserDto {
    fn from(value: User) -> Self {
        UserDto {
            id: value.id,
            email: value.email,
            nickname: value.nickname,
            favourites_sync_timestamp: value.favourites_sync_timestamp,
            history_sync_timestamp: value.history_sync_timestamp,
        }
    }
}

pub struct UserInput {
    pub email: String,
    pub password: SecretString,
    pub nickname: Option<String>,
}
