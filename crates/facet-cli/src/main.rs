use clap::Parser;
// use facet_webdriver::{ChromeDriver, ConnectionMode};

#[derive(Parser)]
#[command(name = "facet")]
#[command(version = "0.1.0")]
#[command(about = "Facet CLI", long_about = None)]
struct Cli {}

#[tokio::main]
async fn main() {
    let _cli = Cli::parse();

    println!("Facet CLI v0.1.0");
    println!("================\n");

    // Detect CI environment
    let is_ci = std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("GITLAB_CI").is_ok()
        || std::env::var("JENKINS_HOME").is_ok()
        || std::env::var("CIRCLECI").is_ok();

    // Auto-enable no-sandbox and headless in CI environments
    // let _no_sandbox = cli.no_sandbox || is_ci;
    // let _headless = cli.headless || is_ci;
    if is_ci {
        println!("CI environment detected.");
    }

    Ok(())
}
