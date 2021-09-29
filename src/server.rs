use tonic::{transport::Server, Request, Response, Status};

use helloworld::transaction_service_server::{TransactionService, TransactionServiceServer};
use helloworld::{TransactionRequest, TransactionResponse};
use std::env;
use uuid::Uuid;

use tracing::{debug, info, span, warn, Level};
use opentelemetry::global;
use opentelemetry::sdk::{
    trace::{self, IdGenerator, Sampler},
    Resource,
};
use tracing_attributes::instrument;
use tracing_subscriber;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

use redis::{AsyncCommands, RedisResult};

pub mod helloworld {
    tonic::include_proto!("helloworld");
}

#[derive(Debug)]
pub struct MyTransactionService {
    client: redis::Client,
}

#[tracing::instrument]
async fn add_pan(last_pan: String, client: &redis::Client) -> RedisResult<()> {
    let mut con = client.get_async_connection().await?;

    con.set("my_key", last_pan).await?;

    Ok(())
}

#[tonic::async_trait]
impl TransactionService for MyTransactionService {
    #[tracing::instrument]
    async fn make_transaction(
        &self,
        request: Request<TransactionRequest>,
    ) -> Result<Response<TransactionResponse>, Status> {
        info!(
            message = "Got a transaction request",
            request = %request.get_ref().pan
        );

        let my_uuid = Uuid::new_v4();

        let reply = TransactionResponse {
            transaction_id: my_uuid.to_string(),
        };

        add_pan(my_uuid.to_string(), &self.client)
            .await
            .expect("upload pan");

        Ok(Response::new(reply))
    }
}

fn add_tracing() {
    global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());

    // let tracer = opentelemetry_jaeger::new_pipeline()
    //     .with_service_name("my_app")
    //     .install_batch(opentelemetry::runtime::Tokio)
    //     .expect("Error initializing Jaeger exporter");

    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("my_app")
        .install_simple()
        .expect("Could not register tracer");

    let otel_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer);
    
    let subscriber = Registry::default()
        .with(otel_layer)
        .try_init()
        .expect("Could not add tracing layer");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
 
    add_tracing();

    let redis_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| r#"redis://localhost:6379"#.to_string());

    let addr: String = "0.0.0.0:50051".parse().unwrap();

    info!("Redis server at {}", &redis_addr);
    let client = redis::Client::open(redis_addr)?;

    let greeter = MyTransactionService { client };

    info!("Orchestrator microservice listening on {}", addr);

    let addr = "[::1]:50051".parse().unwrap();
    info!(message = "Starting server.", %addr);

    let service = TransactionServiceServer::new(greeter)
        .send_gzip()
        .accept_gzip();

    Server::builder()
        .trace_fn(|_| tracing::info_span!("helloworld_server"))
        .add_service(service)
        .serve(addr)
        .await?;

    global::shutdown_tracer_provider();

    Ok(())
}
