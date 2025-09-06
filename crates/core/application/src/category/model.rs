use core_domain::category::model::Category;

pub struct CategoryDto {
    pub id: i64,
    pub created_at: i64,
    pub sort_key: i32,
    pub title: String,
    pub order: String,
    pub deleted_at: i64,
    pub track: bool,
    pub show_in_lib: bool,
}

impl CategoryDto {
    pub fn to_category(self, user_id: i64) -> Category {
        Category {
            id: self.id,
            created_at: self.created_at,
            sort_key: self.sort_key,
            title: self.title,
            order: self.order,
            deleted_at: self.deleted_at,
            track: self.track,
            show_in_lib: self.show_in_lib,
            user_id,
        }
    }
}

impl From<Category> for CategoryDto {
    fn from(value: Category) -> Self {
        CategoryDto {
            id: value.id,
            created_at: value.created_at,
            sort_key: value.sort_key,
            title: value.title,
            order: value.order,
            deleted_at: value.deleted_at,
            track: value.track,
            show_in_lib: value.show_in_lib,
        }
    }
}
