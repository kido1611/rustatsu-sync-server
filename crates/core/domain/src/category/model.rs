pub struct Category {
    pub id: i64,
    pub created_at: i64,
    pub sort_key: i32,
    pub title: String,
    pub order: String,
    pub user_id: i64,
    pub track: bool,
    pub show_in_lib: bool,
    pub deleted_at: i64,
}
