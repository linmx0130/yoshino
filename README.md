Project Yoshino
===
Simple structural data modeling for Rust.

Yoshino automatically manages how the structural data should be persisted in 
(SQLite) database, so you only need to focus on your business logic.

## Code Structure
* `core` - Core data types/abstractions
* `derive` - Marcos for deriving code to implement schema trait.
* `sqlite` - SQLite database adaptor
* `user` - User identity type

## Copyright and License
Copyright 2022-present Mengxiao Lin <<linmx0130@gmail.com>>.

This project is licensed under [MIT License](LICENSE).