use tonic::{transport::Server, Request, Response, Status};

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{TransactionRequest, TransactionResponse};
use uuid::Uuid;
use std::env;

use redis::{AsyncCommands, RedisResult};


pub mod hello_world {
    tonic::include_proto!("helloworld");
}


pub struct MyGreeter {
    client: redis::Client,
}


// fn insert_pan(last_pan: String, client: &redis::Client) -> RedisResult<()> {
//     let mut con = client.get_connection().expect("conn");

//     con.set("my_key", last_pan)?;

//     Ok(())
// }

async fn add_pan(last_pan: String, client: &redis::Client) -> RedisResult<()> {
    let mut con = client.get_async_connection().await?;

    con.set("my_key", last_pan).await?;

    Ok(())
}

#[tonic::async_trait]
impl Greeter for MyGreeter {

    async fn make_transaction(
        &self,
        request: Request<TransactionRequest>,
    ) -> Result<Response<TransactionResponse>, Status> {

        println!("Got a transaction request: {:?}", request);

        let my_uuid = Uuid::new_v4();

        let reply = hello_world::TransactionResponse {
            transaction_id: my_uuid.to_string()
        };

        add_pan(my_uuid.to_string(), &self.client).await.expect("upload pan");

        Ok(Response::new(reply))
    }
    

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let redis_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| r#"redis://localhost:6379"#.to_string());

    let addr = "0.0.0.0:50051".parse().unwrap();

    let client = redis::Client::open(redis_addr)?;

    let greeter = MyGreeter {
        client,
    };

    println!("Awesome microservice listening on {}", addr);

    let service = GreeterServer::new(greeter).send_gzip().accept_gzip();

    Server::builder()
        .add_service(service)
        .serve(addr)
        .await?;

    Ok(())
}