use env_logger::Env;
use log::info;
use ntex::web;
use std::{
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
    let config = Arc::new(Mutex::new(Config::read_config("./config.json").unwrap()));
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
    .bind(("127.0.0.1", 8085))?
    .run()
    .await;
    Ok(())
}
