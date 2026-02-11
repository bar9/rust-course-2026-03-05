use clap::Parser;

fn main() -> imgforge::Result<()> {
    let config = imgforge::Config::parse();
    imgforge::run(config)
}
