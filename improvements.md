# Code Review: mailport

## What you're doing right

- Modular file layout (`smtp/` with one file per command) is clean and will scale
- Session state tracking with ordering validation (503 on wrong sequence) is the right approach
- `data.rs` is stubbed and ready — good awareness it's coming
- Async with tokio is appropriate

## Immediate bugs & critical issues
2. **`try_read` in a fixed 512B buffer breaks for anything larger and for pipelining** — You read at most once (512 bytes), then break.
If a client sends `HELO a\r\nMAIL FROM:<x>\r\n` in one TCP segment (common with PIPELINING), you only parse the first command. The rest is lost.
You need a `BufReader` with proper line-buffered reads via `read_until(b'\n')` or `read_line()`.

3. **Regex command regex is buggy** — `(HELO)\s?(.*)` against `"HELOO"` matches group1=`"HELO"` group2=`"O"`, so `HELOO` is accepted as HELO with junk.
Similarly `(MAIL FROM:)\s?(.*)` only handles one optional whitespace character — `"MAIL FROM:  <x>"` (two spaces) produces payload `" <x>"` which
fails your `<` boundary check. SMTP commands are also **case-insensitive** per RFC 5321; `helo` won't work at all.

4. **`try_write` ignores short writes** — It returns `usize` (bytes written) which you discard. If the kernel can't write the full response, the
client gets a truncated reply. You need `write_all` instead.

5. **CRLF check panics on short input** — `trimmed_buff[trimmed_buff.len() - 2]` blows up if the client sends just `\r\n` (bytes_num == 2) or an
empty read (bytes_num == 0). Always guard the length.

## Structural hurdles coming soon

1. **DATA is fundamentally incompatible with current parse-once approach** — DATA is multi-line and ends with `\r\n.\r\n`. Your `read_command` reads a single 512-byte chunk. You'll need to refactor the read loop significantly — state-machine per connection that knows "I'm in DATA mode, accumulate until terminator".

2. **`&TcpStream` lifetime on every command struct** will bite you when you need to do more complex async flows (e.g., timeouts, pipelining, TLS wrapping). Consider using `Arc<TcpStream>` or wrapping it in a custom `Connection` struct with a `BufReader`/`BufWriter`.

3. **No common trait for commands** — `do_command` is a growing match statement. A `Command` trait with `async fn execute(...)` would let you dispatch generically.

4. **No EHLO** — Modern SMTP starts with EHLO, not HELO. Without it you can't advertise SIZE, STARTTLS, PIPELINING, CHUNKING, etc. Clients like Thunderbird or Postfix will EHLO first.

5. **UTF-8 assumption in `String::from_utf8`** — During DATA, email bodies can contain arbitrary bytes (8BITMIME). This will fail.

6. **`bytes_num` starts at 0, `#[allow(unused)]` is stale/misleading** — it is used, but the attribute suggests confusion.

## Summary

The command dispatch pattern and session tracking are fine foundations. The biggest immediate blockers are (1) missing 220 banner, (2) the fragile `try_read`-once model, and (3) no DATA multi-line reading strategy. I'd fix the buffered reader approach before adding more commands.
