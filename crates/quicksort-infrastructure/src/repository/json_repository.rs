// // quicksort-infrastructure/src/operation_repository/json_repository.rs
//
// use anyhow::{Context, Result};
// use quicksort_application::ports::OperationRepository;
// use quicksort_domain::{Operation, OperationStatus};
// use std::path::PathBuf;
// use tokio::fs;
//
// pub struct JsonOperationRepository {
//     path: PathBuf,
// }
//
// impl JsonOperationRepository {
//     pub fn new() -> Result<Self> {
//         let dir = directories::ProjectDirs::from("com", "quicksort", "QuickSort")
//             .map(|d| d.config_dir().to_path_buf())
//             .unwrap_or_else(|| PathBuf::from("."));
//         fs::create_dir_all(&dir).await?;
//         let path = dir.join("operations.json");
//         Ok(Self { path })
//     }
//
//     async fn read_all(&self) -> Result<Vec<Operation>> {
//         if !self.path.exists() {
//             return Ok(vec![]);
//         }
//         let data = fs::read_to_string(&self.path).await?;
//         let ops = serde_json::from_str(&data).unwrap_or_default();
//         Ok(ops)
//     }
//
//     async fn write_all(&self, ops: &[Operation]) -> Result<()> {
//         let json = serde_json::to_string_pretty(ops)?;
//         fs::write(&self.path, json).await?;
//         Ok(())
//     }
// }
//
// #[async_trait::async_trait]
// impl OperationRepository for JsonOperationRepository {
//     async fn save(&self, operation: &Operation) -> Result<()> {
//         let mut ops = self.read_all().await?;
//         if let Some(pos) = ops.iter().position(|o| o.id == operation.id) {
//             ops[pos] = operation.clone();
//         } else {
//             ops.push(operation.clone());
//         }
//         self.write_all(&ops).await
//     }
//
//     async fn load_all(&self) -> Result<Vec<Operation>> {
//         self.read_all().await
//     }
//
//     async fn load_last(&self) -> Result<Option<Operation>> {
//         let ops = self.read_all().await?;
//         // Сортируем по timestamp, берём последнюю
//         let mut sorted = ops;
//         sorted.sort_by_key(|o| o.timestamp);
//         Ok(sorted.pop())
//     }
//
//     async fn update_status(&self, id: &str, status: OperationStatus) -> Result<()> {
//         let mut ops = self.read_all().await?;
//         if let Some(op) = ops.iter_mut().find(|o| o.id == id) {
//             op.status = status;
//             self.write_all(&ops).await?;
//         }
//         Ok(())
//     }
//
//     async fn delete_all(&self) -> Result<()> {
//         if self.path.exists() {
//             fs::remove_file(&self.path).await?;
//         }
//         Ok(())
//     }
// }