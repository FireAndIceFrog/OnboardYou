mod api;
mod models;
mod server;

use clap::Parser;
use rmcp::{ServiceExt, transport::stdio};
use server::OnboardYouMcp;

#[derive(Parser)]
#[command(name = "onboardyou-mcp", about = "MCP server for OnboardYou pipeline configuration")]
struct Cli {
    /// Print MCP client configuration JSON and exit
    #[arg(long)]
    config: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
        tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();


    if cli.config {
        // Look for .env next to Cargo.toml (source dir), not cwd
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dotenvy::from_path(manifest_dir.join(".env")).ok();

        let api_url =
            std::env::var("ONBOARDYOU_API_URL").expect("ONBOARDYOU_API_URL must be set (in .env or environment)");
        let email =
            std::env::var("ONBOARDYOU_MCP_EMAIL").expect("ONBOARDYOU_MCP_EMAIL must be set (in .env or environment)");
        let password =
            std::env::var("ONBOARDYOU_MCP_PASSWORD").expect("ONBOARDYOU_MCP_PASSWORD must be set (in .env or environment)");
   
        let exe = std::env::current_exe()?
            .to_string_lossy()
            .into_owned();
        let config = serde_json::json!({
            "mcpServers": {
                "onboardyou": {
                    "command": exe,
                    "args": [],
                    "env": {
                        "ONBOARDYOU_API_URL": api_url,
                        "ONBOARDYOU_MCP_EMAIL": email,
                        "ONBOARDYOU_MCP_PASSWORD": password
                    }
                }
            }
        });
        println!("{}", serde_json::to_string_pretty(&config)?);
        return Ok(());
    }

    let api_url =
        std::env::var("ONBOARDYOU_API_URL").expect("ONBOARDYOU_API_URL must be set (in .env or environment)");
    let email =
        std::env::var("ONBOARDYOU_MCP_EMAIL").expect("ONBOARDYOU_MCP_EMAIL must be set (in .env or environment)");
    let password =
        std::env::var("ONBOARDYOU_MCP_PASSWORD").expect("ONBOARDYOU_MCP_PASSWORD must be set (in .env or environment)");

    tracing::info!("Logging in to {api_url}…");
    let api_client = api::ApiClient::login(&api_url, &email, &password).await?;
    tracing::info!("Authenticated — starting stdio MCP server");

    let service = OnboardYouMcp::new(api_client)
        .serve(stdio())
        .await
        .inspect_err(|e| tracing::error!("serving error: {:?}", e))?;

    service.waiting().await?;
    Ok(())
}
