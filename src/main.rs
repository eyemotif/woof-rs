mod cli;
mod http;

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = argh::from_env::<cli::Args>();
    println!("{}", args.file);
    Ok(())
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            std::process::ExitCode::FAILURE
        }
    }
}
