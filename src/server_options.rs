use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub enum Authorization {
    None,
    Basic(String),
}

#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub addr: SocketAddr,
    pub authorization: Authorization,
}
