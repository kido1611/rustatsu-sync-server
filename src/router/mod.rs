mod auth;
mod index;
mod manga;
mod resources;

pub use auth::auth;
pub use index::index;
pub use manga::{get_manga, get_manga_by_id};
