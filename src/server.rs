use tonic::{transport::Server, Request, Response, Status};

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{TransactionRequest, TransactionResponse};
use uuid::Uuid;
use std::env;

use redis::{Commands, RedisResult};


pub mod hello_world {
    tonic::include_proto!("helloworld");
}


pub struct MyGreeter {

    client: redis::Client,
}


fn insert_pan(last_pan: String, client: &redis::Client) -> RedisResult<()> {
    let mut con = client.get_connection().expect("conn");

    con.set("my_key", last_pan)?;

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


        let _ = insert_pan(my_uuid.to_string(), &self.client);

        Ok(Response::new(reply))
    }
    

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // let args: Vec<String> = env::args().collect();

    // let redis_addr = &args[1];

    let addr = "0.0.0.0:50051".parse().unwrap();

    let client = redis::Client::open("redis://127.0.0.1/")?;

    let greeter = MyGreeter {
        client: client,
    };

    println!("Awesome microservice listening on {}", addr);

    let service = GreeterServer::new(greeter).send_gzip().accept_gzip();

    Server::builder()
        .add_service(service)
        .serve(addr)
        .await?;

    Ok(())
}