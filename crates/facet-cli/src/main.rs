use clap::Parser;
// use facet_webdriver::{ChromeDriver, ConnectionMode};

#[derive(Parser)]
#[command(name = "facet")]
#[command(version = "0.1.0")]
#[command(about = "Browser automation CLI prototype", long_about = None)]
struct Cli {
    /// URL to navigate to
    url: String,

    /// Connect to existing Chrome debug port (advanced mode)
    /// Example: --debug-port 9222
    #[arg(long)]
    debug_port: Option<u16>,

    /// Path to Chrome/Chromium executable (optional)
    /// Example: --chrome-path /usr/bin/chromium
    #[arg(long)]
    chrome_path: Option<String>,

    /// Output format: html or text
    #[arg(short = 'f', long, default_value = "html")]
    format: String,

    /// CSS selector for specific element (optional)
    #[arg(short = 's', long)]
    selector: Option<String>,

    /// Disable Chrome sandbox (required on some Linux systems with AppArmor restrictions)
    /// WARNING: Reduces security. Only use if Chrome fails to launch.
    #[arg(long)]
    no_sandbox: bool,

    /// Run Chrome in headless mode (no visible window)
    #[arg(long)]
    headless: bool,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("❌ Error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    println!("Facet CLI v0.1.0");
    println!("================\n");

    // Detect CI environment
    let is_ci = std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("GITLAB_CI").is_ok()
        || std::env::var("JENKINS_HOME").is_ok()
        || std::env::var("CIRCLECI").is_ok();

    // Auto-enable no-sandbox and headless in CI environments
    let _no_sandbox = cli.no_sandbox || is_ci;
    let _headless = cli.headless || is_ci;

    Ok(())
}
