use std::path::Path;

use crate::error::{Error, Result};
use crate::transform::{Operation, Transform};

/// Process all images in `input_dir`, writing results to `output_dir`.
///
/// Supports JPEG and PNG files. Skips files that cannot be read as images.
pub fn process_directory(
    backend: &dyn Transform,
    input_dir: &Path,
    output_dir: &Path,
    operation: &Operation,
) -> Result<BatchResult> {
    if !input_dir.is_dir() {
        return Err(Error::InvalidOperation {
            message: format!("Not a directory: {}", input_dir.display()),
        });
    }

    std::fs::create_dir_all(output_dir)?;

    let mut result = BatchResult {
        processed: 0,
        skipped: 0,
        errors: Vec::new(),
    };

    for entry in std::fs::read_dir(input_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !matches!(ext.as_str(), "jpg" | "jpeg" | "png") {
            result.skipped += 1;
            continue;
        }

        let file_name = path.file_name().unwrap();
        let output_path = output_dir.join(file_name);

        match process_single_file(backend, &path, &output_path, operation) {
            Ok(()) => result.processed += 1,
            Err(e) => {
                result.errors.push(format!("{}: {e}", path.display()));
            }
        }
    }

    Ok(result)
}

fn process_single_file(
    backend: &dyn Transform,
    input: &Path,
    output: &Path,
    operation: &Operation,
) -> Result<()> {
    let input_data = std::fs::read(input)?;
    let output_data = backend.apply(&input_data, operation)?;
    std::fs::write(output, output_data)?;
    Ok(())
}

#[derive(Debug)]
pub struct BatchResult {
    pub processed: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

impl std::fmt::Display for BatchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Batch complete: {} processed, {} skipped, {} errors",
            self.processed,
            self.skipped,
            self.errors.len()
        )?;
        for err in &self.errors {
            write!(f, "\n  - {err}")?;
        }
        Ok(())
    }
}
