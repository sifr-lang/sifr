use crate::capabilities;
use crate::errors::{LspError, ServerResult};
use crate::notifications;
use crate::request_queue;
use crate::request_queue::CancellationTarget;
use crate::requests;
use crate::scheduler::Scheduler;
use crate::session::Session;
use crate::watchdog::{LspServerOptions, ParentWatchdog};
use lsp_server::{Connection, IoThreads, Message, Request, Response};
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
        Self {
            connection,
            io_threads,
            session: Session::new(),
            watchdog: ParentWatchdog::new(options.parent_pid),
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
        self.session.store_mut().apply_settings(settings.clone());
        let initialize_data = serde_json::json!({
            "capabilities": capabilities::server_capabilities(settings.format_enable),
            "serverInfo": {
                "name": "sifr-lsp",
                "version": env!("CARGO_PKG_VERSION")
            }
        });
        self.connection
            .initialize_finish(initialize_id, initialize_data)?;
        while let Ok(message) = self.connection.receiver.recv() {
            self.watchdog.check()?;
            match message {
                Message::Request(request) => {
                    if request.method == "shutdown" {
                        let response = Response::new_ok(request.id, ());
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
        self.finish()
    }

    fn handle_request(&mut self, request: Request) -> ServerResult<()> {
        let id = request.id.clone();
        let lane = Scheduler::lane_for_method(&request.method);
        if let Err(error) = self.session.enqueue_request(&id, &request.method, lane) {
            let error = LspError::request_cancelled(error);
            let response = Response::new_err(id, error.code(), error.message());
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
                let response = Response::new_err(id, error.code(), error.message());
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
            let response = match result {
                Ok(result) => Response::new_ok(id, result),
                Err(error) => Response::new_err(id, error.code(), error.message()),
            };
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
        let response = Response::new_err(id, error.code(), error.message());
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
