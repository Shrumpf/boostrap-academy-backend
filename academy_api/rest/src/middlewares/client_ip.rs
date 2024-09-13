use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use axum::{
    extract::{ConnectInfo, Request},
    middleware::{from_fn, Next},
    Router,
};
use tracing::{debug, error, warn};

use crate::RealIpConfig;

pub fn add<S: Clone + Send + Sync + 'static>(
    real_ip_config: Option<Arc<RealIpConfig>>,
) -> impl FnOnce(Router<S>) -> Router<S> {
    |router| {
        router.layer(from_fn(move |mut request: Request, next: Next| {
            let client_ip = ClientIp::from_request(&request, real_ip_config.as_deref());
            request.extensions_mut().insert(client_ip);
            next.run(request)
        }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClientIp(pub IpAddr);

impl ClientIp {
    fn from_request(request: &Request, real_ip_config: Option<&RealIpConfig>) -> Self {
        let client_ip = request
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .unwrap()
            .ip();

        let Some(RealIpConfig { header, set_from }) = real_ip_config else {
            return Self(client_ip);
        };

        let header_value = request.headers().get(header);

        if *set_from != client_ip {
            if let Some(header_value) = header_value {
                debug!(%client_ip, ?header_value, "ignoring real ip header value from untrusted source");
            }
            return Self(client_ip);
        }

        let Some(header_value) = header_value else {
            warn!(%client_ip, "real ip header not found");
            return Self(client_ip);
        };

        let Some(real_ip) = header_value
            .to_str()
            .ok()
            .and_then(|real_ip| real_ip.parse().ok())
        else {
            error!(%client_ip, ?header_value, "failed to parse real ip header value");
            return Self(client_ip);
        };

        ClientIp(real_ip)
    }
}
