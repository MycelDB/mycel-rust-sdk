use std::fs;

use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint, Identity};

use crate::{
    config::Config,
    error::{Error, Result},
};

pub async fn connect_channel(cfg: &Config) -> Result<Channel> {
    let mut endpoint = Endpoint::from_shared(cfg.endpoint_uri())
        .map_err(|_| Error::InvalidEndpoint(cfg.endpoint_uri()))?;

    if cfg.tls {
        let tls = tls_config(cfg)?;
        endpoint = endpoint.tls_config(tls)?;
    }

    Ok(endpoint.connect().await?)
}

fn tls_config(cfg: &Config) -> Result<ClientTlsConfig> {
    if cfg.tls_insecure_skip_verify {
        return Err(Error::InsecureSkipVerifyUnsupported);
    }
    if cfg.tls_client_cert_file.is_empty() != cfg.tls_client_key_file.is_empty() {
        return Err(Error::PartialClientCertificate);
    }

    let mut tls = ClientTlsConfig::new().with_native_roots();

    if !cfg.tls_server_name.is_empty() {
        tls = tls.domain_name(cfg.tls_server_name.clone());
    }

    if !cfg.tls_ca_file.is_empty() {
        let pem = read_file(&cfg.tls_ca_file)?;
        tls = tls.ca_certificate(Certificate::from_pem(pem));
    }

    if !cfg.tls_client_cert_file.is_empty() {
        let cert = read_file(&cfg.tls_client_cert_file)?;
        let key = read_file(&cfg.tls_client_key_file)?;
        tls = tls.identity(Identity::from_pem(cert, key));
    }

    Ok(tls)
}

fn read_file(path: &str) -> Result<Vec<u8>> {
    fs::read(path).map_err(|source| Error::ReadFile {
        path: path.to_string(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_partial_client_certificate() {
        let cfg = Config {
            tls: true,
            tls_client_cert_file: "client.pem".into(),
            ..Default::default()
        };
        assert!(matches!(
            tls_config(&cfg),
            Err(Error::PartialClientCertificate)
        ));
    }
}
