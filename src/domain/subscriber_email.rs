use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
	pub fn parse(s: String) -> Result<SubscriberEmail, String> {
		if validate_email(&s) {
			Ok(Self(s))
		} else {
			Err(format!("{} is not a valid subscriber email.", s))
		}
	}
}

impl AsRef<str> for SubscriberEmail {
	fn as_ref(&self) -> &str {
		&self.0
	}
}

impl std::fmt::Display for SubscriberEmail {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

#[cfg(test)]
mod tests {
	use super::SubscriberEmail;
	use argon2::password_hash::rand_core::SeedableRng;
	use claims::assert_err;
	use fake::faker::internet::en::SafeEmail;
	use fake::Fake;
	use rand::rngs::StdRng;

	#[test]
	fn empty_string_is_rejected() {
		let email = "".to_string();
		assert_err!(SubscriberEmail::parse(email));
	}

	#[test]
	fn email_missing_at_symbol_is_rejected() {
		let email = "ursuladomain.com".to_string();
		assert_err!(SubscriberEmail::parse(email));
	}

	#[test]
	fn email_missing_subject_is_rejected() {
		let email = "@domain.com".to_string();
		assert_err!(SubscriberEmail::parse(email));
	}

	#[derive(Debug, Clone)]
	struct ValidEmailFixture(pub String);

	impl quickcheck::Arbitrary for ValidEmailFixture {
		fn arbitrary(g: &mut quickcheck::Gen) -> Self {
			let mut rand_slice: [u8; 32] = [0; 32];
			for i in 0..32 {
				rand_slice[i] = u8::arbitrary(g)
			}
			let mut seed = StdRng::from_seed(rand_slice);
			let email = SafeEmail().fake_with_rng(&mut seed);
			Self(email)
		}
	}

	#[quickcheck_macros::quickcheck]
	fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
		SubscriberEmail::parse(valid_email.0).is_ok()
	}
}
