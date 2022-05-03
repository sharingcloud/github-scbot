//! Adapter

mod github;
mod interface;

pub use github::GithubApiService;
pub use interface::{
    ApiService, GhReviewApi, GhReviewStateApi, GifFormat, GifObject, GifResponse, MediaObject,
    MockApiService,
};
