use actix_multipart::Multipart;
use futures_util::TryStreamExt;
use isabelle_dm::data_model::item::Item;
use isabelle_dm::data_model::process_result::ProcessResult;
use std::ops::Deref;

use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use serde_qs;
use serde_qs::Config;

use crate::state::state::*;

use log::info;

use crate::notif::gcal::*;
use crate::server::user_control::*;
use crate::state::data_rw::*;
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;

pub async fn setting_edit(
    user: Identity,
    data: web::Data<State>,
    req: HttpRequest,
    mut payload: Multipart,
) -> HttpResponse {
    let mut srv = data.server.lock().unwrap();
    let usr = get_user(srv.deref(), user.id().unwrap());

    if !check_role(&srv, &usr, "admin") {
        return HttpResponse::Forbidden().into();
    }

    let mut itm = serde_qs::from_str::<Item>(&req.query_string()).unwrap();

    while let Ok(Some(mut field)) = payload.try_next().await {
        while let Ok(Some(chunk)) = field.try_next().await {
            let data = chunk;

            if field.name() == "item" {
                let v = &data.to_vec();
                let strv = std::str::from_utf8(v).unwrap_or("{}");
                let new_itm: Item = serde_json::from_str(strv).unwrap_or(Item::new());
                itm.merge(&new_itm);
            }
        }
    }

    srv.settings = itm.clone();
    info!("Settings edited");
    write_data(srv.deref_mut());
    return HttpResponse::Ok().body(
        serde_json::to_string(&ProcessResult {
            succeeded: true,
            error: "".to_string(),
        })
        .unwrap(),
    );
}

pub async fn setting_list(
    user: Identity,
    data: web::Data<State>,
    _req: HttpRequest,
) -> HttpResponse {
    let srv = data.server.lock().unwrap();
    let usr = get_user(srv.deref(), user.id().unwrap());

    if !check_role(&srv, &usr, "admin") {
        return HttpResponse::Forbidden().into();
    }

    let st = srv.settings.clone();
    HttpResponse::Ok()
        .body(serde_json::to_string(&st).unwrap())
        .into()
}

pub async fn setting_gcal_auth(
    user: Identity,
    data: web::Data<State>,
    _req: HttpRequest,
) -> HttpResponse {
    let srv = data.server.lock().unwrap();
    let usr = get_user(srv.deref(), user.id().unwrap());

    if !check_role(&srv, &usr, "admin") {
        return HttpResponse::Forbidden().into();
    }

    HttpResponse::Ok().body(auth_google(&srv)).into()
}

pub async fn setting_gcal_auth_end(
    user: Identity,
    data: web::Data<State>,
    _req: HttpRequest,
) -> HttpResponse {
    let srv = data.server.lock().unwrap();
    let usr = get_user(srv.deref(), user.id().unwrap());

    if !check_role(&srv, &usr, "admin") {
        return HttpResponse::Forbidden().into();
    }

    info!("Auth end");
    #[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
    pub struct AuthEndData {
        pub state: String,
        pub code: String,
        pub scope: String,
    }

    let config = Config::new(10, false);
    let data: AuthEndData = config.deserialize_str(&_req.query_string()).unwrap();

    HttpResponse::Ok()
        .body(auth_google_end(
            &srv,
            srv.public_url.clone() + "/?" + _req.query_string(),
            data.state,
            data.code,
        ))
        .into()
}
