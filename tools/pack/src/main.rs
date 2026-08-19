mod cli;
mod config;
mod format;
mod archive;
mod output;

use std::process;

fn main() {
    let args = cli::parse();
    let config = config::Config::load(&args);

    let result = match &args.command {
        cli::Command::Create { paths, output_file } => {
            archive::create(paths, output_file, &config)
        }
        cli::Command::Extract { archive, output, files } => {
            archive::extract(archive, output.as_deref(), files, &config)
        }
        cli::Command::List { archive } => {
            archive::list(archive, &config)
        }
        cli::Command::Info { archive } => {
            archive::info(archive, &config)
        }
        cli::Command::Verify { archive } => {
            archive::verify(archive, &config)
        }
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        process::exit(1);
    }
}
