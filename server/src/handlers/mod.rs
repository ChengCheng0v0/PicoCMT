pub mod add_comment;
pub mod get_top_comments;

pub async fn root() -> &'static str {
    "Hello World!"
}
