use sifr_sql_mysql_runtime::{MysqlPool, MysqlRowStream, Unverified, Verified};
use static_assertions::{assert_impl_all, assert_not_impl_any};

assert_impl_all!(MysqlPool<Unverified>: Send, Sync);
assert_impl_all!(MysqlPool<Verified>: Send, Sync);
assert_not_impl_any!(MysqlRowStream: Send, Sync);

#[test]
fn runtime_public_handles_preserve_ownership_boundaries() {
    fn consume_unverified(_: Option<MysqlPool<Unverified>>) {}
    fn consume_verified(_: Option<MysqlPool<Verified>>) {}
    consume_unverified(None);
    consume_verified(None);
}
