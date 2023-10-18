use actix_files::NamedFile;
use push_service::{push_message_request, PushSubscription, PushSubscriptionOptions};

use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};
use web_push_native::jwt_simple::prelude::{ECDSAP256KeyPairLike, ES256KeyPair, P256PublicKey};
//

use actix_cors::Cors;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result};
use base64ct::{Base64UrlUnpadded, Encoding};
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().json("Hello world!")
}

#[get("/pkey")]
async fn get_public_key(data: web::Data<KeysState>) -> impl Responder {
    let pub_key = data.pub_key.to_bytes_uncompressed();

    return HttpResponse::Ok().body(pub_key);
}

#[get("/send_push")]
async fn send_push(notif_state: web::Data<PushSubscription>) -> impl Responder {
    println!("notif_state: {:?}", notif_state);
    push_message_request(notif_state).await;

    return HttpResponse::Ok().body("Hello world!");
}

#[post("/subscribe")]
async fn subscribe(// json: web::Json<PushSubscription>,
) -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[derive(Clone, Debug)]
pub struct KeysState {
    pub pub_key: P256PublicKey,
    // pub sec_key: &ES256KeyPair,
}

async fn static_file(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    Ok(NamedFile::open(path)?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = load_rustls_config();
    let pub_key = lookup_keys();

    HttpServer::new(move || {
        let cors = Cors::default().allow_any_origin().allow_any_method().allow_any_header();

        let state_keys = web::Data::new(KeysState {
            pub_key: pub_key.clone(),
        });
        let mut notifications: Mutex<HashMap<String, PushSubscription>> =Mutex::new(HashMap::new());

        let mut notifications_state = web::Data::new(notifications);

        let notif_option = PushSubscriptionOptions {
                p256dh: "BNmRp61O5ZaGfT5k5Q5BKUpkF6Xw4P6NTg2RXVgvd_diFi3x86g2gf0BfbgfRKj5HFRBpL5nmxdnCBSKGd5yCt4".to_string(),
                auth: "5GfkRaNKByyjRwOEkFscpA".to_string(),
        };
        let sub = PushSubscription {
            endpoint: "https://fcm.googleapis.com/fcm/send/fWexfx6siI0:APA91bHN78EZEb30KJPZiymNuZeTrE0MPAGI7KS5QHvX0dzE1Iyiz_EUeqDxFzswQhO83ge7jI0IV3z5KpeWFpuN-7ru0JHB85wmzL6WnUKLp1gYyBo0CDIrYeIcYYyJH_hnjcWpfivI".to_string(),
            expirationTime: None,
            keys: notif_option
        };

        let notif = web::Data::new(sub);

        App::new()
            .wrap(cors)
            .app_data(state_keys.clone())
            .app_data(notif)
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

use std::io::prelude::*;
/// Lookup keys if there are vapid keys in the file system and if not it will generate them
fn lookup_keys() -> P256PublicKey {
    let privkey_mutex: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    thread::scope(|s| {
        let priv_key = privkey_mutex.clone();

        let handle = thread::spawn(move || {
            let mut private_key = priv_key.lock().unwrap();
            match File::open("vapid/private.key").ok() {
                Some(mut file_t) => {
                    file_t
                        .read_to_end(&mut private_key)
                        .expect("Could not read private key file");
                    return ();
                }
                None => {
                    let sec_key = generate_keys();
                    let file = File::create("vapid/private.key")
                        .expect("Could not create private key file");

                    let mut writer = BufWriter::new(&file);

                    let key = sec_key.to_bytes();

                    writer
                        .write_all(&key)
                        .expect("Could not write private key file");
                    writer.flush().expect("Could not flush private key file");

                    let mut file2 =
                        File::open("vapid/private.key").expect("Could not open private key file");
                    file2
                        .read_to_end(&mut private_key)
                        .expect("Could not read private key file 2");

                    return ();
                }
            };
        });

        handle.join().unwrap();
    });

    let private_key = privkey_mutex.lock().expect("Could not lock private key");

    let secret_key = ES256KeyPair::from_bytes(&private_key).expect("coould not get key pair");
    let string_key = Base64UrlUnpadded::encode_string(private_key.as_slice());

    std::env::set_var("VAPID_PRIVATE", string_key.as_str());

    let pub_key = secret_key.key_pair().public_key();

    pub_key
}

fn remove_pem_headers(pem_string: &str) -> String {
    let lines: Vec<&str> = pem_string.lines().collect();
    let mut result = String::new();

    let mut in_pem_section = false;
    for line in lines {
        if line.starts_with("-----BEGIN PRIVATE KEY-----") {
            in_pem_section = true;
            continue;
        } else if line.starts_with("-----END PRIVATE KEY-----") {
            in_pem_section = false;
            continue;
        }

        if !in_pem_section {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}

fn load_rustls_config() -> rustls::ServerConfig {
    // init server config builder with safe defaults
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();

    // load TLS key/cert files
    let cert_file = &mut BufReader::new(File::open("cert.pem").unwrap());
    let key_file = &mut BufReader::new(File::open("key.pem").unwrap());

    // convert files to key/cert objects
    let cert_chain = certs(cert_file)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();
    let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_file)
        .unwrap()
        .into_iter()
        .map(PrivateKey)
        .collect();

    // exit if no keys could be parsed
    if keys.is_empty() {
        eprintln!("Could not locate PKCS 8 private keys.");
        std::process::exit(1);
    }

    config.with_single_cert(cert_chain, keys.remove(0)).unwrap()
}

fn generate_keys() -> ES256KeyPair {
    let keypair = ES256KeyPair::generate();

    return keypair;
}
