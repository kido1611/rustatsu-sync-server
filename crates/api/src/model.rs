use core_application::user::model::UserDto;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub nickname: Option<String>,
}

impl From<UserDto> for User {
    fn from(value: UserDto) -> Self {
        User {
            id: value.id,
            email: value.email,
            nickname: value.nickname,
        }
    }
}
