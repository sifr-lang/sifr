from alembic.autogenerate import compare_metadata
from alembic.runtime.migration import MigrationContext
import psycopg
from psycopg.conninfo import conninfo_to_dict, make_conninfo
import sqlalchemy


def run() -> str:
    engine = sqlalchemy.create_engine("sqlite+pysqlite:///:memory:")
    with engine.connect() as connection:
        scalar = connection.execute(sqlalchemy.text("select 40 + 2")).scalar_one()
        base = sqlalchemy.MetaData()
        sqlalchemy.Table(
            "account",
            base,
            sqlalchemy.Column("id", sqlalchemy.Integer, primary_key=True),
            sqlalchemy.Column("balance", sqlalchemy.Integer),
        )
        base.create_all(connection)
        target = sqlalchemy.MetaData()
        sqlalchemy.Table(
            "account",
            target,
            sqlalchemy.Column("id", sqlalchemy.Integer, primary_key=True),
            sqlalchemy.Column("balance", sqlalchemy.Integer),
            sqlalchemy.CheckConstraint("balance >= 0", name="ck_account_balance"),
        )
        context = MigrationContext.configure(connection, opts={
            "autogenerate_plugins": [
                "alembic.autogenerate.*", "alembic.ext.checkconstraint_byname",
            ],
        })
        dialect_name = context.dialect.name
        differences = compare_metadata(context, target)
    engine.dispose()
    conninfo = make_conninfo(host="localhost", dbname="sifr verification")
    parsed_conninfo = conninfo_to_dict(conninfo)
    named_check_added = any(
        difference[0] == "add_constraint" and difference[1].name == "ck_account_balance"
        for difference in differences
    )
    if (
        scalar != 42
        or dialect_name != "sqlite"
        or not named_check_added
        or not psycopg.__version__
        or parsed_conninfo != {"host": "localhost", "dbname": "sifr verification"}
    ):
        raise RuntimeError("SQLAlchemy/psycopg full example failed")
    return (
        "sifr-python-interop:sqlalchemy-psycopg:scalar=42:dialect=sqlite:"
        "named-check=add:conninfo=ok"
    )
