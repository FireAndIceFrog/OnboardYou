# Database Migrations

Drop your SQL migration files here. The Makefile provides a `migrate` target that
will execute all `*.sql` files in this directory against the **direct**
connection string (Terraform output `db_connection_string_direct`).

Example:

```sh
make migrate
```

You need the `psql` client installed locally (e.g. `sudo apt install postgresql-client`)
for the command to work.  Replace path or client if you prefer another tool.
