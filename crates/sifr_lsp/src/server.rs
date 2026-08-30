use crate::capabilities;
use crate::errors::{LspError, LspResult, ServerResult};
use crate::notifications;
use crate::request_queue;
use crate::request_queue::CancellationTarget;
use crate::requests;
use crate::scheduler::Scheduler;
use crate::session::Session;
use crate::watchdog::{LspServerOptions, ParentWatchdog};
use lsp_server::{Connection, IoThreads, Message, Request, RequestId, Response, ResponseError};
use serde_json::Value;
use sifr_analysis::WorkspaceTracePhase;
use std::collections::BTreeMap;

pub fn run_stdio() -> ServerResult<()> {
    run_stdio_with_options(LspServerOptions::stdio())
}

pub fn run_stdio_with_options(options: LspServerOptions) -> ServerResult<()> {
    LspServer::stdio(options).run()
}

struct LspServer {
    connection: Connection,
    io_threads: IoThreads,
    session: Session,
    watchdog: ParentWatchdog,
    queued_requests: BTreeMap<String, Request>,
}

impl LspServer {
    fn stdio(options: LspServerOptions) -> Self {
        let (connection, io_threads) = Connection::stdio();
        let watchdog = ParentWatchdog::new(options.parent_pid);
        watchdog.spawn_exit_thread();
        Self {
            connection,
            io_threads,
            session: Session::new(),
            watchdog,
            queued_requests: BTreeMap::new(),
        }
    }

    fn run(mut self) -> ServerResult<()> {
        let (initialize_id, initialize_params) = self.connection.initialize_start()?;
        let settings = crate::settings::settings_from_initialize_params(
            &initialize_params,
            self.session.store().settings(),
        )?;
        self.session.set_work_done_progress_enabled(
            crate::settings::work_done_progress_from_initialize_params(&initialize_params),
        );
        let position_encoding = capabilities::negotiated_position_encoding(&initialize_params);
        self.session.set_position_encoding(position_encoding);
        self.session.store_mut().apply_settings(settings.clone());
        let initialize_data = serde_json::json!({
            "capabilities": capabilities::server_capabilities(
                settings.format_enable,
                position_encoding
            ),
            "serverInfo": {
                "name": "sifr-lsp",
                "version": env!("CARGO_PKG_VERSION")
            }
        });
        self.connection
            .initialize_finish(initialize_id, initialize_data)?;
        let source = self.connection.receiver.clone();
        let cancellation = self.session.cancellation_registry();
        let (forward, incoming) = std::sync::mpsc::channel();
        let message_pump = std::thread::spawn(move || {
            while let Ok(message) = source.recv() {
                if let Message::Notification(notification) = &message
                    && notification.method == "$/cancelRequest"
                    && let Some(id) = notifications::cancel_request_id(&notification.params)
                {
                    cancellation.cancel(&id);
                }
                if forward.send(message).is_err() {
                    break;
                }
            }
        });
        while let Ok(message) = incoming.recv() {
            self.watchdog.check()?;
            match message {
                Message::Request(request) => {
                    if request.method == "shutdown" {
                        let response = response_from_result(request.id, Ok(Value::Null));
                        self.connection.sender.send(Message::Response(response))?;
                        self.session.begin_shutdown();
                        continue;
                    }
                    self.handle_request(request)?;
                }
                Message::Notification(notification) => {
                    if notification.method == "$/cancelRequest" {
                        if let Some(id) = notifications::cancel_request_id(&notification.params) {
                            self.cancel_request(&id)?;
                        }
                        continue;
                    }
                    let is_exit = notification.method == "exit";
                    if let Err(error) = notifications::handle(
                        &mut self.session,
                        &self.connection,
                        &notification.method,
                        notification.params,
                    ) {
                        self.session.trace(
                            WorkspaceTracePhase::LspTiming,
                            format!(
                                "notification {} failed: {}",
                                notification.method,
                                error.message()
                            ),
                        );
                    }
                    if is_exit {
                        #[allow(clippy::bool_to_int_with_if)]
                        let code = if self.session.shutdown_requested() {
                            0
                        } else {
                            1
                        };
                        std::process::exit(code);
                    }
                }
                Message::Response(_) => {}
            }
        }
        if message_pump.join().is_err() {
            return Err(Box::new(LspError::internal("LSP message pump panicked")));
        }
        self.finish()
    }

    fn handle_request(&mut self, request: Request) -> ServerResult<()> {
        let id = request.id.clone();
        let lane = Scheduler::lane_for_method(&request.method);
        if let Err(error) = self.session.enqueue_request(&id, &request.method, lane) {
            let error = LspError::request_cancelled(error);
            let response = response_from_result(id, Err(error));
            self.connection
                .sender
                .send(Message::Response(response))
                .map_err(|error| {
                    LspError::internal(format!("failed to send LSP response: {error}"))
                })?;
            return Ok(());
        }
        self.queued_requests
            .insert(request_queue::request_key(&id), request);
        self.drain_queued_requests()?;
        Ok(())
    }

    fn cancel_request(&mut self, id: &lsp_server::RequestId) -> ServerResult<()> {
        match self.session.cancel_request(id) {
            CancellationTarget::None => {}
            CancellationTarget::Queued => {
                self.queued_requests.remove(&request_queue::request_key(id));
                self.send_cancelled_response(
                    id.clone(),
                    format!("request {id:?} was cancelled before dispatch"),
                )?;
            }
            CancellationTarget::InFlight => {}
        }
        Ok(())
    }

    fn drain_queued_requests(&mut self) -> ServerResult<()> {
        while let Some(scheduled) = self.session.start_next_request() {
            let Some(request) = self.queued_requests.remove(scheduled.key()) else {
                let id = scheduled.id().clone();
                self.session.finish_request(scheduled.id());
                let error = LspError::internal(format!(
                    "scheduled request body was missing for key {}",
                    scheduled.key()
                ));
                let response = response_from_result(id, Err(error));
                self.connection
                    .sender
                    .send(Message::Response(response))
                    .map_err(|error| {
                        LspError::internal(format!("failed to send LSP response: {error}"))
                    })?;
                continue;
            };
            let id = request.id.clone();
            let result = self
                .session
                .begin_request_execution(&id)
                .and_then(|()| requests::handle(&mut self.session, &request.method, request.params))
                .and_then(|result| {
                    self.session.check_request_cancelled(&id)?;
                    Ok(result)
                });
            self.session.finish_request(&id);
            let response = response_from_result(id, result);
            self.connection
                .sender
                .send(Message::Response(response))
                .map_err(|error| {
                    LspError::internal(format!("failed to send LSP response: {error}"))
                })?;
        }
        Ok(())
    }

    fn send_cancelled_response(
        &self,
        id: lsp_server::RequestId,
        message: String,
    ) -> ServerResult<()> {
        let error = LspError::request_cancelled(message);
        let response = response_from_result(id, Err(error));
        self.connection
            .sender
            .send(Message::Response(response))
            .map_err(|error| LspError::internal(format!("failed to send LSP response: {error}")))?;
        Ok(())
    }

    fn finish(self) -> ServerResult<()> {
        let Self {
            connection,
            io_threads,
            session: _,
            watchdog: _,
            queued_requests: _,
        } = self;
        drop(connection);
        io_threads.join()?;
        Ok(())
    }
}

fn response_from_result(id: RequestId, result: LspResult<Value>) -> Response {
    let response_result = result.map_err(|error| ResponseError {
        code: error.code(),
        message: error.message(),
        data: None,
    });
    Response {
        id,
        response_result,
    }
}

#[cfg(test)]
mod tests {
    use super::response_from_result;
    use crate::errors::LspError;
    use lsp_server::RequestId;
    use serde_json::json;

    #[test]
    fn responses_use_the_typed_result_model() {
        let success = response_from_result(RequestId::from(1), Ok(json!({"ready": true})));
        assert_eq!(
            success.response_result.expect("success response"),
            json!({"ready": true})
        );

        let failure = response_from_result(
            RequestId::from(2),
            Err(LspError::invalid_params("invalid request")),
        );
        let error = failure.response_result.expect_err("error response");
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "invalid request");
        assert_eq!(error.data, None);
    }

    #[test]
    fn typed_responses_serialize_one_protocol_outcome() {
        let success = response_from_result(RequestId::from(1), Ok(json!(null)));
        assert_eq!(
            serde_json::to_value(success).expect("success response must serialize"),
            json!({"id": 1, "result": null})
        );

        let failure = response_from_result(
            RequestId::from(2),
            Err(LspError::internal("request failed")),
        );
        assert_eq!(
            serde_json::to_value(failure).expect("error response must serialize"),
            json!({
                "id": 2,
                "error": {"code": -32603, "message": "request failed"}
            })
        );
    }
}
