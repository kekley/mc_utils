#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
pub mod block;
pub mod block_display;
pub mod block_element;
pub mod block_face;
pub mod block_models;
pub mod block_states;
pub mod block_texture;
pub mod multipart;
pub mod resource;
pub mod resource_error;
pub mod utils;
pub mod variant;
