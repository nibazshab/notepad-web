use app::app::lib::main as running;

#[cfg(feature = "serverless")]
fn main() -> Result<(), vercel_runtime::Error> {
    running()
}

#[cfg(feature = "server")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    running()
}
