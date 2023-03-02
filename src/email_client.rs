use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

pub struct EmailClient {
	http_client: Client,
	base_url: String,
	sender: SubscriberEmail,
	authorization_token: Secret<String>,
}

impl EmailClient {
	pub fn new(
		base_url: String,
		sender: SubscriberEmail,
		authorization_token: Secret<String>,
	) -> Self {
		let http_client = Client::builder()
			.timeout(std::time::Duration::from_secs(10))
			.build()
			.unwrap();
		Self {
			http_client,
			base_url,
			sender,
			authorization_token,
		}
	}

	pub async fn send_email(
		&self,
		recipient: SubscriberEmail,
		subject: &str,
		html_content: &str,
		text_content: &str,
	) -> Result<(), reqwest::Error> {
		let url = format!("{}/email", self.base_url);
		let request_body = SendEmailRequest {
			from: self.sender.as_ref(),
			to: recipient.as_ref(),
			subject,
			html_body: html_content,
			text_body: text_content,
		};
		self.http_client
			.post(&url)
			.json(&request_body)
			.header(
				"X-Postmark-Server-Token",
				self.authorization_token.expose_secret(),
			)
			.json(&request_body)
			.send()
			.await?
			.error_for_status()?;
		Ok(())
	}
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
	from: &'a str,
	to: &'a str,
	subject: &'a str,
	html_body: &'a str,
	text_body: &'a str,
}

#[cfg(test)]
mod tests {
	use crate::domain::SubscriberEmail;
	use crate::email_client::EmailClient;
	use claim::{assert_err, assert_ok};
	use fake::faker::internet::en::SafeEmail;
	use fake::faker::lorem::en::{Paragraph, Sentence};
	use fake::{Fake, Faker};
	use secrecy::Secret;
	use wiremock::matchers::{any, header, header_exists, method, path};
	use wiremock::{Mock, MockServer, Request, ResponseTemplate};

	struct SendEmailBodyMatcher;

	impl wiremock::Match for SendEmailBodyMatcher {
		fn matches(&self, request: &Request) -> bool {
			let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
			if let Ok(body) = result {
				dbg!(&body);
				// Check that all the mandatory fields are populated
				// without inspecting the field values
				body.get("From").is_some()
					&& body.get("To").is_some()
					&& body.get("Subject").is_some()
					&& body.get("HtmlBody").is_some()
					&& body.get("TextBody").is_some()
			} else {
				false
			}
		}
	}

	/// Generate a random email subject
	fn subject() -> String {
		Sentence(1..2).fake()
	}

	/// Generate a random email content
	fn content() -> String {
		Paragraph(1..10).fake()
	}

	/// Generate a random subscriber email
	fn email() -> SubscriberEmail {
		SubscriberEmail::parse(SafeEmail().fake()).unwrap()
	}

	/// Get a test instance of `EmailClient`.
	fn email_client(base_url: String) -> EmailClient {
		EmailClient::new(base_url, email(), Secret::new(Faker.fake()))
	}

	#[tokio::test]
	async fn send_email_sends_the_expected_request() {
		// Arrange
		let mock_server = MockServer::start().await;
		let email_client = email_client(mock_server.uri());

		Mock::given(header_exists("X-Postmark-Server-Token"))
			.and(header("Content-Type", "application/json"))
			.and(path("/email"))
			.and(method("POST"))
			.and(SendEmailBodyMatcher)
			.respond_with(ResponseTemplate::new(200))
			.expect(1)
			.mount(&mock_server)
			.await;

		// Act
		let _ = email_client
			.send_email(email(), &subject(), &content(), &content())
			.await;

		// Assert
	}

	#[tokio::test]
	async fn send_email_succeeds_if_the_server_returns_200() {
		// Arrange
		let mock_server = MockServer::start().await;
		let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
		let email_client = EmailClient::new(mock_server.uri(), sender, Secret::new(Faker.fake()));

		let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
		let subject: String = Sentence(1..2).fake();
		let content: String = Paragraph(1..10).fake();

		// We do not copy in all the matchers we have in the other test.
		// The purpose of this test is not to assert on the request we
		// are sending out!
		// We add the bare minimum needed to trigger the path we want
		// to test in `send_email`.
		Mock::given(any())
			.respond_with(ResponseTemplate::new(200))
			.expect(1)
			.mount(&mock_server)
			.await;

		// Act
		let outcome = email_client
			.send_email(subscriber_email, &subject, &content, &content)
			.await;

		// Assert
		assert_ok!(outcome);
	}

	#[tokio::test]
	async fn send_email_times_out_if_the_server_takes_too_long() {
		// Arrange
		let mock_server = MockServer::start().await;
		let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
		let email_client = EmailClient::new(mock_server.uri(), sender, Secret::new(Faker.fake()));

		let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
		let subject: String = Sentence(1..2).fake();
		let content: String = Paragraph(1..10).fake();

		let response = ResponseTemplate::new(200)
			// 3 minutes!
			.set_delay(std::time::Duration::from_secs(180));
		Mock::given(any())
			.respond_with(response)
			.expect(1)
			.mount(&mock_server)
			.await;

		// Act
		let outcome = email_client
			.send_email(subscriber_email, &subject, &content, &content)
			.await;

		// Assert
		assert_err!(outcome);
	}
}