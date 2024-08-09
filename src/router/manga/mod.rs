mod index;
mod show;

pub use index::{get_manga, get_manga_tags_by_manga_id, Manga, Tag};
pub use show::get_manga_by_id;
