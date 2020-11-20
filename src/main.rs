#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::sync::Arc;
use actix_web::body::Body;
use actix_web::middleware::Logger;
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use tokio::sync::{mpsc, RwLock};

mod error;
mod helper;
use helper::{Crate, CrateReq};

lazy_static! {
    static ref ACTIVE_DOWNLOADS: Arc<RwLock<HashMap<CrateReq, Arc<Crate>>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

///
/// With this as config in crates.io-index
/// ```json
/// {
///     "dl": "https://bucket-cdn/{crate}/{version}",
///     "api": "https://crates.io"
/// }
/// ```
/// Upyun will redirect 404 (non-exist) crate to given address configured
/// replace `$_URI` with the path part `/{crate}/{version}`
#[get("/sync/{crate}/{version}")]
async fn sync(web::Path(krate_req): web::Path<CrateReq>) -> HttpResponse {
    format!("{:?}", krate_req);
    let task = Crate::create(krate_req).await.unwrap();
    let (tx, rx) = mpsc::unbounded_channel::<Result<bytes::Bytes, ()>>();
    task.tee(tx);
    HttpResponse::Ok()
        .content_type("application/x-tar")
        .streaming(rx)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("config/log4rs.yml", Default::default()).unwrap();
    HttpServer::new(|| App::new().wrap(Logger::default()).service(sync))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}