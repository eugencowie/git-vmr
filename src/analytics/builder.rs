use aptabase_rs::Builder;
use std::sync::Arc;

pub(super) trait AnalyticsClientBuilder
{
    type Client;

    fn with_session_id(self, session_id: String) -> Self;
    fn build(self) -> Self::Client;
}

impl AnalyticsClientBuilder for Builder
{
    type Client = Arc<aptabase_rs::AptabaseClient>;

    fn with_session_id(self, session_id: String) -> Self
    {
        Builder::with_session_id(self, session_id)
    }

    fn build(self) -> Self::Client
    {
        Builder::build(self)
    }
}

pub(super) fn build_analytics_client<B: AnalyticsClientBuilder>(
    builder: B,
    session_id: String
) -> B::Client
{
    builder.with_session_id(session_id).build()
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn client_builder_receives_session_id()
    {
        struct FakeBuilder
        {
            session_id: Option<String>
        }

        impl AnalyticsClientBuilder for FakeBuilder
        {
            type Client = String;

            fn with_session_id(mut self, session_id: String) -> Self
            {
                self.session_id = Some(session_id);
                self
            }

            fn build(self) -> Self::Client
            {
                self.session_id.unwrap()
            }
        }

        let session_id = build_analytics_client(
            FakeBuilder { session_id: None },
            "persisted-session".to_owned()
        );

        assert_eq!(session_id, "persisted-session");
    }
}
