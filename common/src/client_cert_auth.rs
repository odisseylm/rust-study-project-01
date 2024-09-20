use std::{
    sync::Arc,
};
use anyhow::anyhow;
use log::{error, info};
use rustls::{
    DigitallySignedStruct, DistinguishedName,
    client::danger::HandshakeSignatureValid,
    server::danger::ClientCertVerified,
};
use rustls_pki_types::CertificateDer;
use crate::{
    cfg::ServerConf,
    client_auth_cert_info::extract_client_auth_cert_info_from_cert,
};
//--------------------------------------------------------------------------------------------------



pub async fn server_rustls_with_ssl_cert_client_auth_config<Conf: ServerConf>(server_conf: &Conf)
    -> anyhow::Result<axum_server::tls_rustls::RustlsConfig> {

    use std::sync::Arc;
    use axum_server::tls_rustls::RustlsConfig;

    let cert = server_conf.server_ssl_cert()
        .ok_or_else(||anyhow!("No server SSL cert (for SSL mode)")) ?
        .as_secure_string() ?;
    let key = server_conf.server_ssl_key()
        .ok_or_else(||anyhow!("No server SSL key (for SSL mode)")) ?
        .as_secure_string() ?;
    let client_auth_ca_cert = server_conf.client_auth_ssl_ca_cert()
        .ok_or_else(||anyhow!("No server SSL key (for SSL mode with client cert auth support)")) ?
        .as_secure_string() ?;

    let cert = Vec::<u8>::from(cert.as_ref().as_bytes());
    let key = Vec::<u8>::from(key.as_ref().as_bytes());
    let client_auth_ca_cert: Vec<u8> = Vec::<u8>::from(client_auth_ca_cert.as_ref().as_bytes());

    let serv_conf = config_from_pem(cert, key, client_auth_ca_cert) ?;
    let rust_tls_config = RustlsConfig::from_config(Arc::new(serv_conf));
    Ok(rust_tls_config)
}

//--------------------------------------------------------------------------------------------------
//
// Code copied from axum-server since I don't know how to add custom
// auth client cert verifier.
//
fn config_from_pem(cert: Vec<u8>, key: Vec<u8>, client_auth_ca_cert: Vec<u8>) -> std::io::Result<rustls::ServerConfig> {

    use rustls_pemfile::Item;

    let cert = rustls_pemfile::certs(&mut cert.as_ref())
        .map(|it| it.map(|it| it.to_vec()))
        .collect::<Result<Vec<_>, _>>()?;

    // Check the entire PEM file for the key in case it is not first section
    let mut key_vec: Vec<Vec<u8>> = rustls_pemfile::read_all(&mut key.as_ref())
        .filter_map(|i| match i.ok()? {
            Item::Sec1Key(key) => Some(key.secret_sec1_der().to_vec()),
            Item::Pkcs1Key(key) => Some(key.secret_pkcs1_der().to_vec()),
            Item::Pkcs8Key(key) => Some(key.secret_pkcs8_der().to_vec()),
            _ => None,
        })
        .collect();

    // Make sure file contains only one key
    if key_vec.len() != 1 {
        return Err(io_other("private key format not supported"));
    }

    let key: Vec<u8> = key_vec.pop()
        .ok_or_else(||io_other("No cert key")) ?;

    config_from_der(cert, key, client_auth_ca_cert)
}

fn io_other<E: Into<Box<dyn std::error::Error + Send + Sync>>>(error: E) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, error)
}


// Code copied and ADOPTED from axum-server
// since I don't know how to add custom auth client cert verifier.
//
// fn config_from_der(cert: Vec<Vec<u8>>, key: Vec<u8>, client_auth_ca_cert: &SslConfValue) -> std::io::Result<rustls::ServerConfig> {
fn config_from_der(cert: Vec<Vec<u8>>, key: Vec<u8>, client_auth_ca_cert: Vec<u8>)
    -> std::io::Result<rustls::ServerConfig> {

    let cert = cert.into_iter().map(CertificateDer::from).collect();
    let key = rustls_pki_types::PrivateKeyDer::try_from(key).map_err(io_other)?;

    use rustls::RootCertStore;
    use rustls::server::WebPkiClientVerifier;
    use std::sync::Arc;

    let mut root_s: RootCertStore = RootCertStore::empty();
    //
    // let client_auth_cert = client_auth_ca_cert.as_secure_string() ?;
    // let client_auth_cert = match client_auth_cert {
    //     Cow::Borrowed(secure_str_ref) =>
    //         Vec::from(secure_str_ref.as_bytes()),
    //     Cow::Owned(secure_str) =>
    //         secure_str.into_bytes(),
    // };
    // let client_auth_cert = client_auth_cert.as_bytes();

    let ca_bytes = client_auth_ca_cert;
    // let ca = ca_bytes.into_iter().map(rustls::pki_types::CertificateDer::from).collect();
    // let ca = rustls::pki_types::CertificateDer::from(ca_bytes);

    let mut certs: &mut dyn std::io::BufRead = &mut ca_bytes.as_slice();

    for cert in rustls_pemfile::certs(&mut certs).collect::<Result<Vec<_>, _>>()? {
        root_s
            .add(cert)
            // .map_err(|_| TlsError::CertificateParseError)
            .map_err(|err|io_other(err))
            // .map_err(|_err|io_other("Error of adding client cert to roots."))
            ?;
    }

    let verifier = WebPkiClientVerifier::builder(Arc::new(root_s))
        .allow_unauthenticated()
        .build()
        // .map_err(|err| io_other("Error of creating WebPkiClientVerifier."));
        .map_err(|err| io_other(err))
        ?;

    let mut config = rustls::ServerConfig::builder()
        .with_client_cert_verifier(Arc::new(MyClientCertVerifier { delegate: verifier }))
        .with_single_cert(cert, key)
        .map_err(io_other)?;

    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Ok(config)
}


#[derive(Debug)]
struct MyClientCertVerifier {
    delegate: Arc<dyn rustls::server::danger::ClientCertVerifier>,
}

impl rustls::server::danger::ClientCertVerifier for MyClientCertVerifier {
    fn offer_client_auth(&self) -> bool {
        self.delegate.offer_client_auth()
    }

    fn client_auth_mandatory(&self) -> bool {
        self.delegate.client_auth_mandatory()
    }

    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        self.delegate.root_hint_subjects()
    }


    fn verify_client_cert(&self,
                          end_entity: &CertificateDer<'_>,
                          intermediates: &[CertificateDer<'_>],
                          now: rustls_pki_types::UnixTime)
        -> Result<ClientCertVerified, rustls::Error> {

        let res = self.delegate.verify_client_cert(end_entity, intermediates, now);
        match res {
            Ok(ref _client_cert_verified) => {
                info!("Client cert verification succeeded");
                let der_bytes = end_entity.as_ref();

                let cert_info = extract_client_auth_cert_info_from_cert(der_bytes);
                match cert_info {
                    Ok(client_auth_cert_info) => {
                        info!("cert_info: {client_auth_cert_info:?}");
                        // TODO: put client_auth_cert_info to request context
                    }
                    Err(err) => {
                        error!("Client verification succeeded, but extracting cert info failed ({err:?})");
                        // return Err(rustls::Error::Other(rustls::OtherError(Arc::new(err)))) // TODO: Why?
                        return Err(rustls::Error::General(err.to_string()))
                    }
                }
            }
            Err(ref err) => {
                error!("Client cert verification failed: {err:?}");
            }
        }
        res
    }

    fn verify_tls12_signature(&self,
                              message: &[u8],
                              cert: &CertificateDer<'_>,
                              dss: &DigitallySignedStruct)
                              -> Result<HandshakeSignatureValid, rustls::Error> {
        self.delegate.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(&self,
                              message: &[u8],
                              cert: &CertificateDer<'_>,
                              dss: &DigitallySignedStruct)
                              -> Result<HandshakeSignatureValid, rustls::Error> {
        self.delegate.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.delegate.supported_verify_schemes()
    }
}
