//! Listeners

pub mod builder;
pub mod connection;
pub mod link;
pub mod sasl_acceptor;
pub mod session;

pub use self::connection::*;

// /// Trait for listeners
// #[async_trait]
// pub trait Listener {
//     /// Type of accepted IO stream
//     type Stream: AsyncRead + AsyncWrite + std::fmt::Debug + Send + Unpin + 'static;
//     // type Stream;

//     /// Type for local addr
//     type Addr;

//     /// Obtain the local address
//     fn local_addr(&self) -> Result<Self::Addr, io::Error>;

//     /// Accept an incoming stream
//     async fn accept(&self) -> Result<Self::Stream, io::Error>;
// }

// #[async_trait]
// impl Listener for TcpListener {
//     type Stream = TcpStream;
//     type Addr = SocketAddr;

//     fn local_addr(&self) -> Result<Self::Addr, io::Error> {
//         self.local_addr()
//     }

//     async fn accept(&self) -> Result<Self::Stream, io::Error> {
//         self.accept().await.map(|(socket, _)| socket)
//     }
// }
