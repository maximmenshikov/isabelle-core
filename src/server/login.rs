use serde_qs;
use serde_qs::Config;
use actix_identity::Identity;
use actix_web::{web, HttpMessage, HttpResponse, HttpRequest, Responder};
use crate::state::state::*;
use log::{info};
use serde::{Deserialize, Serialize};

pub async fn login(_user: Option<Identity>, _data: web::Data<State>, request: HttpRequest) -> impl Responder {
    let srv = _data.server.lock().unwrap();

    #[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
    pub struct LoginUser {
        pub username: String,
        pub password: String,
    }

    let config = Config::new(10, false);
    let c : LoginUser = config.deserialize_str(&request.query_string()).unwrap();
    let mut found : bool = false;

    for item in &srv.items {
        if item.1.bool_params.contains_key("is_human") {
            info!("{} / {} against {} / {}",
                  item.1.fields["login"], item.1.fields["password"],
                  c.username, c.password);
        }
        if item.1.bool_params.contains_key("is_human") &&
           item.1.fields.contains_key("login") &&
           item.1.fields["login"] == c.username &&
           item.1.fields["password"] == c.password {
            Identity::login(&request.extensions(), c.username.clone()).unwrap();
            info!("Logged in! {}", c.username);
            found = true;
            break;
        }
    }

    if !found {
        info!("No user found, couldn't log in");
    }

    HttpResponse::Ok()
}

pub async fn logout(_user: Identity, _data: web::Data<State>, _request: HttpRequest) -> impl Responder {
    _user.logout();
    info!("Logged out");

    HttpResponse::Ok()
}

pub async fn is_logged_in(_user: Option<Identity>, data: web::Data<State>) -> impl Responder {
    let srv = data.server.lock().unwrap();

    #[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
    pub struct LoginUser {
        pub username: String,
        pub id: u64,
        pub role: Vec<String>,
        pub site_name: String,
        pub site_logo: String,
        pub licensed_to: String,
    }

    let mut user : LoginUser = LoginUser { username: "".to_string(), id: 0, role: Vec::new(), site_name: "".to_string(), site_logo: "".to_string(), licensed_to: "".to_string()};

    user.site_name = srv.settings.clone().safe_str("site_name", "Isabelle");
    if user.site_name == "" {
        user.site_name = "Isabelle".to_string();
    }

    user.site_logo = srv.settings.clone().safe_str("site_logo", "");
    if user.site_logo == "" {
        user.site_logo = "logo.png".to_string();
    }

    user.licensed_to = srv.settings.clone().safe_str("licensed_to", "");
    if user.licensed_to == "" {
        user.licensed_to = "end user".to_string();
    }

    if _user.is_none() {
        info!("No user");
        return web::Json(user)
    }

    for item in &srv.items {
        if item.1.fields.contains_key("login") &&
           item.1.fields["login"] == _user.as_ref().unwrap().id().unwrap() {
            if item.1.bool_params.contains_key("is_human") {
                user.username = _user.as_ref().unwrap().id().unwrap();
                user.id = *item.0;
                for bp in &item.1.bool_params {
                    if bp.0.starts_with("role_is_") {
                        user.role.push(bp.0[8..].to_string());
                    }
                }
                break;
            }
        }
    }

    web::Json(user)
}
