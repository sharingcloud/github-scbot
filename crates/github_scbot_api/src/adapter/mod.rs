//! Adapter

mod dummy;
mod github;
mod interface;

pub use dummy::DummyAPIAdapter;
pub use github::GithubAPIAdapter;
pub use interface::{
    GhReviewApi, GhReviewStateApi, GifFormat, GifObject, GifResponse, IAPIAdapter, MediaObject,
};
