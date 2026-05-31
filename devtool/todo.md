# devtool — todo

## Done

- [x] JSON formatter — pretty-print with error
- [x] Base64 — encode/decode simultaneously
- [x] JWT decoder — header + payload as pretty JSON
- [x] Hash — MD5 / SHA1 / SHA256 / SHA512 live
- [x] URL encode/decode — simultaneously
- [x] UUID generator — v4, Enter/Space, history
- [x] Timestamp — unix/ISO/local/day; current time when empty
- [x] Regex — pattern + test string, Tab to switch, match positions

## In progress

- [ ] Text Transform — UPPER/lower/Title/camelCase/PascalCase/snake_case/kebab-case/SCREAMING_SNAKE
- [ ] Number Base — decimal ↔ hex ↔ binary ↔ octal live
- [ ] String Stats — chars, words, lines, bytes, unique chars
- [ ] HTML Entity — encode/decode `&amp;` `&lt;` `&#x...;` etc.

## Backlog

- [ ] Cron parser — human-readable description + next 5 fire times (needs `croner` or similar dep)
- [ ] Diff — two text areas, colored added/removed lines
- [ ] YAML ↔ JSON — paste either, get the other (needs `serde_yaml` dep)
- [ ] JWT signer/verifier — HS256 sign + verify with key field
- [ ] HMAC — input + key + algorithm → signature
- [ ] PEM decoder — subject/issuer/expiry/SANs from certificate
