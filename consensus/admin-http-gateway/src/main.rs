// Copyright (c) 2018-2020 MobileCoin Inc.

//! An HTTP frontend for a Consensus Service's admin GRPC interface.

#![feature(proc_macro_hygiene, decl_macro)]

use grpcio::ChannelBuilder;
use mc_common::logger::{create_app_logger, log, o};
use mc_connection::ConnectionUriGrpcioChannel;
use mc_consensus_api::{consensus_admin, consensus_admin_grpc::ConsensusAdminApiClient, Empty};
use mc_util_uri::ConsensusAdminUri;
use rocket::{
    get, post,
    request::Form,
    response::{content, Redirect},
    routes, FromForm,
};
use rocket_contrib::json::Json;
use serde_derive::Serialize;
use std::{convert::TryFrom, sync::Arc};
use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
#[structopt(
    name = "mc-consensus-admin-http-gateway",
    about = "An HTTP frontend for a Consensus Service's admin GRPC interface."
)]
pub struct Config {
    /// Host to listen on.
    #[structopt(long, default_value = "127.0.0.1")]
    pub listen_host: String,

    /// Post to start webserver on.
    #[structopt(long, default_value = "9090")]
    pub listen_port: u16,

    /// Consensus service admin URI to connect to.
    #[structopt(long)]
    pub admin_uri: ConsensusAdminUri,
}

struct State {
    pub admin_api_client: ConsensusAdminApiClient,
}

#[get("/")]
fn index() -> content::Html<String> {
    content::Html(include_str!("../templates/index.html").to_owned())
}

#[derive(Serialize)]
struct JsonInfoResponse {
    build_info: serde_json::Value,
    config: serde_json::Value,
    network: serde_json::Value,
    rust_log: String,
}

impl TryFrom<&consensus_admin::GetInfoResponse> for JsonInfoResponse {
    type Error = String;

    fn try_from(src: &consensus_admin::GetInfoResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            build_info: serde_json::from_str(&src.build_info_json).map_err(|err| {
                format!(
                    "failed parsing build info '{}': {}",
                    src.build_info_json, err
                )
            })?,
            config: serde_json::from_str(&src.config_json)
                .map_err(|err| format!("failed parsing config '{}': {}", src.config_json, err))?,
            network: serde_json::from_str(&src.network_json)
                .map_err(|err| format!("failed parsing network '{}': {}", src.network_json, err))?,
            rust_log: src.rust_log.clone(),
        })
    }
}

#[get("/info")]
fn info(state: rocket::State<State>) -> Result<Json<JsonInfoResponse>, String> {
    let info = state
        .admin_api_client
        .get_info(&Empty::new())
        .map_err(|err| format!("Failed getting info: {}", err))?;

    Ok(Json(JsonInfoResponse::try_from(&info)?))
}

#[derive(FromForm)]
struct SetRustLogForm {
    rust_log: String,
}

#[post("/set-rust-log", data = "<form>")]
fn set_rust_log(
    state: rocket::State<State>,
    form: Form<SetRustLogForm>,
) -> Result<Redirect, String> {
    let mut req = consensus_admin::SetRustLogRequest::new();
    req.set_rust_log(form.rust_log.clone());

    let _resp = state
        .admin_api_client
        .set_rust_log(&req)
        .map_err(|err| format!("failed setting rust_log: {}", err))?;

    Ok(Redirect::to("/"))
}

#[get("/metrics")]
fn metrics(state: rocket::State<State>) -> Result<String, String> {
    let resp = state
        .admin_api_client
        .get_prometheus_metrics(&Empty::new())
        .map_err(|err| format!("failed getting metrics: {}", err))?;
    Ok(resp.metrics)
}

fn main() {
    mc_common::setup_panic_handler();
    let _sentry_guard = mc_common::sentry::init();

    let config = Config::from_args();

    let (logger, _global_logger_guard) = create_app_logger(o!());
    log::info!(
        logger,
        "Starting consensus admin HTTP gateway on {}:{}, connecting to {}",
        config.listen_host,
        config.listen_port,
        config.admin_uri
    );

    let env = Arc::new(grpcio::EnvBuilder::new().build());
    let ch =
        ChannelBuilder::default_channel_builder(env).connect_to_uri(&config.admin_uri, &logger);
    let admin_api_client = ConsensusAdminApiClient::new(ch);

    let rocket_config = rocket::Config::build(rocket::config::Environment::Production)
        .address(&config.listen_host)
        .port(config.listen_port)
        .unwrap();

    rocket::custom(rocket_config)
        .mount("/", routes![index, info, set_rust_log, metrics])
        .manage(State { admin_api_client })
        .launch();
}
