use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

use crate::handlers::settings::{ChangeRoleSchema, ExchangeRate, UpdateExchangeRateSchema};
use crate::handlers::{auth, product, purchase, quotation, report, sale, settings};
use crate::models::{product::*, purchase::*, quotation::*, sale::*, user::*};

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::register_user,
        auth::login_user,
        auth::get_me,
        product::get_products,
        product::create_product,
        product::get_product_by_id,
        product::update_product,
        product::update_product_status,
        product::get_product_purchases,
        purchase::create_purchase,
        purchase::get_purchases,
        quotation::create_quotation,
        quotation::get_quotations,
        quotation::update_quotation_status,
        quotation::convert_to_sale,
        sale::create_sale,
        sale::get_sales,
        sale::update_sale_status,
        report::get_dashboard_summary,
        report::get_product_reports,
        report::get_low_stock_report,
        settings::get_exchange_rates,
        settings::update_exchange_rate,
        settings::get_users,
        settings::block_user,
        settings::unblock_user,
        settings::change_user_role,
        settings::delete_user,
    ),
    components(
        schemas(
            User, CreateUserSchema, LoginSchema, UserResponse,
            Product, CreateProductSchema, UpdateProductSchema, product::UpdateProductStatusSchema,
            Purchase, PurchaseItem, CreatePurchaseItemSchema, CreatePurchaseSchema,
            Quotation, QuotationItem, CreateQuotationItemSchema, CreateQuotationSchema, UpdateQuotationStatusSchema,
            Sale, SaleItem, CreateSaleItemSchema, CreateSaleSchema, UpdateSaleStatusSchema,
            ExchangeRate, UpdateExchangeRateSchema, ChangeRoleSchema
        )
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        )
    }
}
