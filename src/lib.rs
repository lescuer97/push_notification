use actix_web::Result;
use base64ct::{Base64UrlUnpadded, Encoding};
use error::CustomError;
use hyper::{Body, Client, Response};
use hyper_rustls::HttpsConnectorBuilder;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use web_push_native::{p256::PublicKey, Auth, WebPushBuilder};

use web_push_native::jwt_simple::prelude::{ECDSAP256KeyPairLike, ES256KeyPair, P256PublicKey};

use std::fs::File;
use std::io::prelude::*;
use std::{
    io::{BufReader, BufWriter, Write},
    sync::{Arc, Mutex},
    thread,
};

pub mod db;
pub mod error;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Notification {
    pub action_condition: String,
    pub subscriptions: Option<i64>,
    pub id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SubscriptionOptions {
    pub auth: String,
    pub p256dh: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Subscription {
    pub endpoint: String,
    pub expirationTime: Option<i64>,
    pub keys: SubscriptionOptions,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SubscriptionBody {
    pub subscription_push: Subscription,
    pub action_condition: Vec<String>,
}

pub async fn push_message_request(post: &Subscription) -> Result<(), CustomError> {
    let vapid_env = std::env::var("VAPID_PRIVATE")?;

    let bytes = Base64UrlUnpadded::decode_vec(vapid_env.as_str())?;

    let vapid_private = ES256KeyPair::from_bytes(&bytes)?;

    let content = r#"{"title": "Portugal vs. Denmark",
                "data":"hello world from app server"}"#;

    let builder = WebPushBuilder::new(
        post.endpoint.parse()?,
        PublicKey::from_sec1_bytes(&Base64UrlUnpadded::decode_vec(post.keys.p256dh.as_str())?)?,
        Auth::clone_from_slice(&Base64UrlUnpadded::decode_vec(post.keys.auth.as_str())?),
    )
    .with_vapid(&vapid_private, "mailto:leo@leito.dev");

    let https = HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_only()
        .enable_http1()
        .build();

    let client: Client<_, Body> = Client::builder().build(https);

    // Parse the string of data into serde_json::Value.
    let value: Value = serde_json::from_str(&content)?;

    let req = builder.build(value.to_string())?.map(|body| body.into());

    let res = client.request(req).await?;

    return Ok(());
}

async fn body_to_string(req: Response<Body>) -> Result<String, CustomError> {
    let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
    return Ok(String::from_utf8(body_bytes.to_vec())?);
}

/// Lookup keys if there are vapid keys in the file system and if not it will generate them
pub fn lookup_keys() -> Result<P256PublicKey, CustomError> {
    let privkey_mutex: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));

    thread::scope(|s| {
        let priv_key = privkey_mutex.clone();

        let handle = thread::spawn(move || {
            let mut private_key = priv_key.lock().expect("Could not lock private key");
            match File::open("vapid/private.key").ok() {
                Some(mut file_t) => {
                    file_t
                        .read_to_end(&mut private_key)
                        .expect("Could not read private key file");
                    return ();
                }
                None => {
                    let sec_key = ES256KeyPair::generate();
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

    return Ok(pub_key);
}

pub fn load_rustls_config() -> rustls::ServerConfig {
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

    return config
        .with_single_cert(cert_chain, keys.remove(0))
        .expect("Could not load cert");
}
