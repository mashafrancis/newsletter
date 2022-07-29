use newsletter::configuration::get_configuration;
use newsletter::startup::run;
use newsletter::telemetry::{get_subscriber, init_subscriber};
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
	let subscriber = get_subscriber("newsletter".into(), "info".into(), std::io::stdout);
	init_subscriber(subscriber);

	// Panic if we can't read configuration
	let configuration = get_configuration().expect("Failed to read configuration.");
	let connection_pool = PgPoolOptions::new().connect_lazy_with(configuration.database.with_db());

	let address = format!(
		"{}:{}",
		configuration.application.host, configuration.application.port
	);
	let listener = TcpListener::bind(address)?;
	run(listener, connection_pool)?.await?;
	Ok(())
}
