pub mod prompt;
pub mod trash;
pub mod verify;

pub use prompt::ConfirmPrompt;
pub use trash::{CleanStats, TrashCleaner};
pub use verify::PreDeleteVerifier;
