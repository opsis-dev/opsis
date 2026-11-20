use opsis_folders::address::this_computers_api_address;
use opsis_mcp::cli::{Mode, Options, parse};
use opsis_mcp::local_api::LocalApi;
use opsis_mcp::server::OpsisTools;
use rmcp::ServiceExt;

async fn serve(options: Options) -> Result<(), Box<dyn std::error::Error>> {
    let address = this_computers_api_address();
    if let Some(error) = &address.error {
        log::warn!("{error}");
    }
    let tools = OpsisTools::new(
        LocalApi::new(address.address, options.session),
        options.full,
    );
    match options.mode {
        Mode::Stdio => {
            let running = tools.serve(rmcp::transport::stdio()).await?;
            running.waiting().await?;
        }
        Mode::Http { host, port } => opsis_mcp::http::serve(tools, &host, port).await?,
    }
    Ok(())
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let options = match parse(&args) {
        Ok(options) => options,
        Err(usage) => {
            eprintln!("{usage}");
            std::process::exit(2);
        }
    };
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("opsis-mcp couldn't start: {error}");
            std::process::exit(1);
        }
    };
    if let Err(error) = runtime.block_on(serve(options)) {
        eprintln!("opsis-mcp stopped: {error}");
        std::process::exit(1);
    }
}
