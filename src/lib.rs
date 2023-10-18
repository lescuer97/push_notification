use actix_web::web::Data;
use base64ct::{Base64UrlUnpadded, Encoding};
use hyper::{Body, Client, Response};
use hyper_rustls::HttpsConnectorBuilder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use web_push_native::{jwt_simple::prelude::ES256KeyPair, p256::PublicKey, Auth, WebPushBuilder};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PushSubscriptionOptions {
    pub auth: String,
    pub p256dh: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PushSubscription {
    pub endpoint: String,
    pub expirationTime: Option<i64>,
    pub keys: PushSubscriptionOptions,
}

pub async fn push_message_request(post: Data<PushSubscription>) {
    let vapid_env = std::env::var("VAPID_PRIVATE").expect("VAPID_PRIVATE to be set");

    let bytes = Base64UrlUnpadded::decode_vec(vapid_env.as_str()).expect("this to be valid base64");

    let vapid_private = ES256KeyPair::from_bytes(&bytes).expect("this to be a valid private key");

    let content = r#"{"title": "Portugal vs. Denmark",
                "data":"hello world from app server"}"#;

    let builder = WebPushBuilder::new(
        post.endpoint.parse().unwrap(),
        PublicKey::from_sec1_bytes(
            &Base64UrlUnpadded::decode_vec(post.keys.p256dh.as_str()).unwrap(),
        )
        .unwrap(),
        Auth::clone_from_slice(&Base64UrlUnpadded::decode_vec(post.keys.auth.as_str()).unwrap()),
    )
    .with_vapid(&vapid_private, "mailto:leo@leito.dev");

    let https = HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_only()
        .enable_http1()
        .build();

    let client: Client<_, Body> = Client::builder().build(https);

    // Parse the string of data into serde_json::Value.
    let value: Value = serde_json::from_str(&content).unwrap();

    let req = builder
        .build(value.to_string())
        .unwrap()
        .map(|body| body.into());

    let res = client.request(req).await.expect("made request to server");

    println!("res: {:?}", res);
    println!("res_body: {:?}", body_to_string(res).await);
}

async fn body_to_string(req: Response<Body>) -> String {
    let body_bytes = hyper::body::to_bytes(req.into_body())
        .await
        .expect("body to bytes");
    String::from_utf8(body_bytes.to_vec()).unwrap()
}
