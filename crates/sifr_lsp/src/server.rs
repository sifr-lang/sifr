use crate::capabilities;
use crate::errors::{LspError, ServerResult};
use crate::notifications;
use crate::requests;
use crate::session::Session;
use lsp_server::{Connection, IoThreads, Message, Request, Response};

pub fn run_stdio() -> ServerResult<()> {
    LspServer::stdio().run()
}

struct LspServer {
    connection: Connection,
    io_threads: IoThreads,
    session: Session,
}

impl LspServer {
    fn stdio() -> Self {
        let (connection, io_threads) = Connection::stdio();
        Self {
            connection,
            io_threads,
            session: Session::new(),
        }
    }

    fn run(mut self) -> ServerResult<()> {
        self.connection
            .initialize(capabilities::server_capabilities())?;
        while let Ok(message) = self.connection.receiver.recv() {
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
                    let is_exit = notification.method == "exit";
                    if let Err(error) = notifications::handle(
                        &mut self.session,
                        &self.connection,
                        &notification.method,
                        notification.params,
                    ) {
                        self.session.trace(format!(
                            "notification {} failed: {}",
                            notification.method,
                            error.message()
                        ));
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
        let result = self
            .session
            .start_request(&id)
            .map_err(LspError::request_cancelled)
            .and_then(|()| requests::handle(&mut self.session, &request.method, request.params));
        self.session.finish_request(&id);
        let response = match result {
            Ok(result) => Response::new_ok(id, result),
            Err(error) => Response::new_err(id, error.code(), error.message()),
        };
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
        } = self;
        drop(connection);
        io_threads.join()?;
        Ok(())
    }
}
