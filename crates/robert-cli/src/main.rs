use clap::Parser;

#[derive(Parser)]
#[command(name = "robert")]
#[command(version = "0.1.0")]
#[command(about = "Robert CLI", long_about = None)]
struct Cli {}

#[tokio::main]
async fn main() {
    let _cli = Cli::parse();

    println!("Robert CLI v0.1.0");
    println!("Coming soon...");
}
