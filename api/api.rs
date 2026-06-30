use app::{driver, router};

#[cfg(feature = "serverless")]
#[tokio::main]
async fn main() -> Result<(), vercel_runtime::Error> {
    use tower::ServiceBuilder;
    use vercel_runtime::axum::VercelLayer;

    driver().await?;

    let router = router();

    let app = ServiceBuilder::new()
        .layer(VercelLayer::new())
        .service(router);

    vercel_runtime::run(app).await
}

#[cfg(feature = "server")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use std::net::SocketAddr;
    use tokio::net::TcpListener;

    let mut args = env::args().skip(1).peekable();
    let mut port: Option<u16> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--port" | "-p" => {
                port = args.next().map(|v| v.parse::<u16>()).transpose()?;
            }
            _ => {
                return Err(format!("unknown argument: {arg}").into());
            }
        }
    }

    let _port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let port = port.unwrap_or(_port);

    driver().await?;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;

    let router = router().into_make_service_with_connect_info::<SocketAddr>();

    axum::serve(listener, router).await?;

    Ok(())
}
