mod create;
mod index;

pub use create::{post_favourites, upsert_manga};
pub use index::{get_favourites_package, get_favourites_package_by_user};
