use crate::db::AppState;
use crate::security::decode_jwt;
use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    web, Error, HttpMessage,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::rc::Rc;

pub struct Authorize {
    pub allowed_roles: Vec<String>,
}

impl Authorize {
    pub fn new(roles: Vec<&str>) -> Self {
        Self {
            allowed_roles: roles.into_iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for Authorize
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthorizeMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthorizeMiddleware {
            service: Rc::new(service),
            allowed_roles: Rc::new(self.allowed_roles.clone()),
        }))
    }
}

pub struct AuthorizeMiddleware<S> {
    service: Rc<S>,
    allowed_roles: Rc<Vec<String>>,
}

impl<S, B> Service<ServiceRequest> for AuthorizeMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let allowed_roles = self.allowed_roles.clone();
        let data = req.app_data::<web::Data<AppState>>().cloned();

        Box::pin(async move {
            let data = match data {
                Some(d) => d,
                None => return Err(ErrorUnauthorized("App state not found")),
            };

            let auth_header = req.headers().get("Authorization");

            let token = match auth_header {
                Some(h) => h.to_str().unwrap_or("").replace("Bearer ", ""),
                None => return Err(ErrorUnauthorized("No auth header")),
            };

            let claims = match decode_jwt(&token, &data.env) {
                Ok(c) => c,
                Err(_) => return Err(ErrorUnauthorized("Invalid token")),
            };

            if !allowed_roles.contains(&claims.role) {
                return Err(actix_web::error::ErrorForbidden("Insufficient permissions"));
            }

            req.extensions_mut().insert(claims);

            // Proceed to the service
            service.call(req).await
        })
    }
}
