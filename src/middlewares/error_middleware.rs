use ntex::service::{Middleware, Service, ServiceCtx};
use ntex::web::{self, WebResponse};

use crate::models::rest::generic_response::GenericResponse;

pub struct Error;

impl<S> Middleware<S> for Error {
    type Service = ErrorMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        ErrorMiddleware { service }
    }
}

pub struct ErrorMiddleware<S> {
    service: S,
}

impl<S, Err> Service<web::WebRequest<Err>> for ErrorMiddleware<S>
where
    S: Service<web::WebRequest<Err>, Response = web::WebResponse, Error = web::Error>,
    Err: web::ErrorRenderer,
{
    type Response = web::WebResponse;
    type Error = web::Error;

    ntex::forward_ready!(service);

    async fn call(
        &self,
        req: web::WebRequest<Err>,
        ctx: ServiceCtx<'_, Self>,
    ) -> Result<Self::Response, Self::Error> {
        ctx.call(&self.service, req).await.map(|mut res| {
            let status = res.status();
            if status.is_client_error() || status.is_server_error() {
                let obj = GenericResponse {
                    success: false,
                    message: String::from("request failed"),
                };
                let nresp = web::HttpResponse::BadRequest().json(&obj);
                let rq = res.request().clone();
                res = WebResponse::new(nresp, rq);
            }
            res
        })
    }
}
