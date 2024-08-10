mod auth;
mod index;
mod manga;
mod me;
mod resources;

pub use auth::post_auth_route;
pub use index::index;
pub use manga::{get_manga_id_route, get_manga_route};
pub use me::get_me_route;
pub use resources::{
    get_favourites_route, get_history_route, post_favourites_route, post_history_route,
};
