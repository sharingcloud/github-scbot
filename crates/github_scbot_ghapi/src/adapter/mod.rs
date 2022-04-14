//! Adapter

mod github;
mod interface;

pub use github::GithubAPIAdapter;
pub use interface::{
    ApiService, GhReviewApi, GhReviewStateApi, GifFormat, GifObject, GifResponse, MediaObject,
    MockApiService,
};
