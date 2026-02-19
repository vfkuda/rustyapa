# Project Work
Многомодульный Rust-проект (workspace).

## Структура
- `parser/` — крейт /библиотека парсера данных транзакций в разных форматах
- `src/bin/` — CLI-приложения (бинари), использующие `parser`
- `src/bin/converter`
- `src/bin/comparer`

## (DEVELOPMENT) Как запустить 
```bash
cargo test
cargo test -p parser
cargo build
```