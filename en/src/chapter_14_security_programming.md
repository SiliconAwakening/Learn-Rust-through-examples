# Chapter 14: Security Programming

Rust eliminates entire classes of vulnerabilities by construction — buffer overflows, use-after-free, null pointer dereferences, and data races are compile-time errors, not runtime exploits. But memory safety is not the whole story. A secure application must also validate input, manage secrets, authenticate users, and resist the attacks that target any web service. This chapter covers the practical security practices you add *on top of* Rust's guarantees.

## Learning Objectives

- Understand what Rust prevents automatically and what it does not.
- Validate and sanitize untrusted input.
- Hash and salt passwords correctly.
- Manage secrets without leaking them into logs or source control.
- Apply TLS and common web-security headers.

---

## 14.1 What Rust prevents, and what it does not

Rust's ownership model makes these impossible in safe code:

- **Buffer overflows** — bounds-checked array access panics instead of overflowing.
- **Use-after-free and double-free** — the move/borrow system prevents aliased mutable access to freed memory.
- **Null pointer dereferences** — there is no null; absence is `Option<T>`.
- **Data races** — `Send`/`Sync` make concurrent mutation without synchronization a compile error.

What Rust does **not** prevent:

- **Logic bugs** — correct memory, wrong answer.
- **Integer overflow** in release builds (it wraps; use `checked_*` / `saturating_*` when it matters).
- **Panics from untrusted input** — `unwrap` on attacker-controlled data crashes the process.
- **Injection** — building SQL, HTML, or shell commands from raw input.
- **Leaking secrets** — a `String` holding a password is just memory the compiler will happily print.

Security is therefore about the boundary between your program and untrusted data.

---

## 14.2 Validate input at the boundary

The first line of defense is to reject malformed input before it reaches your domain logic. `serde` deserialization already catches type errors; for semantic rules, use the `validator` crate:

```toml
[dependencies]
validator = { version = "0.16", features = ["derive"] }
```

```rust
use validator::Validate;

#[derive(serde::Deserialize, Validate)]
struct Signup {
    #[validate(length(min = 3, max = 32))]
    username: String,
    #[validate(email)]
    email: String,
    #[validate(length(min = 8))]
    password: String,
}

fn handle_signup(input: Signup) -> Result<(), String> {
    input.validate().map_err(|e| e.to_string())?;
    Ok(())
}
```

Treat all external data — HTTP bodies, query strings, environment variables, file contents — as untrusted until validated.

---

## 14.3 SQL and command injection

The rule is absolute: **never interpolate untrusted data into a command string.** Use parameter binding for SQL (Chapter 11) and typed argument arrays for subprocesses:

```rust
use std::process::Command;

// Good — arguments are passed, not parsed by a shell.
let output = Command::new("ls")
    .arg("-l")
    .arg(user_path)        // safe even if it contains spaces or ";"
    .output()?;
```

Avoid `Command::new("sh").arg("-c").arg(format!("ls {user_path}"))` — that hands the user's input to a shell and reopens injection.

---

## 14.4 Passwords: hash, never store

Never store passwords in plaintext or with a reversible cipher. Use a slow, salted hash designed for passwords. `argon2` is the current standard:

```toml
[dependencies]
argon2 = "0.5"
```

```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

fn hash_password(plain: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(plain.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

fn verify_password(plain: &str, stored: &str) -> Result<(), argon2::password_hash::Error> {
    let parsed = PasswordHash::new(stored)?;
    Argon2::default().verify_password(plain.as_bytes(), &parsed)
}
```

The stored string embeds the salt and parameters, so verification is a one-liner. Never roll your own hashing.

---

## 14.5 Secrets management

A secret (API key, database password, token) must satisfy three rules: it comes from the environment, not the source; it is loaded once and held in memory; and it never reaches logs.

```rust
use std::env;

struct Config {
    db_url: String,
    api_key: String,
}

impl Config {
    fn from_env() -> Result<Self, String> {
        Ok(Config {
            db_url: env::var("DATABASE_URL").map_err(|_| "DATABASE_URL missing")?,
            api_key: env::var("API_KEY").map_err(|_| "API_KEY missing")?,
        })
    }
}
```

Practical safeguards:

- Load at startup from environment variables or a secret manager — never hard-code, never commit to git.
- Mark secret fields so logging crates skip them (`tracing` supports `#[redact]`-style patterns via `secrecy`).
- Use the `secrecy` crate to wrap secrets in a `Secret<String>` that does not implement `Display`, so an accidental `println!` is a compile error.

```toml
[dependencies]
secrecy = "0.8"
```

```rust
use secrecy::Secret;
let api_key: Secret<String> = Secret::new(env::var("API_KEY")?);
// println!("{}", api_key); // would not compile
```

---

## 14.6 TLS

Plaintext on the public internet is inexcusable. Use `rustls` (a pure-Rust TLS stack) to terminate TLS, either inside your server or at a reverse proxy. For client HTTPS, `reqwest` uses rustls by default:

```rust
let resp = reqwest::get("https://example.com").await?.text().await?;
```

Pin to specific TLS versions (require TLS 1.2+) and a curated cipher list; the defaults are conservative and usually correct.

---

## 14.7 Web security headers

A few response headers stop whole classes of browser attacks:

| Header | Purpose |
|--------|---------|
| `Content-Security-Policy` | Restrict where scripts/styles may load from — defeats most XSS. |
| `Strict-Transport-Security` | Force HTTPS for future visits (HSTS). |
| `X-Content-Type-Options: nosniff` | Stop MIME-type sniffing. |
| `X-Frame-Options: DENY` | Prevent clickjacking via iframes. |

In `axum`, add these with `tower_http::set_header::SetResponseHeaderLayer`, or use a dedicated middleware. CSP is the most powerful — a strict policy stops reflected XSS even if you have a rendering bug.

---

## 14.8 Authentication and sessions

For cookie-based authentication:

- Issue a random, unguessable session token (use `getrandom` or `uuid` v4).
- Store the session server-side, mapped to a user id, with an expiry.
- Set the cookie `HttpOnly` (no JS access), `Secure` (HTTPS only), and `SameSite=Lax` (CSRF defense).
- Rotate the token on privilege change (login, privilege escalation).

For stateless tokens (JWT), sign them with a strong algorithm (EdDSA or HS256 with a long key), set a short expiry, and revoke via a server-side denylist for sensitive operations.

---

## 14.9 Best Practices

1. **Treat all external input as hostile** until validated.
2. **Bind, never interpolate** — for SQL, shell, and URL construction.
3. **Hash passwords** with Argon2; never store or log them.
4. **Load secrets from the environment**, wrap them so they cannot be printed.
5. **TLS everywhere** on the public internet.
6. **Set security headers**, especially CSP.
7. **Keep dependencies current** — `cargo audit` flags known vulnerabilities.

---

## 14.10 Summary

Rust removes the memory-safety attack surface, which is a large fraction of real-world vulnerabilities. What remains is the application layer: validating input, preventing injection, hashing passwords, protecting secrets, and terminating TLS. Combine Rust's compile-time guarantees with these boundary disciplines and you have a notably harder target than most stacks.

### Exercises

1. Add `validator` to a signup handler and reject usernames shorter than 3 characters or emails that are not valid.
2. Hash and verify a password with `argon2`, and store the hash string in a SQLite row.
3. Wrap an API key in `secrecy::Secret` and confirm the compiler rejects an accidental `println!`.
4. Add `Strict-Transport-Security` and a basic `Content-Security-Policy` header to an Axum router.
