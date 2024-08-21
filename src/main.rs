use env_logger::Env;
use log::info;
use ntex::web;
use std::{
    fs,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tgcall::{
    handlers::{call_handler::call_handler, tgcode_handler::tgcode_handler},
    models::{appstate::AppState, config::Config},
    telegram::events::{Events, Handlers},
};
use tokio::{
    sync::{mpsc, Mutex},
    task,
    time::sleep,
};

#[ntex::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    if !Path::new("./config.json").exists() {
        panic!("config.json was not found!");
    }
    let config = Arc::new(Mutex::new(Config::read_config("./config.json").unwrap()));
    /* ensure directories */
    {
        let cfg = config.lock().await;
        let res = fs::create_dir_all(cfg.db_path.clone());
        match res {
            Err(_) => {
                panic!("failed to create directory, check your 'db_path' config variable");
            }
            _ => {}
        }

        let res = fs::create_dir_all(cfg.files_path.clone());
        match res {
            Err(_) => {
                panic!("failed to create directory, check your 'files_path' config variable");
            }
            _ => {}
        }
    }

    let events = Arc::new(Mutex::new(Events {
        config: config.clone(),
    }));
    let events_clone = events.clone();
    let run_flag = Arc::new(AtomicBool::new(true));
    let run_flag_clone = run_flag.clone();

    let (tx, mut rx) = mpsc::unbounded_channel();
    // update or response listener
    task::spawn(async move {
        while run_flag_clone.load(Ordering::Acquire) {
            if let Some((update, client_id)) = tdlib_rs::receive() {
                let _ = tx.send((update, client_id));
            }
            sleep(Duration::from_millis(10)).await;
        }
    });

    // update handler
    tokio::spawn(async move {
        loop {
            while let Some((update, client_id)) = rx.recv().await {
                let evt = events.lock().await;
                evt.handle_update(update, client_id).await;
            }
        }
    });

    let client_id = tdlib_rs::create_client();
    let _ = tdlib_rs::functions::set_log_verbosity_level(1, client_id).await;

    info!("initialized");
    let host: String;
    let port: u16;
    {
        let cfg = config.lock().await;
        match &cfg.server {
            Some(srv) => {
                host = srv.host.clone();
                port = srv.port;
            }
            _ => {
                host = String::from("127.0.0.1");
                port = 8080;
            }
        }
    }

    let _ = web::HttpServer::new(move || {
        web::App::new()
            .state(AppState {
                config: config.clone(),
                client_id: client_id,
                events: events_clone.clone(),
            })
            .wrap(tgcall::middlewares::error_middleware::Error)
            .service(call_handler)
            .service(tgcode_handler)
    })
    .bind((host.as_str(), port))?
    .run()
    .await;
    Ok(())
}
