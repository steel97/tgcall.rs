use crate::{
    models::{
        appstate::AppState,
        rest::{call_request::CallRequest, generic_response::GenericResponse},
    },
    telegram::events::Functions,
};
use ntex::web;

#[web::get("/call")]
pub async fn call_handler(
    req: web::types::Query<CallRequest>,
    data: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let key: String;
    {
        let cfg = data.config.lock().await;
        key = cfg.access_key.clone();
    }

    if req.key == key {
        // call telegram user
        let events = data.events.lock().await;
        let res = events.call(data.client_id, req.username.clone()).await;
        return match res {
            Ok(()) => {
                let obj = GenericResponse {
                    success: true,
                    message: String::from("done"),
                };
                Ok(web::HttpResponse::Ok().json(&obj))
            }
            _ => {
                let obj = GenericResponse {
                    success: false,
                    message: String::from("failed to call user"),
                };
                Ok(web::HttpResponse::Ok().json(&obj))
            }
        };
    }

    let obj = GenericResponse {
        success: false,
        message: String::from("wrong access key"),
    };
    Ok(web::HttpResponse::Ok().json(&obj))
}
