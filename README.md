# A workaround of IN clause placeholders for sqlx crate and MySQL 8

This repository provides a workaround to avoid writing a tedious code which generates placeholders
when you use `IN` clause in MySQL queries with [sqlx crate](https://crates.io/crates/sqlx).

## Motivation

If you would like to use placeholders in `IN` clause,
you must write `?` marks for each element in the given array.
For example, if you have an array of IDs whose length is five,
you must write five `?` marks in your query and call `bind` five times:

```rust
sqlx::query("SELECT * FROM table WHERE id IN (?, ?, ?, ?, ?)")
	.bind(ids[0])
	.bind(ids[1])
	.bind(ids[2])
	.bind(ids[3])
	.bind(ids[4]);
```

If the number of IDs is ten,
ten `?` marks are required and `bind` must be called ten times.

This fact is not convenience when the array of IDs have an arbitrary length.

## Solution

I propose to use the `JSON_TABLE` function of MySQL 8.
This function constructs a table from the given JSON data.
`IN` clause is substituted by `JOIN`.

An example is given in _src/main.rs_.

## How to run

First, you run an instance of MySQL server with Docker compose:

```console
$ docker compose up -d
```

Then you run the Rust program:

```console
$ cargo run
```
