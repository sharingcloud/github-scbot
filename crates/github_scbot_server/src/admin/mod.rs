use std::sync::Arc;

use actix_http::{http, Error};
use actix_identity::Identity;
use actix_web::{web, HttpResponse, Responder};
use juniper::http::{graphiql::graphiql_source, GraphQLRequest};
use serde::Deserialize;

use crate::{external::validator::extract_account_from_token, server::AppContext};

pub(crate) mod graphql;

#[derive(Deserialize)]
pub struct AdminLogin {
    key: String,
}

async fn graphiql_endpoint(id: Identity) -> HttpResponse {
    if let Some(_name) = id.identity() {
        let html = graphiql_source("http://localhost:8008/admin/graphql", None);
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html)
    } else {
        redirect_to_login().await
    }
}

async fn redirect_to_login() -> HttpResponse {
    HttpResponse::Found()
        .header(http::header::LOCATION, "/admin/login")
        .finish()
}

async fn graphql_endpoint(
    id: Identity,
    st: web::Data<Arc<graphql::Schema>>,
    ctx: web::Data<Arc<AppContext>>,
    data: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, Error> {
    if let Some(_name) = id.identity() {
        let cloned_ctx = ctx.clone();
        let resp = data.execute(&st, &cloned_ctx).await;
        let body = serde_json::to_string(&resp)?;
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body))
    } else {
        Ok(redirect_to_login().await)
    }
}

async fn admin_login_get(id: Identity) -> impl Responder {
    if let Some(_name) = id.identity() {
        HttpResponse::Found()
            .header(http::header::LOCATION, "/admin/")
            .finish()
    } else {
        let tpl = r#"
        <!doctype html>
        <html>
        <head><title>Login</title></head>
        <body>
            <form method="post" action="/admin/login">
            Token: <input name="key"></input>
            <button type="submit">Submit</button>
            </form>
        </body>
        </html>
        "#;
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(tpl)
    }
}

async fn admin_login_post(
    id: Identity,
    ctx: web::Data<Arc<AppContext>>,
    body: web::Form<AdminLogin>,
) -> impl Responder {
    let token = &body.key;
    if let Ok(account) = extract_account_from_token(&*ctx.db_adapter, token).await {
        if account.is_superuser {
            id.remember(account.username);
            return HttpResponse::Found()
                .header(http::header::LOCATION, "/admin/")
                .finish();
        }
    }

    redirect_to_login().await
}

async fn admin_logout(id: Identity) -> impl Responder {
    id.forget();
    redirect_to_login().await
}

async fn admin_index(id: Identity) -> HttpResponse {
    if let Some(name) = id.identity() {
        admin_index_frontend(name).await
    } else {
        redirect_to_login().await
    }
}

#[cfg(not(feature = "embedded-frontend"))]
async fn admin_index_frontend(_name: String) -> HttpResponse {
    HttpResponse::Found()
        .header(http::header::LOCATION, "http://localhost:8081")
        .finish()
}

#[cfg(feature = "embedded-frontend")]
async fn admin_index_frontend(_name: String) -> HttpResponse {
    let body = include_str!("./frontend/dist/index.html");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body)
}

#[cfg(not(feature = "embedded-frontend"))]
async fn bundle_js() -> impl Responder {
    HttpResponse::NotFound()
}

#[cfg(feature = "embedded-frontend")]
async fn bundle_js() -> impl Responder {
    let body = include_str!("./frontend/dist/bundle.js");
    HttpResponse::Ok()
        .content_type("text/javascript")
        .body(body)
}

pub fn configure_admin_handlers(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/").route(web::get().to(admin_index)));
    cfg.service(
        web::resource("/login")
            .route(web::get().to(admin_login_get))
            .route(web::post().to(admin_login_post)),
    );
    cfg.service(web::resource("/logout").route(web::get().to(admin_logout)));
    cfg.service(web::resource("/graphql").route(web::post().to(graphql_endpoint)));
    cfg.service(web::resource("/graphiql").route(web::get().to(graphiql_endpoint)));
    cfg.service(web::resource("/bundle.js").route(web::get().to(bundle_js)));
}
