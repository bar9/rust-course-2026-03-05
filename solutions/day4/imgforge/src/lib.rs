mod error;
pub mod config;
pub mod transform;
pub mod transform_imagers;
#[cfg(feature = "turbojpeg")]
pub mod transform_turbojpeg;
pub mod server;
pub mod batch;

pub use error::{Error, Result};
pub use config::Config;
pub use transform::{Transform, Operation};

use config::{Command, CliOperation};

/// Main entry point: dispatch CLI, batch, or server mode.
pub fn run(config: Config) -> Result<()> {
    match config.command {
        Command::Cli { operation } => run_cli(operation),
        Command::Batch {
            input_dir,
            output_dir,
            operation,
        } => run_batch(&input_dir, &output_dir, operation),
        Command::Serve { port } => run_server(port),
    }
}

fn run_cli(operation: CliOperation) -> Result<()> {
    let backend = transform::default_backend();
    println!("Using backend: {}", backend.name());

    let (input_path, output_path, op) = match operation {
        CliOperation::Resize {
            width,
            height,
            input,
            output,
        } => (input, output, Operation::Resize { width, height }),
        CliOperation::Grayscale { input, output } => (input, output, Operation::Grayscale),
        CliOperation::Blur {
            sigma,
            input,
            output,
        } => (input, output, Operation::Blur { sigma }),
    };

    let input_data = std::fs::read(&input_path)?;
    let output_data = backend.apply(&input_data, &op)?;
    std::fs::write(&output_path, &output_data)?;

    println!(
        "Processed {} -> {}",
        input_path.display(),
        output_path.display()
    );
    Ok(())
}

fn run_batch(
    input_dir: &std::path::Path,
    output_dir: &std::path::Path,
    operation: CliOperation,
) -> Result<()> {
    let backend = transform::default_backend();
    println!("Using backend: {}", backend.name());

    let op = match operation {
        CliOperation::Resize { width, height, .. } => Operation::Resize { width, height },
        CliOperation::Grayscale { .. } => Operation::Grayscale,
        CliOperation::Blur { sigma, .. } => Operation::Blur { sigma },
    };

    let result = batch::process_directory(backend.as_ref(), input_dir, output_dir, &op)?;
    println!("{result}");
    Ok(())
}

fn run_server(port: u16) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let state = server::AppState {
            backend: std::sync::Arc::from(transform::default_backend()),
            jobs: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        };

        let app = server::router(state);
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
            .await
            .map_err(|e| Error::Io(e))?;

        println!("imgforge server listening on http://localhost:{port}");
        axum::serve(listener, app)
            .await
            .map_err(|e| Error::Io(e))?;

        Ok(())
    })
}
