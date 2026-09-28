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

mod watcher_registration;
use watcher_registration::WatcherRegistration;

/// Run the embedded server with the caller's compiler and toolchain ownership.
pub fn run_stdio(compiler: sifr_compiler_services::CompilerContext) -> ServerResult<()> {
    run_stdio_with_options(LspServerOptions::stdio(), compiler)
}

pub fn run_stdio_with_options(
    options: LspServerOptions,
    compiler: sifr_compiler_services::CompilerContext,
) -> ServerResult<()> {
    LspServer::stdio(options, compiler).run()
}

pub fn run_stdio_with_identity(
    options: LspServerOptions,
    identity: sifr_identity::CompilerIdentity,
) -> ServerResult<()> {
    run_stdio_with_options(
        options,
        sifr_compiler_services::CompilerContext::new(identity),
    )
}

struct LspServer {
    connection: Connection,
    io_threads: IoThreads,
    session: Session,
    watchdog: ParentWatchdog,
    queued_requests: BTreeMap<String, Request>,
    watchers: WatcherRegistration,
}

impl LspServer {
    fn stdio(options: LspServerOptions, compiler: sifr_compiler_services::CompilerContext) -> Self {
        let (connection, io_threads) = Connection::stdio();
        let watchdog = ParentWatchdog::new(options.parent_pid);
        watchdog.spawn_exit_thread();
        Self {
            connection,
            io_threads,
            session: Session::with_compiler(compiler),
            watchdog,
            queued_requests: BTreeMap::new(),
            watchers: WatcherRegistration::new(&Value::Null),
        }
    }

    fn run(mut self) -> ServerResult<()> {
        let (initialize_id, initialize_params) = self.connection.initialize_start()?;
        self.watchers = WatcherRegistration::new(&initialize_params);
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
        let generations = self.session.generations.clone();
        let (forward, incoming) = std::sync::mpsc::channel();
        let message_pump = std::thread::spawn(move || {
            while let Ok(message) = source.recv() {
                if let Message::Notification(notification) = &message
                    && notification.method == "$/cancelRequest"
                    && let Some(id) = notifications::cancel_request_id(&notification.params)
                {
                    cancellation.cancel(&id);
                }
                let method = match &message {
                    Message::Notification(n) => Some(n.method.as_str()),
                    _ => None,
                };
                let Ok(generation) = generations.observe(method) else {
                    break;
                };
                if forward.send((message, generation)).is_err() {
                    break;
                }
            }
        });
        while let Ok((message, generation)) = incoming.recv() {
            self.session.generation = generation;
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
                    let is_initialized = notification.method == "initialized";
                    let folders_changed =
                        notification.method == "workspace/didChangeWorkspaceFolders";
                    if folders_changed {
                        self.watchers.change_workspace_folders(&notification.params);
                    }
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
                        if notification.method == "workspace/didChangeWatchedFiles" {
                            self.watchers.invalidate();
                            self.session.revalidate_open_external_inputs();
                        }
                        // A rejected mutation still advanced ingress. Reconcile the
                        // unchanged authoritative state after suppressing older work.
                        let _ = crate::diagnostics::DiagnosticsController::publish_all(
                            &self.connection,
                            &mut self.session,
                        );
                    }
                    if folders_changed || is_initialized && !self.watchers.started() {
                        self.watchers.register(&self.connection)?;
                        self.session.revalidate_open_external_inputs();
                    }
                    crate::diagnostics::DiagnosticsController::flush_clears(
                        &self.connection,
                        &mut self.session,
                    )?;
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
                Message::Response(response) => {
                    if self.watchers.respond(&response)
                        && self.session.revalidate_open_external_inputs()
                    {
                        crate::diagnostics::DiagnosticsController::reconcile_changes(
                            &self.connection,
                            &mut self.session,
                        )?;
                    }
                }
            }
        }
        if message_pump.join().is_err() {
            return Err(Box::new(LspError::internal("LSP message pump panicked")));
        }
        self.finish()
    }

    fn handle_request(&mut self, request: Request) -> ServerResult<()> {
        revalidate_and_publish_without_watchers(
            &mut self.session,
            &self.watchers,
            &self.connection,
        )?;
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
            self.session
                .generations
                .publish(self.session.generation, |current| {
                    let result = if current {
                        result
                    } else {
                        Err(LspError::content_modified(
                            "request snapshot was superseded by a source or configuration event",
                        ))
                    };
                    self.connection
                        .sender
                        .send(Message::Response(response_from_result(id, result)))
                        .map_err(|error| {
                            LspError::internal(format!("failed to send LSP response: {error}"))
                        })
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
            watchers: _,
        } = self;
        drop(connection);
        io_threads.join()?;
        Ok(())
    }
}

fn revalidate_without_watchers(session: &mut Session, watchers: &WatcherRegistration) -> bool {
    session.revalidate_open_external_inputs_where(|path| !watchers.covers(path))
}

fn revalidate_and_publish_without_watchers(
    session: &mut Session,
    watchers: &WatcherRegistration,
    connection: &Connection,
) -> LspResult<()> {
    if revalidate_without_watchers(session, watchers) {
        crate::diagnostics::DiagnosticsController::reconcile_changes(connection, session)?;
    }
    Ok(())
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

    fn watcher_fixture(root: &std::path::Path, name: &str) -> String {
        std::fs::create_dir_all(root.join("src")).expect("source directory");
        std::fs::write(root.join("src/lib.rs"), "").expect("Cargo library marker");
        std::fs::write(root.join("Cargo.toml"), format!(
            "[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[package.metadata.sifr]\nmanifest = \"sifr.toml\"\n\n[workspace]\n"
        )).expect("Cargo manifest");
        std::fs::write(
            root.join("Cargo.lock"),
            format!("version = 4\n\n[[package]]\nname = \"{name}\"\nversion = \"0.0.0\"\n"),
        )
        .expect("Cargo lock");
        std::fs::write(root.join("sifr.toml"), "[package]\nname = \"watcher\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n").expect("Sifr manifest");
        let path = root.join("src/main.sifr");
        std::fs::write(&path, "def main() -> int:\n    return 1\n").expect("Sifr source");
        url::Url::from_file_path(path)
            .expect("file URI")
            .to_string()
    }

    fn open_watcher_document(session: &mut crate::session::Session, uri: String) {
        session
            .open_document(
                uri,
                crate::capabilities::LANGUAGE_ID,
                Some(1),
                "def main() -> int:\n    return 1\n".to_string(),
            )
            .expect("open document");
    }

    #[test]
    fn watcher_registration_acknowledgement_enables_root_scoped_events() {
        use super::watcher_registration::WatcherRegistration;
        use lsp_server::{Connection, Message, Response};
        use serde_json::Value;
        let temp = tempfile::tempdir().expect("temporary roots");
        let first = temp.path().join("first");
        let second = temp.path().join("second");
        let first_uri = watcher_fixture(&first, "watcher-first");
        let second_uri = watcher_fixture(&second, "watcher-second");
        let mut session = crate::session::Session::new();
        open_watcher_document(&mut session, first_uri);
        open_watcher_document(&mut session, second_uri);
        let first_generation = session
            .external_input_generation(&first)
            .expect("first generation");
        let second_generation = session
            .external_input_generation(&second)
            .expect("second generation");
        let (server, client) = Connection::memory();
        let mut watchers = WatcherRegistration::new(&json!({
            "workspaceFolders": [{"uri": url::Url::from_file_path(&first).expect("first folder URI").to_string(), "name": "first"},
                                 {"uri": url::Url::from_file_path(&second).expect("second folder URI").to_string(), "name": "second"}],
            "capabilities": {"workspace": {"didChangeWatchedFiles": {"dynamicRegistration": true}}}
        }));
        assert!(!watchers.acknowledged());
        watchers.register(&server).expect("send registration");
        let Message::Request(request) = client.receiver.recv().expect("registration request")
        else {
            panic!("expected registration request");
        };
        assert_eq!(request.method, "client/registerCapability");
        assert_eq!(
            request
                .params
                .pointer("/registrations/0/registerOptions/watchers/0/kind"),
            Some(&json!(7))
        );
        assert!(!watchers.acknowledged());
        watchers.respond(&Response {
            id: request.id,
            response_result: Ok(Value::Null),
        });
        assert!(watchers.acknowledged());
        std::fs::write(first.join("Cargo.lock"), "version = 4\n# changed\n")
            .expect("change first lock");
        crate::notifications::handle(
            &mut session,
            &server,
            "workspace/didChangeWatchedFiles",
            json!({"changes": [{
                "uri": url::Url::from_file_path(first.join("Cargo.lock")).expect("lock URI").to_string(),
                "type": 2
            }]}),
        ).expect("watched-file notification");
        assert!(
            session
                .external_input_generation(&first)
                .expect("new first generation")
                > first_generation
        );
        assert_eq!(
            session.external_input_generation(&second),
            Some(second_generation)
        );
    }

    #[test]
    fn request_time_external_change_publishes_current_diagnostics() {
        use super::{
            revalidate_and_publish_without_watchers, watcher_registration::WatcherRegistration,
        };
        use lsp_server::{Connection, Message};
        let temp = tempfile::tempdir().expect("temporary root");
        let uri = watcher_fixture(temp.path(), "watcher-publication");
        let mut session = crate::session::Session::new();
        open_watcher_document(&mut session, uri.clone());
        let unsupported = WatcherRegistration::new(&json!({"capabilities": {}}));
        let (server, client) = Connection::memory();

        revalidate_and_publish_without_watchers(&mut session, &unsupported, &server)
            .expect("unchanged request");
        assert!(client.receiver.try_recv().is_err());

        let manifest = temp.path().join("sifr.toml");
        std::fs::write(
            &manifest,
            "invalid = [
",
        )
        .expect("invalidate manifest");
        revalidate_and_publish_without_watchers(&mut session, &unsupported, &server)
            .expect("changed request");
        let publication = client
            .receiver
            .try_iter()
            .find_map(|message| match message {
                Message::Notification(notification)
                    if notification.method == "textDocument/publishDiagnostics" =>
                {
                    Some(notification.params)
                }
                _ => None,
            })
            .expect("current diagnostics publication");
        assert_eq!(publication["uri"], uri);
        assert_eq!(publication["version"], 1);
        assert!(
            !publication["diagnostics"]
                .as_array()
                .expect("diagnostics")
                .is_empty()
        );

        revalidate_and_publish_without_watchers(&mut session, &unsupported, &server)
            .expect("warm request");
        assert!(client.receiver.try_recv().is_err());
    }

    #[test]
    fn unconfirmed_rejected_and_unsupported_watchers_revalidate_per_request() {
        use super::{revalidate_without_watchers, watcher_registration::WatcherRegistration};
        use lsp_server::{Connection, Message, Response, ResponseError};
        let temp = tempfile::tempdir().expect("temporary root");
        let uri = watcher_fixture(temp.path(), "watcher-fallback");
        let mut session = crate::session::Session::new();
        open_watcher_document(&mut session, uri);
        let (server, client) = Connection::memory();
        let mut watchers = WatcherRegistration::new(&json!({"capabilities": {"workspace": {
            "didChangeWatchedFiles": {"dynamicRegistration": true}
        }}}));
        watchers.register(&server).expect("register watcher");
        let Message::Request(request) = client.receiver.recv().expect("registration") else {
            panic!("expected registration");
        };
        let initial = session
            .external_input_generation(temp.path())
            .expect("initial generation");
        std::fs::write(temp.path().join("Cargo.lock"), "version = 4\n# pending\n")
            .expect("change pending lock");
        revalidate_without_watchers(&mut session, &watchers);
        let pending = session
            .external_input_generation(temp.path())
            .expect("pending generation");
        assert!(pending > initial);
        watchers.respond(&Response {
            id: request.id,
            response_result: Err(ResponseError {
                code: -32603,
                message: "registration rejected".to_string(),
                data: None,
            }),
        });
        assert!(!watchers.acknowledged());
        std::fs::write(temp.path().join("Cargo.lock"), "version = 4\n# rejected\n")
            .expect("change rejected lock");
        revalidate_without_watchers(&mut session, &watchers);
        let rejected = session
            .external_input_generation(temp.path())
            .expect("rejected generation");
        assert!(rejected > pending);
        let unsupported = WatcherRegistration::new(&json!({"capabilities": {}}));
        let (unsupported_server, unsupported_client) = Connection::memory();
        let mut unsupported = unsupported;
        unsupported
            .register(&unsupported_server)
            .expect("unsupported registration is skipped");
        assert!(unsupported_client.receiver.try_recv().is_err());
        std::fs::write(
            temp.path().join("Cargo.lock"),
            "version = 4\n# unsupported\n",
        )
        .expect("change unsupported lock");
        revalidate_without_watchers(&mut session, &unsupported);
        assert!(
            session
                .external_input_generation(temp.path())
                .expect("unsupported generation")
                > rejected
        );
        let mut acknowledged = WatcherRegistration::new(&json!({
            "rootUri": url::Url::from_file_path(temp.path()).expect("root URI").to_string(),
            "capabilities": {"workspace": {"didChangeWatchedFiles": {"dynamicRegistration": true}}}
        }));
        let (ack_server, ack_client) = Connection::memory();
        acknowledged
            .register(&ack_server)
            .expect("supported registration");
        let Message::Request(ack_request) =
            ack_client.receiver.recv().expect("registration request")
        else {
            panic!("expected registration request");
        };
        acknowledged.respond(&Response {
            id: ack_request.id,
            response_result: Ok(serde_json::Value::Null),
        });
        assert!(acknowledged.covers(&temp.path().join("src/main.sifr")));
        let before_malformed = session
            .external_input_generation(temp.path())
            .expect("generation before malformed event");
        std::fs::write(
            temp.path().join("Cargo.lock"),
            "version = 4\n# malformed event\n",
        )
        .expect("change lock before malformed event");
        revalidate_without_watchers(&mut session, &acknowledged);
        assert_eq!(
            session.external_input_generation(temp.path()),
            Some(before_malformed)
        );
        assert!(
            crate::notifications::handle(
                &mut session,
                &ack_server,
                "workspace/didChangeWatchedFiles",
                json!({"changes": [{"uri": "https://example.invalid/Cargo.lock", "type": 2}]})
            )
            .is_err()
        );
        acknowledged.invalidate();
        revalidate_without_watchers(&mut session, &acknowledged);
        assert!(
            session
                .external_input_generation(temp.path())
                .expect("generation after malformed event")
                > before_malformed
        );
    }

    #[test]
    fn watcher_reconnect_and_workspace_folder_change_restore_authority() {
        use super::watcher_registration::WatcherRegistration;
        use lsp_server::{Connection, Message, Response};
        use serde_json::Value;
        let temp = tempfile::tempdir().expect("workspace roots");
        let first_root = temp.path().join("first");
        let second_root = temp.path().join("second");
        let first_uri = url::Url::from_file_path(&first_root)
            .expect("first URI")
            .to_string();
        let second_uri = url::Url::from_file_path(&second_root)
            .expect("second URI")
            .to_string();
        let capabilities = json!({"workspaceFolders": [{"uri": first_uri, "name": "first"}],
            "capabilities": {"workspace": {"didChangeWatchedFiles": {"dynamicRegistration": true}}}});
        let (server, client) = Connection::memory();
        let mut watchers = WatcherRegistration::new(&capabilities);
        watchers.register(&server).expect("initial registration");
        let Message::Request(first) = client.receiver.recv().expect("initial request") else {
            panic!("expected registration");
        };
        watchers.respond(&Response {
            id: first.id,
            response_result: Ok(Value::Null),
        });
        assert!(watchers.acknowledged());
        assert!(watchers.covers(&first_root.join("src/main.sifr")));
        watchers.change_workspace_folders(&json!({"event": {
            "removed": [{"uri": first_uri, "name": "first"}],
            "added": [{"uri": second_uri, "name": "second"}]
        }}));
        watchers
            .register(&server)
            .expect("folder change registration");
        assert!(!watchers.acknowledged());
        let Message::Request(unregister) = client.receiver.recv().expect("unregister request")
        else {
            panic!("expected unregistration");
        };
        assert_eq!(unregister.method, "client/unregisterCapability");
        assert_eq!(
            unregister.params.pointer("/unregisterations/0/id"),
            Some(&json!("sifr/watched-files/1"))
        );
        let Message::Request(second) = client.receiver.recv().expect("replacement request") else {
            panic!("expected replacement registration");
        };
        watchers.respond(&Response {
            id: unregister.id,
            response_result: Ok(Value::Null),
        });
        assert!(!watchers.acknowledged());
        watchers.respond(&Response {
            id: second.id,
            response_result: Ok(Value::Null),
        });
        assert!(watchers.acknowledged());
        assert!(!watchers.covers(&first_root.join("src/main.sifr")));
        assert!(watchers.covers(&second_root.join("src/main.sifr")));
        let reconnect_params = json!({"workspaceFolders": [{"uri": second_uri, "name": "second"}],
            "capabilities": {"workspace": {"didChangeWatchedFiles": {"dynamicRegistration": true}}}});
        let mut reconnected = WatcherRegistration::new(&reconnect_params);
        assert!(!reconnected.acknowledged());
        let (new_server, new_client) = Connection::memory();
        reconnected
            .register(&new_server)
            .expect("reconnect registration");
        let Message::Request(reconnect_request) =
            new_client.receiver.recv().expect("reconnect request")
        else {
            panic!("expected reconnect registration");
        };
        reconnected.respond(&Response {
            id: reconnect_request.id,
            response_result: Ok(Value::Null),
        });
        assert!(reconnected.acknowledged());
        let (pending_server, pending_client) = Connection::memory();
        let mut pending = WatcherRegistration::new(&capabilities);
        pending
            .register(&pending_server)
            .expect("pending registration");
        let Message::Request(old_pending) =
            pending_client.receiver.recv().expect("old pending request")
        else {
            panic!("expected pending registration");
        };
        pending.change_workspace_folders(&json!({"event": {
            "removed": [{"uri": first_uri, "name": "first"}],
            "added": [{"uri": second_uri, "name": "second"}]
        }}));
        pending
            .register(&pending_server)
            .expect("replace pending registration");
        let Message::Request(pending_unregistration) = pending_client
            .receiver
            .recv()
            .expect("pending unregistration")
        else {
            panic!("expected pending unregistration");
        };
        assert_eq!(
            pending_unregistration
                .params
                .pointer("/unregisterations/0/id"),
            Some(&json!("sifr/watched-files/1"))
        );
        let Message::Request(replacement) =
            pending_client.receiver.recv().expect("pending replacement")
        else {
            panic!("expected replacement registration");
        };
        pending.respond(&Response {
            id: old_pending.id,
            response_result: Ok(Value::Null),
        });
        assert!(!pending.acknowledged());
        pending.respond(&Response {
            id: replacement.id,
            response_result: Ok(Value::Null),
        });
        assert!(pending.covers(&second_root.join("src/main.sifr")));
    }
}
