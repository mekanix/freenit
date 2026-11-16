# freenit

A [Topcoat](https://github.com/tokio-rs/topcoat) application.

## Development

Install the Topcoat CLI:

```sh
cargo install topcoat-cli
```

Create the database tables by generating a migration from the models and applying it:

```sh
cargo run -- toasty migration generate
cargo run -- toasty migration apply
```

Run both commands again after changing a model. The default database is `data.db`.

Make an existing user an administrator with:

```sh
cargo run --bin make-admin -- user@example.com
```

## Configuration

Configuration is loaded from `freenit.toml` by default. The checked-in file lists every setting and its default. Select another file with `CONFIG` (useful with `topcoat dev`) or `--config`:

```sh
CONFIG=/path/to/freenit.toml topcoat dev
CONFIG=/path/to/freenit.toml cargo run
cargo run -- --config /path/to/freenit.toml
```

When both are set, `CONFIG` takes precedence over `--config`. If the selected file does not exist, built-in defaults are used. Other application settings are not read from environment variables. The same options work for migrations and `make-admin`.

Set `[logging].level` to `error`, `warn`, `info`, `debug`, or `trace` to choose the maximum emitted log level. The default is `info`.

The `[auth]` section selects `backend = "sql"` or `backend = "ldap"`, and contains the shared secret used for tokens and OMEMO encryption plus `secure_cookies` for HTTPS deployments. `[database]` selects the SQL database URL. `[ldap]` contains the LDAP URL, optional `starttls` upgrade setting, service credentials, user/group trees, object classes, membership attributes, OMEMO storage, and UID/GID counter entries. Set `starttls = true` when the server requires StartTLS over `ldap://`; use an `ldaps://` URL for implicit TLS. The LDAP server must provide the Freenit schema and counter entries.

## Auth backends

The default feature set includes both SQL and LDAP auth implementations, while `[auth] backend` selects which one the app uses at runtime. SQL uses numeric IDs; LDAP uses DNs. For a lean LDAP-only build, run:

```sh
cargo run --no-default-features --features ldap
```

## Optional services

Enable `mail` for registration and recovery email over SMTP using `[mail]`. Enable `webmail` separately for the webmail and Sieve pages. The features are independent: `mail` does not add webmail pages, and `webmail` does not enable registration or recovery email.

Webmail and Sieve use Dziber's JMAP and ManageSieve clients. `[stalwart]` contains the server URL and per-user impersonation credentials; `[sieve]` contains the ManageSieve port and security mode (`none`, `starttls`, or `tls`).

Enable `calendar` for native calendars, events, tasks, and iCalendar import/export. `[calendar] protocol` selects `dav` or `jmap`. Enable `contacts` for address books and contacts; `[contacts] protocol` selects `dav` or `jmap`. Both protocols use the Stalwart settings.

Enable `kanban` for projects, boards, columns, tasks, assignees, parent/child tasks, project groups, and group permissions. Kanban stores its data in the SQL database, independently of whether SQL or LDAP is used for authentication. When using LDAP authentication with Kanban, configure both the LDAP service and the SQL database. Generate and apply the migration after enabling it.

Enable `jabber` for the Rust/WASM XMPP client and encrypted OMEMO state management. `[jabber] ws_url` sets the browser-facing WebSocket URL; `[auth] secret` should be a strong secret. Chat is available at `/chat`; administrative OMEMO management is at `/jabber`. There is no backend chat worker or JSON API.

The JMAP/ManageSieve clients, OMEMO module, and browser chat client live in the sibling Dziber checkout, imported as `dziber = { path = "dziber" }`. Install the WASM target and matching `wasm-bindgen-cli`, then regenerate the browser bundle after changing Dziber:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.118
./scripts/build-chat.sh
```

For example:

```sh
cargo run --no-default-features --features sql,mail
cargo run --no-default-features --features sql,webmail
cargo run --no-default-features --features ldap,mail,jabber
```

Start the development server with the default SQL backend, rebuilding the application when source files change:

```sh
topcoat dev
```

Then open <http://127.0.0.1:3000>.

## Project structure

| Path | Contents |
| --- | --- |
| `freenit.toml` | Application, database, LDAP, mail, Stalwart, calendar, and Jabber settings. |
| `src/config.rs` | Typed configuration and file loading. |
| `src/app/mod.rs`, `src/app/` | The route tree: the router, layouts, pages, and routes. |
| `src/features/mod.rs`, `src/features/` | Authentication, account backends, and service integrations. |
| `src/db/mod.rs` | The SQL or LDAP connection. |
| `assets/` | Dziber's generated WASM chat client and bootstrap. |
| `toasty/` | The database migrations. Commit them with your code. |

Pages in `app` use features and components. A feature can use shared components but never imports from `app`, and shared components never import from a feature.

## Learn more

- [Getting started](https://github.com/tokio-rs/topcoat/blob/main/docs/getting_started.md)
- [Guides](https://github.com/tokio-rs/topcoat/tree/main#learn-topcoat)
