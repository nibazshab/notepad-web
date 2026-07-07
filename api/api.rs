use app::app::app;

#[cfg(feature = "serverless")]
fn main() -> Result<(), vercel_runtime::Error> {
    app()
}

#[cfg(feature = "server")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    app()
}
