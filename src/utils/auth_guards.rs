use std::{
    future::{ready, Ready},
    io::ErrorKind,
};

use actix_web::{
    cookie::Cookie,
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;

use crate::constants::auth_cookie_name;

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct AuthGuardFactory;

// Middleware factory is `Transform` trait from actix-service crate
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for AuthGuardFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthGuard<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthGuard { service }))
    }
}

pub struct AuthGuard<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthGuard<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if let Some(_) = req.cookie(auth_cookie_name) {
            println!("Got cookie");
            let fut = self.service.call(req);
            Box::pin(async move { fut.await })
        } else {
            println!("Aint got cookie");
            Box::pin(async move {
                Err(Error::from(std::io::Error::new(
                    ErrorKind::Other,
                    "Unauthenticated",
                )))
            })
        }
    }
}