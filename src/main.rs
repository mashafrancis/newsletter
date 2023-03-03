use newsletter::configuration::get_configuration;
use newsletter::email_client::EmailClient;
use newsletter::startup::{build, run, Application};
use newsletter::telemetry::{get_subscriber, init_subscriber};
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
	let subscriber = get_subscriber("newsletter".into(), "info".into(), std::io::stdout);
	init_subscriber(subscriber);

	let configuration = get_configuration().expect("Failed to read configuration.");
	let application = Application::build(configuration).await?;
	application.run_until_stopped().await?;

	Ok(())
}
