//! Conflict resolution strategies for file operations.
//!
//! # Design Decisions
//! - Follows Teleport's ConflictForm pattern: Add (timestamp), Replace, Cancel
//! - Extended with QuickSort-specific strategies: Rename (unique name), Skip
//! - `ConflictContext` tracks user choice across batch operations
//! - Supports both interactive and non-interactive modes

use serde::{Deserialize, Serialize};

/// Resolution strategy when a target file already exists.
///
/// This is the Domain-level representation of conflict resolution,
/// distinct from `OverwritePolicy` which is a DTO-level concept.
/// The Domain entity captures the semantic intent, while the DTO
/// captures the technical implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConflictResolution {
    /// Skip the conflicting file entirely.
    /// The source file is left untouched.
    #[default]
    Skip,

    /// Add the file with a timestamp suffix to avoid collision.
    /// Example: "report.txt" → "report_2608211430.txt"
    /// Preserves the original file.
    AddWithTimestamp,

    /// Replace the existing file with the new one.
    /// **Warning:** This is destructive and the original is lost.
    Replace,

    /// Generate a unique name by appending a numeric suffix.
    /// Example: "report.txt" → "report (1).txt"
    /// Preserves the original file.
    Rename,

    /// Cancel the entire batch operation.
    /// No more files will be processed.
    Cancel,

    /// Prompt the user for a decision.
    /// In non-interactive mode, falls back to `AddWithTimestamp`.
    Ask,
}

impl ConflictResolution {
    /// Returns the fallback resolution for non-interactive mode.
    pub fn non_interactive_fallback(self) -> Self {
        match self {
            ConflictResolution::Ask => ConflictResolution::AddWithTimestamp,
            other => other,
        }
    }

    /// Returns true if the operation should continue after this resolution.
    pub fn should_continue(&self) -> bool {
        !matches!(self, ConflictResolution::Cancel)
    }

    /// Returns true if the source file should be processed.
    pub fn should_process(&self) -> bool {
        !matches!(self, ConflictResolution::Skip | ConflictResolution::Cancel)
    }
}

/// Context for conflict resolution across batch operations.
///
/// Remembers the user's choice so they don't have to answer
/// for every conflicting file. Inspired by Teleport's `ConflictContext`.
///
/// # Example
/// ```rust,ignore
/// let mut ctx = ConflictContext::new();
///
/// for file in files {
///     if has_conflict(&file) {
///         let resolution = ctx.resolve(|| ask_user(&file));
///         match resolution {
///             ConflictResolution::Replace => { /* overwrite */ }
///             ConflictResolution::Cancel => break,
///             _ => { /* handle other cases */ }
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ConflictContext {
    /// The remembered resolution, if any.
    remembered: Option<ConflictResolution>,

    /// Whether the user has made a choice.
    is_chosen: bool,

    /// Count of files processed with this context.
    files_processed: u32,

    /// Count of files skipped due to conflicts.
    files_skipped: u32,

    /// Count of files renamed to avoid conflicts.
    files_renamed: u32,

    /// Count of files overwritten.
    files_overwritten: u32,
}

impl ConflictContext {
    /// Create a new conflict context with no remembered choice.
    pub fn new() -> Self {
        Self {
            remembered: None,
            is_chosen: false,
            files_processed: 0,
            files_skipped: 0,
            files_renamed: 0,
            files_overwritten: 0,
        }
    }

    /// Create a context with a pre-set resolution (for batch operations).
    pub fn with_resolution(resolution: ConflictResolution) -> Self {
        Self {
            remembered: Some(resolution),
            is_chosen: true,
            files_processed: 0,
            files_skipped: 0,
            files_renamed: 0,
            files_overwritten: 0,
        }
    }

    /// Resolve a conflict, using the remembered choice or prompting the user.
    ///
    /// The `prompt` closure is called only if no choice has been remembered.
    /// It should show a dialog or prompt and return the user's choice.
    pub fn resolve(&mut self, prompt: impl FnOnce() -> ConflictResolution) -> ConflictResolution {
        if !self.is_chosen {
            let resolution = prompt();
            let effective = resolution.non_interactive_fallback();
            self.remembered = Some(effective);
            self.is_chosen = true;
            return effective;
        }

        let resolution = self.remembered.unwrap();
        match resolution {
            ConflictResolution::AddWithTimestamp
            | ConflictResolution::Rename
            | ConflictResolution::Replace => {
                self.files_processed += 1;
                match resolution {
                    ConflictResolution::Replace => self.files_overwritten += 1,
                    ConflictResolution::AddWithTimestamp | ConflictResolution::Rename => {
                        self.files_renamed += 1
                    }
                    _ => {}
                }
            }
            ConflictResolution::Skip => self.files_skipped += 1,
            ConflictResolution::Cancel => {}
            ConflictResolution::Ask => unreachable!("Ask should have been resolved"),
        }
        resolution
    }

    /// Force a specific resolution (for programmatic batch operations).
    pub fn set_resolution(&mut self, resolution: ConflictResolution) {
        self.remembered = Some(resolution);
        self.is_chosen = true;
    }

    /// Check if the user chose to cancel.
    pub fn is_cancelled(&self) -> bool {
        self.remembered == Some(ConflictResolution::Cancel)
    }

    /// Get statistics about this conflict context.
    pub fn stats(&self) -> ConflictStats {
        ConflictStats {
            files_processed: self.files_processed,
            files_skipped: self.files_skipped,
            files_renamed: self.files_renamed,
            files_overwritten: self.files_overwritten,
        }
    }
}

impl Default for ConflictContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about conflict resolution in a batch operation.
#[derive(Debug, Clone, Default)]
pub struct ConflictStats {
    pub files_processed: u32,
    pub files_skipped: u32,
    pub files_renamed: u32,
    pub files_overwritten: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_resolution_default() {
        assert_eq!(ConflictResolution::default(), ConflictResolution::Skip);
    }

    #[test]
    fn test_non_interactive_fallback() {
        assert_eq!(
            ConflictResolution::Ask.non_interactive_fallback(),
            ConflictResolution::AddWithTimestamp
        );
        assert_eq!(
            ConflictResolution::Replace.non_interactive_fallback(),
            ConflictResolution::Replace
        );
        assert_eq!(
            ConflictResolution::Skip.non_interactive_fallback(),
            ConflictResolution::Skip
        );
    }

    #[test]
    fn test_should_continue() {
        assert!(ConflictResolution::Skip.should_continue());
        assert!(ConflictResolution::Replace.should_continue());
        assert!(!ConflictResolution::Cancel.should_continue());
    }

    #[test]
    fn test_should_process() {
        assert!(ConflictResolution::Replace.should_process());
        assert!(ConflictResolution::Rename.should_process());
        assert!(!ConflictResolution::Skip.should_process());
        assert!(!ConflictResolution::Cancel.should_process());
    }

    #[test]
    fn test_context_new() {
        let ctx = ConflictContext::new();
        assert!(!ctx.is_chosen);
        assert!(!ctx.is_cancelled());
    }

    #[test]
    fn test_context_with_resolution() {
        let ctx = ConflictContext::with_resolution(ConflictResolution::Replace);
        assert!(ctx.is_chosen);
        assert_eq!(ctx.remembered, Some(ConflictResolution::Replace));
    }

    #[test]
    fn test_context_resolve_first_time() {
        let mut ctx = ConflictContext::new();
        let result = ctx.resolve(|| ConflictResolution::Replace);
        assert_eq!(result, ConflictResolution::Replace);
        assert!(ctx.is_chosen);
    }

    #[test]
    fn test_context_resolve_remembered() {
        let mut ctx = ConflictContext::with_resolution(ConflictResolution::Skip);
        let result = ctx.resolve(|| ConflictResolution::Replace);
        assert_eq!(result, ConflictResolution::Skip);
        assert_eq!(ctx.stats().files_skipped, 1);
    }

    #[test]
    fn test_context_cancel() {
        let ctx = ConflictContext::with_resolution(ConflictResolution::Cancel);
        assert!(ctx.is_cancelled());
    }
}
