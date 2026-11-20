use opsis_agent::acp::{OpsisAgent, serve};
use opsis_agent::cli::{Command, parse};
use opsis_agent::login::{Answers, Terminal, ask, store};
use opsis_persistence::keys::{KeyError, Keyring};

async fn run(login: Option<Answers>, keys: Result<Keyring, KeyError>) -> Result<(), String> {
    let user_data = opsis_folders::user_data(&opsis_folders::host_folders());
    let pool = opsis_agent::db::open(&user_data)
        .await
        .map_err(|e| e.to_string())?;
    match login {
        None => serve(OpsisAgent::new(pool, keys))
            .await
            .map_err(|e| e.to_string()),
        Some(answers) => {
            let (model, service) = (answers.chosen.model.clone(), answers.chosen.service.id());
            let keys = keys.map_err(|e| e.to_string())?;
            store(&pool, &keys, answers)
                .await
                .map_err(|e| e.to_string())?;
            eprintln!("opsis-agent will use {model} on {service}.");
            Ok(())
        }
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = match parse(&args) {
        Ok(command) => command,
        Err(usage) => {
            eprintln!("{usage}");
            std::process::exit(2);
        }
    };
    let login = match command {
        Command::Serve => None,
        Command::Login => match ask(&mut Terminal) {
            Ok(answers) => Some(answers),
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        },
    };
    let keys = Keyring::platform();
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("opsis-agent couldn't start: {error}");
            std::process::exit(1);
        }
    };
    let ran = runtime.block_on(run(login, keys.clone()));
    drop(runtime);
    drop(keys);
    if let Err(error) = ran {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
