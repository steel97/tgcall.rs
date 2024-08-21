use crate::{
    models::{
        appstate::AppState,
        rest::{generic_response::GenericResponse, tgcode_request::TgCodeRequest},
    },
    telegram::events::Functions,
};
use ntex::web;

#[web::get("/tgcode")]
pub async fn tgcode_handler(
    req: web::types::Query<TgCodeRequest>,
    data: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let key: String;
    {
        let cfg = data.config.lock().await;
        key = cfg.access_key.clone();
    }

    if req.key == key {
        let events = data.events.lock().await;
        events.send_code(data.client_id, req.code.clone()).await;

        let obj = GenericResponse {
            success: true,
            message: String::from("done"),
        };
        return Ok(web::HttpResponse::Ok().json(&obj));
    }

    let obj = GenericResponse {
        success: false,
        message: String::from("wrong access key"),
    };
    Ok(web::HttpResponse::Ok().json(&obj))
}
