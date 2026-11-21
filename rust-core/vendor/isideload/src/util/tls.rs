//! Public TLS roots for the standalone Linux installer, alongside system CAs.
//! Native application builds retain platform trust unless they opt in.

pub fn configure(
    builder: reqwest::ClientBuilder,
) -> Result<reqwest::ClientBuilder, reqwest::Error> {
    #[cfg(feature = "bundled-roots")]
    let builder = builder.tls_certs_merge(
        webpki_root_certs::TLS_SERVER_ROOT_CERTS
            .iter()
            .map(|certificate| reqwest::Certificate::from_der(certificate.as_ref()))
            .collect::<Result<Vec<_>, _>>()?,
    );
    Ok(builder)
}
