import psycopg
from psycopg import sql


def run(endpoint: str, token: str) -> str:
    table = sql.Identifier(f"sifr_live_{token}")
    with psycopg.connect(endpoint) as connection:
        with connection.cursor() as cursor:
            cursor.execute(
                sql.SQL("create table {} (id integer primary key, label text not null)").format(table)
            )
            cursor.execute(
                sql.SQL("insert into {} (id, label) values (%s, %s)").format(table),
                (1, token),
            )
            cursor.execute(sql.SQL("select label from {} where id = %s").format(table), (1,))
            row = cursor.fetchone()
        connection.commit()
    if row != (token,):
        raise RuntimeError(f"Postgres row mismatch: {row!r}")
    return "sifr-python-interop:live:postgres:roundtrip=ok:resources=zero"
