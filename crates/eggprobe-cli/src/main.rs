#[tokio::main]
async fn main() {
    match eggprobe_cli::execute(eggprobe_cli::parse()).await {
        Ok((code, output)) => {
            print!("{output}");
            std::process::exit(code);
        }
        Err(error) => {
            eprintln!("eggprobe: {error}");
            std::process::exit(2);
        }
    }
}
