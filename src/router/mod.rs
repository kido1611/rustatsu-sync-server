mod auth;
mod index;
mod manga;
mod me;
mod resources;

pub use auth::auth;
pub use index::index;
pub use manga::{get_manga, get_manga_by_id};
pub use me::get_user;
pub use resources::{
    get_favourites_package, get_history_package, post_favourites, post_history_package,
};
