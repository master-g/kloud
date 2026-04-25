//! Environment variables

/// Loads environment variables from `.env` and `.env.local` files.
pub fn load_env() {
    dotenvy::from_path(std::path::Path::new(".env")).ok();
    dotenvy::from_path(std::path::Path::new(".env.local")).ok();
}
