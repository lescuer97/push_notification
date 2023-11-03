use actix_files::NamedFile;
use push_service::{
    db::{get_subscription_by_action_condition, insert_subscription, Pool},
    error::CustomError,
    load_rustls_config, lookup_keys, push_message_request, Subscription, SubscriptionBody,
    SubscriptionOptions,
};
use r2d2_sqlite::SqliteConnectionManager;

use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};
//

use actix_cors::Cors;
use actix_web::{
    error::HttpError, get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use actix_web::{web::Query, Error};
use serde::Deserialize;
use web_push_native::jwt_simple::prelude::P256PublicKey;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().json("Hello world!")
}

/// Send uncompressed public key to the client, for encripted coms for the push notification.
#[get("/pkey")]
async fn get_public_key(data: web::Data<KeysState>) -> impl Responder {
    let pub_key = data.pub_key.to_bytes_uncompressed();

    return HttpResponse::Ok().body(pub_key);
}

#[derive(Deserialize, Debug)]
struct PushQuery {
    action: String,
}

/// check for the type of query on the url and send to the respective subscribers.
#[get("/send_push")]
async fn send_push(query: Query<PushQuery>, db: web::Data<Pool>) -> Result<impl Responder, Error> {
    let subs: Vec<Subscription> = get_subscription_by_action_condition(&db, &query.action)?;

    for sub in subs.iter() {
        push_message_request(sub, &db).await?;
    }

    return Ok(HttpResponse::Ok().json("Hello world!"));
}

/// Register a new public key subscription.
#[post("/subscribe")]
async fn subscribe(
    db: web::Data<Pool>,
    json: web::Json<SubscriptionBody>,
) -> Result<impl Responder, Error> {
    println!("subscribe: {:?}", json);
    let insert = insert_subscription(&db, json.clone())?;
    Ok(HttpResponse::Ok().body("Hello world!"))
}

#[derive(Clone, Debug)]
pub struct KeysState {
    pub pub_key: P256PublicKey,
}

/// send the static files for the basic html website.
async fn static_file(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse()?;
    Ok(NamedFile::open(path)?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = load_rustls_config();
    let pub_key = lookup_keys().expect("have vapid keys");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        let state_keys = web::Data::new(KeysState {
            pub_key: pub_key.clone(),
        });
        let mut notifications: Mutex<HashMap<String, Subscription>> = Mutex::new(HashMap::new());

        let mut notifications_state = web::Data::new(notifications);

        // connect to SQLite DB
        let manager = SqliteConnectionManager::file("notifs.db");
        let pool = Pool::new(manager).unwrap();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .app_data(state_keys.clone())
            .service(hello)
            .service(subscribe)
            .service(get_public_key)
            .service(send_push)
            .route("/{filename:.*}", web::get().to(static_file))
    })
    .bind_rustls_021("127.0.0.1:3000", config)?
    .run()
    .await
}
