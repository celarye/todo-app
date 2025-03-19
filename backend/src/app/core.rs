use std::{sync::Arc, u16};

use actix_cors::Cors;
use actix_web::{App as ActixApp, HttpServer, http::Method, middleware::Logger, web};

use crate::Logic;
use crate::app::AppData;
use crate::app::handlers;

pub struct App {}

impl App {
    pub async fn run(
        address: &str,
        port: u16,
        allowed_origin: String,
        logic: Arc<Logic>,
    ) -> std::io::Result<()> {
        HttpServer::new(move || {
            ActixApp::new()
                .wrap(Logger::default())
                .wrap(
                    Cors::default()
                        .allowed_origin(&allowed_origin)
                        .allowed_methods(vec![
                            Method::GET,
                            Method::POST,
                            Method::PATCH,
                            Method::DELETE,
                        ])
                        .allow_any_header()
                        .supports_credentials()
                        .max_age(3600),
                )
                .app_data(web::Data::new(AppData::new(logic.clone())))
                .route("/", web::get().to(handlers::root))
                .service(
                    web::scope("/user")
                        .route("/", web::get().to(handlers::info))
                        .service(
                            web::scope("/auth")
                                .service(
                                    web::scope("/github")
                                        .route("/init", web::get().to(handlers::github_init))
                                        .route(
                                            "/success",
                                            web::post().to(handlers::github_success),
                                        ),
                                )
                                .route("/logout", web::delete().to(handlers::logout)),
                        ),
                )
                .service(
                    web::scope("/todo")
                        .route("/", web::get().to(handlers::get_items))
                        .route("/set", web::post().to(handlers::set_item))
                        .route("/update/{item_id}", web::patch().to(handlers::update_item))
                        .route("/delete/{item_id}", web::delete().to(handlers::delete_item)),
                )
        })
        .bind((address, port))?
        .run()
        .await
    }
}
