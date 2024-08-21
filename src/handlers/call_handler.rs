use crate::models::{
    appstate::AppState,
    rest::{call_request::CallRequest, generic_response::GenericResponse},
};
use log::info;
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
        let chat_res = tdlib_rs::functions::search_public_chat(
            String::from(req.username.clone()),
            data.client_id,
        )
        .await;
        match chat_res {
            Ok(tdlib_rs::enums::Chat::Chat(chat)) => {
                let user = tdlib_rs::functions::get_user(chat.id, data.client_id).await;
                match user {
                    Ok(tdlib_rs::enums::User::User(usr)) => {
                        info!("going to call {}", usr.first_name);
                    }
                    _ => {}
                }
                let _ = tdlib_rs::functions::create_call(
                    chat.id,
                    tdlib_rs::types::CallProtocol {
                        udp_p2p: true,
                        udp_reflector: true,
                        min_layer: 121,
                        max_layer: 121,
                        library_versions: vec![],
                    },
                    false,
                    data.client_id,
                )
                .await;

                let obj = GenericResponse {
                    success: true,
                    message: String::from("done"),
                };
                return Ok(web::HttpResponse::Ok().json(&obj));
            }
            _ => {
                let obj = GenericResponse {
                    success: false,
                    message: String::from("call failed"),
                };
                return Ok(web::HttpResponse::Ok().json(&obj));
            }
        }
    }

    let obj = GenericResponse {
        success: false,
        message: String::from("wrong access key"),
    };
    Ok(web::HttpResponse::Ok().json(&obj))
}
