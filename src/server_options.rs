use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub enum Authorization {
    None,
    Basic(String),
}

#[derive(Debug, Clone)]
pub struct TlsOptions {
    /// Path to file containing PEM encoded certificate chain.
    pub certificate_chain_file: String,
    /// Path to file containing private key.
    pub key_file: String,
    /// Path to file containing a CA certificate for client certificates.
    pub client_certificate_ca_file: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub addr: SocketAddr,
    pub authorization: Authorization,
    pub tls_options: Option<TlsOptions>,
}
