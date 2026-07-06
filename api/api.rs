#[cfg(feature = "serverless")]
use app::serverless::app;

#[cfg(feature = "server")]
use app::server::app;

#[cfg(feature = "serverless")]
fn main() -> Result<(), vercel_runtime::Error> {
    app()
}

#[cfg(feature = "server")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    app()
}
