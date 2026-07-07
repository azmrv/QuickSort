//! Named Pipe server for receiving commands from Windows Shell Extension DLL.

use std::sync::Arc;
use std::time::Duration;
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;
use serde_json;
use tracing::{info, error, warn};

use quicksort_application::dtos::PipeCommand;
use quicksort_application::ports::inbound::ExecuteOperation;

const PIPE_NAME: &str = r"\\.\pipe\quicksort_cmd";
const BUFFER_SIZE: usize = 4096;
const TIMEOUT_SECS: u64 = 5;

/// Starts the Named Pipe server in a background task.
pub fn start_pipe_server(execute_use_case: Arc<dyn ExecuteOperation>) {
    tokio::spawn(async move {
        info!("Starting Named Pipe server on {}", PIPE_NAME);
        loop {
            match create_pipe_server().await {
                Ok(mut server) => {
                    info!("Pipe server created, waiting for connection...");
                    if let Err(e) = handle_client(&mut server, execute_use_case.clone()).await {
                        error!("Error handling client: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to create pipe server: {}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    });
}

async fn create_pipe_server() -> Result<NamedPipeServer, std::io::Error> {
    ServerOptions::new()
        .first_pipe_instance(true)
        .create(PIPE_NAME)
}

async fn handle_client(
    server: &mut NamedPipeServer,
    execute_use_case: Arc<dyn ExecuteOperation>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Wait for client connection
    server.connect().await?;
    info!("Client connected to pipe");

    // Read command
    let mut buffer = vec![0u8; BUFFER_SIZE];
    let n = timeout(
        Duration::from_secs(TIMEOUT_SECS),
        server.read(&mut buffer)
    ).await??;

    if n == 0 {
        warn!("Client disconnected without sending data");
        return Ok(());
    }

    let data = &buffer[..n];
    let command_str = String::from_utf8_lossy(data);

    // Parse command
    let pipe_cmd: PipeCommand = match serde_json::from_str(&command_str) {
        Ok(cmd) => cmd,
        Err(e) => {
            error!("Failed to parse command: {}", e);
            let response = r#"{"status":"error","message":"Invalid command format"}"#;
            let _ = server.write_all(response.as_bytes()).await;
            return Ok(());
        }
    };

    // Process command
    let response = match pipe_cmd.action {
        PipeAction::ExecuteOperation { command } => {
            match execute_use_case.execute(command).await {
                Ok(result) => {
                    // Serialize result as JSON response
                    serde_json::to_string(&result).unwrap_or_else(|_| {
                        r#"{"status":"error","message":"Failed to serialize result"}"#.to_string()
                    })
                }
                Err(e) => {
                    format!(r#"{{"status":"error","message":"{}"}}"#, e)
                }
            }
        }
    };

    // Send response back
    let _ = server.write_all(response.as_bytes()).await;
    info!("Response sent to client");

    // Disconnect gracefully
    server.disconnect()?;
    Ok(())
}