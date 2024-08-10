mod index;
mod show;

pub use index::{get_manga_route, get_manga_tags_by_manga_id, Manga, Tag};
pub use show::get_manga_id_route;
