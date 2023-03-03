use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
	// Arrange
	let app = spawn_app().await;
	let client = reqwest::Client::new();
	let body = "name=Blah&email=blah%40gmail.com";

	// Act
	let response = app.post_subscriptions(body.into()).await;

	// Assert
	assert_eq!(200, response.status().as_u16());

	let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
		.fetch_one(&app.db_pool)
		.await
		.expect("Failed to fetch saved subscription.");

	assert_eq!(saved.email, "blah@gmail.com");
	assert_eq!(saved.name, "Blah");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
	// Arrange
	let app = spawn_app().await;
	let client = reqwest::Client::new();
	let test_cases = vec![
		("name=Blah", "missing the email"),
		("email=blah%40gmail.com", "missing the name"),
		("", "missing both name and email"),
	];

	for (invalid_body, error_message) in test_cases {
		// Act
		let response = app.post_subscriptions(invalid_body.into()).await;

		// Assert
		assert_eq!(
			400,
			response.status().as_u16(),
			// Additional customised error message on test failure
			"The API did not fail with 400 Bad Request when the payload was {}.",
			error_message
		);
	}
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
	// Arrange
	let app = spawn_app().await;
	let client = reqwest::Client::new();
	let test_cases = vec![
		("name=&email=ursula_le_guin%40gmail.com", "empty name"),
		("name=Ursula&email=", "empty email"),
		("name=Ursula&email=definitely-not-an-email", "invalid email"),
	];

	for (body, description) in test_cases {
		// Act
		let response = app.post_subscriptions(body.into()).await;

		// Assert
		assert_eq!(
			// Not 200 anymore!
			400,
			response.status().as_u16(),
			"The API did not return a 400 Bad Request when the payload was {}.",
			description
		);
	}
}
