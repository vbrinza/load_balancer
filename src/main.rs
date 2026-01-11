use async_trait::async_trait;
use pingora::prelude::*;
use std::sync::Arc;

fn main() {
    let mut my_lb = Server::new(None).unwrap();
    my_lb.bootstrap();
    let upstreams = LoadBalancer::try_from_iter(["0.0.0.0:8000", "0.0.0.0:8001"]).unwrap();

    let mut lb = http_proxy_service(&my_lb.configuration, LB(Arc::new(upstreams)));
    lb.add_tcp("0.0.0.0:6188");

    my_lb.add_service(lb);
    my_lb.run_forever();
}

pub struct LB(Arc<LoadBalancer<RoundRobin>>);

#[async_trait]
impl ProxyHttp for LB {
    type CTX = ();
    fn new_ctx(&self) -> () {
        ()
    }

    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let upstream = self.0.select(b"", 256).unwrap();

        eprintln!("upstream peer is: {:?}", upstream);

        let peer = Box::new(HttpPeer::new(upstream, false, "".to_string()));
        eprintln!("Created peer: {:?}", peer);
        Ok(peer)
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        println!("Sending request to upstream: {:?}", upstream_request.uri);
        Ok(())
    }
}
