# Resource lifecycle runtime

This locked package exercises generated opaque-handle signatures against four
local resources: an HTTP loopback served to `reqwest`, a temporary `rusqlite`
database, and deterministic RESP/PostgreSQL protocol loopbacks used by
`redis` and `tokio-postgres`.

The protocol servers implement only the frames used by this scenario. Every
accept, read, write, client operation, and task join is bounded. Cleanup
handles are owned before a task starts, and drop guards abort unfinished tasks
and remove the temporary database on every exit path. The positive path uses a
borrowed generated operation bridge followed by an owned generated
`close=async_close` member routed to the package bridge. The distinct negative path closes the real
four-resource identity and then retries an operation through its bridge-local
shared alias. The Redis client disables library-metadata `CLIENT SETINFO`; the
minimal RESP server covers only the exercised connection and `PING` frames.
