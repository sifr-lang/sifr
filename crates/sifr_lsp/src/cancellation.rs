use lsp_server::RequestId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CancellationToken {
    request_id: RequestId,
}

impl CancellationToken {
    pub(crate) fn new(request_id: &RequestId) -> Self {
        Self {
            request_id: request_id.clone(),
        }
    }

    pub(crate) fn request_id(&self) -> &RequestId {
        &self.request_id
    }
}

#[cfg(test)]
mod tests {
    use super::CancellationToken;
    use lsp_server::RequestId;

    #[test]
    fn token_preserves_request_identity() {
        let id = RequestId::from("cancellation-token".to_string());
        let token = CancellationToken::new(&id);

        assert_eq!(token.request_id(), &id);
    }
}
