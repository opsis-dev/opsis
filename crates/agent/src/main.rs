use opsis_agent::acp::{OpsisAgent, serve};
use opsis_agent::cli::{Command, parse};
use opsis_agent::login::{Answers, Terminal, ask, logout, status, status_text, store};
use opsis_persistence::keys::{KeyError, Keyring};

#[derive(Debug)]
enum Job {
    Serve,
    Login(Answers),
    Logout,
    Status,
}

async fn run(job: Job, keys: Result<Keyring, KeyError>) -> Result<(), String> {
    let user_data = opsis_folders::user_data(&opsis_folders::host_folders());
    let pool = opsis_agent::db::open(&user_data)
        .await
        .map_err(|e| e.to_string())?;
    match job {
        Job::Serve => {
            let logs = opsis_persistence::logs::assistant_logs(&user_data);
            serve(OpsisAgent::new(pool, keys, &logs))
                .await
                .map_err(|e| e.to_string())
        }
        Job::Logout => {
            let keys = keys.map_err(|e| e.to_string())?;
            logout(&pool, &keys).await.map_err(|e| e.to_string())?;
            eprintln!("opsis-agent forgot its model and removed its keys.");
            Ok(())
        }
        Job::Status => {
            let found = status(&pool, keys).await.map_err(|e| e.to_string())?;
            println!("{}", status_text(&found));
            Ok(())
        }
        Job::Login(answers) => {
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
    let job = match parse(std::env::args_os()).unwrap_or_else(|error| error.exit()) {
        None => Job::Serve,
        Some(Command::Logout) => Job::Logout,
        Some(Command::Status) => Job::Status,
        Some(Command::Login) => match ask(&mut Terminal) {
            Ok(answers) => Job::Login(answers),
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
    let ran = runtime.block_on(run(job, keys.clone()));
    drop(runtime);
    drop(keys);
    if let Err(error) = ran {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
