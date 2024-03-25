pub(crate) mod generate_random_gif_comment;
pub(crate) mod handle_issue_comment_event;
pub(crate) mod post_welcome_comment;

pub use generate_random_gif_comment::GenerateRandomGifCommentInterface;
pub use handle_issue_comment_event::HandleIssueCommentEventInterface;
pub use post_welcome_comment::PostWelcomeCommentInterface;

#[cfg(any(test, feature = "testkit"))]
pub use self::{
    generate_random_gif_comment::MockGenerateRandomGifCommentInterface,
    handle_issue_comment_event::MockHandleIssueCommentEventInterface,
    post_welcome_comment::MockPostWelcomeCommentInterface,
};
