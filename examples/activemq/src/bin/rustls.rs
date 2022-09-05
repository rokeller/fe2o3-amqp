//! This example assumes you have an ActiveMQ instant that supports AMQP 1.0
//! running on your localhost
//! 
//! `ActiveMQ` uses alternative TLS establishment (ie. establish TLS without 
//! exchanging ['A', 'M', 'Q', 'P', '2', '1', '0', '0'] header). The user should
//! follow the alternative TLS establishment example which is also copied below.
//! 
//! Please note that you may need to explicitly set you `ActiveMQ` to use TLSv1.2 or higher
//! in the xml configuration file.
//! 
//! ```xml
//! <transportConnector name="amqp+ssl" uri="amqp+ssl://0.0.0.0:5671?transport.enabledProtocols=TLSv1.2"/>
//! ```

use std::sync::Arc;

use fe2o3_amqp::Connection;
use fe2o3_amqp::Sender;
use fe2o3_amqp::Session;
use fe2o3_amqp::sasl_profile::SaslProfile;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

#[tokio::main]
async fn main() {
    let addr = "localhost:5671";
    let domain = rustls::ServerName::try_from("localhost").unwrap();
    let stream = TcpStream::connect(addr).await.unwrap();

    let mut root_store = rustls::RootCertStore::empty();
    root_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));
    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(config));
    let tls_stream = connector.connect(domain, stream).await.unwrap();

    let mut connection = Connection::builder()
        .container_id("connection-1")
        .sasl_profile(SaslProfile::Plain {
            username: "guest".into(),
            password: "guest".into(),
        })
        .open_with_stream(tls_stream)
        .await
        .unwrap();

    let mut session = Session::begin(&mut connection).await.unwrap();
    let mut sender = Sender::attach(&mut session, "rust-sender-link-1", "q1")
        .await
        .unwrap();

    let outcome = sender.send("hello AMQP").await.unwrap();
    outcome.accepted_or_else(|outcome| outcome).unwrap();

    sender.close().await.unwrap();
    session.end().await.unwrap();
    connection.close().await.unwrap();
}
