use std::time::Duration;

use fe2o3_amqp::{
    transaction::{Controller, Transaction, TransactionDischarge},
    types::primitives::Value,
    Connection, Delivery, Receiver, Sender, Session, Sendable,
};
use tracing::{instrument, Level};
use tracing_subscriber::FmtSubscriber;

const SASL_PLAIN: &str = "";
// const SASL_PLAIN: &str = "guest:guest";
const BASE_ADDR: &str = "localhost:5672";

// #[instrument]
async fn client_main() {
    let url = format!("amqp://{}@{}", SASL_PLAIN, BASE_ADDR);

    let mut connection = Connection::open("connection-1", &url[..]).await.unwrap();
    let mut session = Session::begin(&mut connection).await.unwrap();

    // // Test a regular receiver
    // let mut receiver = Receiver::attach(&mut session, "receiver-1", "q1")
    //     .await
    //     .unwrap();
    // let delivery: Delivery<Value> = receiver.recv().await.unwrap();
    // tracing::info!(message = ?delivery.message());
    // receiver.accept(&delivery).await.unwrap();

    // Test a regular sender
    let mut sender = Sender::attach(&mut session, "sender-1", "q1")
        .await
        .unwrap();
    sender.send("hello AMQP").await.unwrap();

    // Test creating a control link
    match Controller::attach(&mut session, "controller").await {
        Ok(mut controller) => {
            let mut txn = Transaction::declare(&mut controller, None).await.unwrap();

            tracing::info!("Transaction declared");

            let sendable = Sendable::builder()
                .message("Hello World")
                .settled(true)
                .build();
            txn.post(&mut sender, sendable).await.unwrap();

            let sendable = Sendable::builder()
                .message("Foo Bar")
                .settled(true)
                .build();
            txn.post(&mut sender, sendable).await.unwrap();

            txn.commit().await.unwrap();
            controller.close().await.unwrap();
        }
        Err(attach_error) => {
            tracing::error!(?attach_error)
        }
    }

    sender.close().await.unwrap();
    // receiver.close().await.unwrap();

    session.end().await.unwrap();
    connection.close().await.unwrap();
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    client_main().await;
}
