#[tokio::main]
async fn main() {
    let execution = eggprobe_cli::execute(eggprobe_cli::parse());
    let result = tokio::select! {
        result = execution => result,
        _ = tokio::signal::ctrl_c() => {
            eprintln!("eggprobe: interrupted");
            std::process::exit(130);
        }
    };
    match result {
        Ok((code, output)) => {
            print!("{output}");
            std::process::exit(code);
        }
        Err(error) => {
            eprintln!("eggprobe: {error}");
            std::process::exit(error.code());
        }
    }
}
