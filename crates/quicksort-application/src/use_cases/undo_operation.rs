use crate::dtos::OperationResult;
use crate::errors::UseCaseError;
use crate::ports::inbound::UndoOperation;
use crate::ports::outbound::{FileSystem, OperationRepository};
use async_trait::async_trait;
use quicksort_domain::{Operation, OperationId, OperationState, OperationType};

pub struct UndoOperationUseCase {
    operation_repo: Box<dyn OperationRepository>,
    file_system: Box<dyn FileSystem>,
}

impl UndoOperationUseCase {
    pub fn new(
        operation_repo: Box<dyn OperationRepository>,
        file_system: Box<dyn FileSystem>,
    ) -> Self {
        Self {
            operation_repo,
            file_system,
        }
    }
}

#[async_trait]
impl UndoOperation for UndoOperationUseCase {
    async fn undo(&self, operation_id: OperationId) -> Result<OperationResult, UseCaseError> {
        let mut op = self
            .operation_repo
            .find_by_id(&operation_id)
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?
            .ok_or_else(|| UseCaseError::OperationNotFound(operation_id.to_string()))?;

        if !matches!(op.state, OperationState::Completed { .. }) {
            return Err(UseCaseError::UndoNotPossible(
                "Only completed operations can be undone".to_string(),
            ));
        }

        match op.operation_type {
            OperationType::Move => self.undo_move(&mut op).await?,
            OperationType::Copy => self.undo_copy(&mut op).await?,
            OperationType::Delete => self.undo_delete(&mut op).await?,
            OperationType::Rename => self.undo_rename(&mut op).await?,
        }

        op.mark_undone()
            .map_err(|e| UseCaseError::Domain(e.to_string()))?;

        self.operation_repo
            .save(&op)
            .await
            .map_err(|e| UseCaseError::RepositoryError(e.to_string()))?;

        Ok(OperationResult {
            operation_id: op.id,
            state: OperationState::Undone,
            processed_files: op.source_paths.len() as u32,
            bytes_moved: 0,
        })
    }
}

impl UndoOperationUseCase {
    async fn undo_move(&self, op: &mut Operation) -> Result<(), UseCaseError> {
        let target_folder = op.target_folder_path.as_ref().ok_or_else(|| {
            UseCaseError::UndoNotPossible("No target folder for Move".to_string())
        })?;

        for source_path in &op.source_paths {
            let file_name = source_path.file_name().ok_or_else(|| {
                UseCaseError::UndoNotPossible("Invalid source file name".to_string())
            })?;

            let target_path = target_folder.join(file_name);

            // If the file was moved out of the target folder by hand since the
            // original operation, there is nothing to restore. Skip it instead
            // of failing the whole undo; the operation is still marked Undone.
            if !self
                .file_system
                .exists(&target_path)
                .await
                .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?
            {
                continue;
            }

            self.file_system
                .rename_file(&target_path, source_path)
                .await
                .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
        }

        Ok(())
    }

    async fn undo_copy(&self, op: &mut Operation) -> Result<(), UseCaseError> {
        let target_folder = op.target_folder_path.as_ref().ok_or_else(|| {
            UseCaseError::UndoNotPossible("No target folder for Copy".to_string())
        })?;

        for source_path in &op.source_paths {
            let file_name = source_path.file_name().ok_or_else(|| {
                UseCaseError::UndoNotPossible("Invalid source file name".to_string())
            })?;

            let target_path = target_folder.join(file_name);

            if self
                .file_system
                .exists(&target_path)
                .await
                .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?
            {
                self.file_system
                    .delete_file(&target_path)
                    .await
                    .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
            }
        }

        Ok(())
    }

    async fn undo_delete(&self, _op: &mut Operation) -> Result<(), UseCaseError> {
        Err(UseCaseError::UndoNotPossible(
            "Undo of Delete operation requires trash can implementation".to_string(),
        ))
    }

    async fn undo_rename(&self, op: &mut Operation) -> Result<(), UseCaseError> {
        let target_paths = op.target_paths.as_ref().ok_or_else(|| {
            UseCaseError::UndoNotPossible("No target paths for Rename".to_string())
        })?;

        for (old_path, new_path) in op.source_paths.iter().zip(target_paths.iter()) {
            if !self
                .file_system
                .exists(new_path)
                .await
                .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?
            {
                return Err(UseCaseError::UndoNotPossible(format!(
                    "File with new name no longer exists: {}",
                    new_path.display()
                )));
            }

            self.file_system
                .rename_file(new_path, old_path)
                .await
                .map_err(|e| UseCaseError::FileSystemError(e.to_string()))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use quicksort_domain::{AbsolutePath, OperationId};
    use std::collections::HashSet;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    struct MockOperationRepository {
        operation: Mutex<Option<Operation>>,
        saved: Mutex<Vec<Operation>>,
    }

    impl MockOperationRepository {
        fn new(operation: Operation) -> Self {
            Self {
                operation: Mutex::new(Some(operation)),
                saved: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl OperationRepository for MockOperationRepository {
        async fn find_by_id(&self, id: &OperationId) -> Result<Option<Operation>, UseCaseError> {
            Ok(self
                .operation
                .lock()
                .unwrap()
                .clone()
                .filter(|op| &op.id == id))
        }

        async fn save(&self, operation: &Operation) -> Result<(), UseCaseError> {
            self.saved.lock().unwrap().push(operation.clone());
            Ok(())
        }

        async fn delete(&self, _id: &OperationId) -> Result<(), UseCaseError> {
            unimplemented!("not needed by undo tests")
        }

        async fn load_all(&self) -> Result<Vec<Operation>, UseCaseError> {
            Ok(self.saved.lock().unwrap().clone())
        }

        async fn clear(&self) -> Result<(), UseCaseError> {
            unimplemented!("not needed by undo tests")
        }
    }

    #[derive(Clone)]
    struct MockFileSystem {
        existing: Arc<Mutex<HashSet<PathBuf>>>,
        renamed: Arc<Mutex<Vec<(PathBuf, PathBuf)>>>,
    }

    impl MockFileSystem {
        fn new(existing: Vec<PathBuf>) -> Self {
            Self {
                existing: Arc::new(Mutex::new(existing.into_iter().collect())),
                renamed: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn renamed_count(&self) -> usize {
            self.renamed.lock().unwrap().len()
        }

        fn to_pathbuf(path: &AbsolutePath) -> PathBuf {
            PathBuf::from(path.to_string_lossy().as_ref())
        }
    }

    #[async_trait]
    impl FileSystem for MockFileSystem {
        async fn exists(&self, path: &AbsolutePath) -> Result<bool, UseCaseError> {
            Ok(self
                .existing
                .lock()
                .unwrap()
                .contains(&Self::to_pathbuf(path)))
        }

        async fn rename_file(
            &self,
            from: &AbsolutePath,
            to: &AbsolutePath,
        ) -> Result<(), UseCaseError> {
            self.renamed
                .lock()
                .unwrap()
                .push((Self::to_pathbuf(from), Self::to_pathbuf(to)));
            Ok(())
        }

        async fn get_file_size(&self, _path: &AbsolutePath) -> Result<u64, UseCaseError> {
            unimplemented!("not needed by undo tests")
        }

        async fn move_file(
            &self,
            _from: &AbsolutePath,
            _to: &AbsolutePath,
        ) -> Result<u64, UseCaseError> {
            unimplemented!("not needed by undo tests")
        }

        async fn copy_file(
            &self,
            _from: &AbsolutePath,
            _to: &AbsolutePath,
        ) -> Result<u64, UseCaseError> {
            unimplemented!("not needed by undo tests")
        }

        async fn delete_file(&self, _path: &AbsolutePath) -> Result<(), UseCaseError> {
            unimplemented!("not needed by undo tests")
        }

        async fn is_dir(&self, _path: &AbsolutePath) -> Result<bool, UseCaseError> {
            unimplemented!("not needed by undo tests")
        }

        async fn copy_tree(
            &self,
            _from: &AbsolutePath,
            _to: &AbsolutePath,
        ) -> Result<u64, UseCaseError> {
            unimplemented!("not needed by undo tests")
        }

        async fn move_tree(
            &self,
            _from: &AbsolutePath,
            _to: &AbsolutePath,
        ) -> Result<u64, UseCaseError> {
            unimplemented!("not needed by undo tests")
        }
    }

    fn completed_move_op(source: &[&str], target: &str) -> Operation {
        let src: Vec<AbsolutePath> = source
            .iter()
            .map(|s| AbsolutePath::new(s).unwrap())
            .collect();
        let tgt = AbsolutePath::new(target).unwrap();
        let mut op = Operation::new_move(src, tgt, Utc::now());
        op.start().unwrap();
        op.complete(source.len() as u32, 0).unwrap();
        op
    }

    #[tokio::test]
    async fn undo_move_restores_existing_file() {
        let op = completed_move_op(&["C:\\src\\a.txt"], "C:\\dst");
        let target = op.target_folder_path.clone().unwrap();
        let file_name = op.source_paths[0].file_name().unwrap();
        let existing = target.join(file_name);

        let repo = MockOperationRepository::new(op.clone());
        let fs = MockFileSystem::new(vec![PathBuf::from(existing.to_string_lossy().as_ref())]);
        let fs_check = fs.clone();
        let use_case = UndoOperationUseCase::new(Box::new(repo), Box::new(fs));

        let result = use_case.undo(op.id.clone()).await.unwrap();

        assert_eq!(result.state, OperationState::Undone);
        assert_eq!(fs_check.renamed_count(), 1);
    }

    #[tokio::test]
    async fn undo_move_skips_missing_target_file() {
        let op = completed_move_op(&["C:\\src\\a.txt"], "C:\\dst");

        let repo = MockOperationRepository::new(op.clone());
        let fs = MockFileSystem::new(vec![]);
        let fs_check = fs.clone();
        let use_case = UndoOperationUseCase::new(Box::new(repo), Box::new(fs));

        let result = use_case.undo(op.id.clone()).await.unwrap();

        assert_eq!(result.state, OperationState::Undone);
        assert_eq!(fs_check.renamed_count(), 0);
    }
}
