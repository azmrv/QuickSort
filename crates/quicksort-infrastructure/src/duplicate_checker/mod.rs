//! Infrastructure implementations for duplicate file detection.

pub mod adapter;
pub mod content_checker;
pub mod name_checker;
pub mod size_checker;

pub use adapter::DuplicateDetectionAdapter;
pub use content_checker::ContentChecker;
pub use name_checker::NameChecker;
pub use size_checker::SizeChecker;
