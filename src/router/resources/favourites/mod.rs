mod create;
mod index;

pub use create::{post_favourites_route, upsert_manga};
pub use index::get_favourites_route;
