use crate::docs::ApiDoc;
use crate::handlers::{auth, product, purchase, quotation, report, sale, settings};
use crate::middleware::Authorize;
use actix_web::web;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(
                web::scope("/auth")
                    .route("/login", web::post().to(auth::login_user))
                    .service(
                        web::resource("/register")
                            .wrap(Authorize::new(vec!["ADMIN"]))
                            .route(web::post().to(auth::register_user)),
                    )
                    .service(
                        web::resource("/me")
                            .wrap(Authorize::new(vec!["ADMIN", "SALE"]))
                            .route(web::get().to(auth::get_me)),
                    ),
            )
            .service(
                web::scope("/products")
                    .service(
                        web::resource("")
                            .route(web::get().to(product::get_products))
                            .route(
                                web::post()
                                    .to(product::create_product)
                                    .wrap(Authorize::new(vec!["ADMIN"])),
                            ),
                    )
                    .service(
                        web::scope("/{id}")
                            .wrap(Authorize::new(vec!["ADMIN", "SALE"]))
                            .route("", web::get().to(product::get_product_by_id))
                            .route("/status", web::patch().to(product::update_product_status))
                            .route("/purchases", web::get().to(product::get_product_purchases)),
                    ),
            )
            .service(
                web::scope("/sales")
                    .wrap(Authorize::new(vec!["ADMIN", "SALE"]))
                    .service(
                        web::resource("")
                            .route(web::post().to(sale::create_sale))
                            .route(web::get().to(sale::get_sales)),
                    )
                    .route("/{id}/status", web::patch().to(sale::update_sale_status)),
            )
            .service(
                web::scope("/purchases")
                    .wrap(Authorize::new(vec!["ADMIN"]))
                    .route("", web::post().to(purchase::create_purchase))
                    .route("", web::get().to(purchase::get_purchases)),
            )
            .service(
                web::scope("/settings")
                    .wrap(Authorize::new(vec!["ADMIN"]))
                    .route("/exchange", web::get().to(settings::get_exchange_rates))
                    .route(
                        "/exchange/{currency}",
                        web::post().to(settings::update_exchange_rate),
                    )
                    .route("/users", web::get().to(settings::get_users))
                    .route("/users/{id}", web::delete().to(settings::delete_user))
                    .route("/users/{id}/block", web::post().to(settings::block_user))
                    .route(
                        "/users/{id}/unblock",
                        web::post().to(settings::unblock_user),
                    )
                    .route(
                        "/users/{id}/role",
                        web::put().to(settings::change_user_role),
                    ),
            )
            .service(
                web::scope("/quotations")
                    .wrap(Authorize::new(vec!["ADMIN", "SALE"]))
                    .route("", web::post().to(quotation::create_quotation))
                    .route("", web::get().to(quotation::get_quotations))
                    .route(
                        "/{id}/status",
                        web::patch().to(quotation::update_quotation_status),
                    )
                    .route("/{id}/convert", web::post().to(quotation::convert_to_sale)),
            )
            .service(
                web::scope("/reports")
                    .wrap(Authorize::new(vec!["ADMIN"]))
                    .route("/products", web::get().to(report::get_product_reports))
                    .route("/low-stock", web::get().to(report::get_low_stock_report)),
            )
            .service(
                web::scope("/dashboard")
                    .wrap(Authorize::new(vec!["ADMIN", "SALE"]))
                    .route("/summary", web::get().to(report::get_dashboard_summary)),
            ),
    );

    cfg.service(
        SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()),
    );
}
