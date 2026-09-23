## Fix

Import the configured profile from `sifr.sql.schemas` before using its query decorator or SQL constructor:

```sifr
from sifr.sql.schemas import app

@app.query
def find_user(user_id: int) -> Template:
    return app.sql(t"SELECT name FROM users WHERE id = {user_id}")
```

An alias is valid when the decorator and SQL call use the imported name:

```sifr
from sifr.sql.schemas import app as database

@database.query
def find_user(user_id: int) -> Template:
    return database.sql(t"SELECT name FROM users WHERE id = {user_id}")
```

A decorator on an unrelated object, such as `@cache.query`, is outside SQL profile discovery.
