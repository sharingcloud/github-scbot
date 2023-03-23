mod random_gif_from_query;

#[cfg(any(test, feature = "testkit"))]
pub use random_gif_from_query::MockRandomGifFromQueryUseCaseInterface;
pub use random_gif_from_query::{RandomGifFromQueryUseCase, RandomGifFromQueryUseCaseInterface};
