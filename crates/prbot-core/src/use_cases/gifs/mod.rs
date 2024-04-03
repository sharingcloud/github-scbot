pub(crate) mod random_gif_from_query;

#[cfg(any(test, feature = "testkit"))]
pub use random_gif_from_query::MockRandomGifFromQueryInterface;
pub use random_gif_from_query::RandomGifFromQueryInterface;
