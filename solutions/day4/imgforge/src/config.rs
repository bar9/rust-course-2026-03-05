use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "imgforge", about = "Image processing CLI & server")]
pub struct Config {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Process a single image
    Cli {
        #[command(subcommand)]
        operation: CliOperation,
    },
    /// Process all images in a directory
    Batch {
        /// Input directory
        #[arg(long)]
        input_dir: PathBuf,
        /// Output directory
        #[arg(long)]
        output_dir: PathBuf,
        #[command(subcommand)]
        operation: CliOperation,
    },
    /// Start the HTTP server
    Serve {
        /// Port to listen on
        #[arg(long, default_value = "3000")]
        port: u16,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum CliOperation {
    /// Resize an image
    Resize {
        /// Target width
        #[arg(long)]
        width: u32,
        /// Target height
        #[arg(long)]
        height: u32,
        /// Input file
        input: PathBuf,
        /// Output file
        output: PathBuf,
    },
    /// Convert to grayscale
    Grayscale {
        /// Input file
        input: PathBuf,
        /// Output file
        output: PathBuf,
    },
    /// Apply Gaussian blur
    Blur {
        /// Blur sigma (strength)
        #[arg(long, default_value = "3.0")]
        sigma: f32,
        /// Input file
        input: PathBuf,
        /// Output file
        output: PathBuf,
    },
}
