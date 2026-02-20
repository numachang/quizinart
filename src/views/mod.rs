pub mod account;
pub mod components;
pub mod homepage;
pub mod layout;
pub mod quiz;

// Re-export commonly used functions from layout
pub use layout::{page, page_with_user, render, titled};
