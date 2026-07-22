from alembic.runtime.migration import MigrationContext
import psycopg
from psycopg.conninfo import make_conninfo
import sqlalchemy


def run() -> str:
    engine = sqlalchemy.create_engine("sqlite+pysqlite:///:memory:")
    with engine.connect() as connection:
        scalar = connection.execute(sqlalchemy.text("select 40 + 2")).scalar_one()
        dialect_name = MigrationContext.configure(connection).dialect.name
    engine.dispose()
    conninfo = make_conninfo(host="localhost", dbname="postgres")
    if scalar != 42 or dialect_name != "sqlite" or not psycopg.__version__ or not conninfo:
        raise RuntimeError("SQLAlchemy/psycopg full example failed")
    return "sifr-python-interop:sqlalchemy-psycopg:scalar=42:dialect=sqlite:conninfo=ok"
