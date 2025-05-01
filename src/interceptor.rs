use headless_chrome::browser::transport::{SessionId, Transport};
use headless_chrome::protocol::cdp::Fetch::events::RequestPausedEvent;
use headless_chrome::browser::tab::{RequestPausedDecision, RequestInterceptor};
use headless_chrome::protocol::cdp::Fetch::FailRequest;
use std::sync::Arc;

pub struct ResourceInterceptor;

impl RequestInterceptor for ResourceInterceptor {
    fn intercept(
        &self,
        _transport: Arc<Transport>,
        _session_id: SessionId,
        event: RequestPausedEvent,
    ) -> RequestPausedDecision {
        let url = &event.params.request.url;
        if !url.ends_with(".css") && !url.ends_with(".js") {
                // println!("Aborting request: {}", url);
            return RequestPausedDecision::Fail(
                FailRequest {
                    request_id: event.params.request_id.clone(),
                    error_reason: headless_chrome::protocol::cdp::Network::ErrorReason::Aborted,
                }
            );
        }
        RequestPausedDecision::Continue(None)
    }
}