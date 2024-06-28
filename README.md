# Runnalo: a modern approach to programming language learning.
[![example workflow](https://github.com/Usioumeo/Testalo/actions/workflows/docs.yml/badge.svg)](https://usioumeo.github.io/Testalo/orchestrator/)
[![Coverage Status](https://coveralls.io/repos/github/Usioumeo/Testalo/badge.svg?branch=main)](https://coveralls.io/github/Usioumeo/Testalo?branch=main)



The project aims to help teachers to provide better learning experiences to students.
In particular, its modular approach allows everyone to extend it easily and integrate others' code with a one-liner.
The orchestrator crate is the default crate in this repository, and its documentation is readable [here](https://usioumeo.github.io/Testalo/orchestrator/).
Then, we developed different plugins:
- Backend: starts a rocket backend serving a generic frontend. See [Documentation](https://usioumeo.github.io/Testalo/backend/)
- SQL-abstractor: A plugin to connect to an SQL database (at the moment, we explicitly support only PostgreSQL). See [Documentation](https://usioumeo.github.io/Testalo/sql_abstractor/);
- Rust-default: compute a Rust exercise from a template file. See [Documentation](https://usioumeo.github.io/Testalo/rust_default/) 


This project was started as my final university dissertation. It is still a work in progress, and any help would be appreciated.

## Installation
For a development installation, the following dependencies are needed:
- Rustup, with at least one nightly toolchain. (the project is compilable with stable, but rust-exercise execution requires a nightly toolchain).
- Standard compilers. On Ubuntu it is called build-essentials; on Arch systems, it is called base-devel.
- trunk for the backend/frontend. It can be easily installed with cargo install trunk:
```sh
cargo install trunk
```
- tarpaulin: to generate coverage. It is not strictly required, but it is helpful. It can be installed with:
```sh
cargo install tarpaulin
```
- PostgreSQL: to test we deployed PostgreSQL in a docker container. If needed, it is possible to deploy it elsewhere. If you want to use our testing script you will need docker installed on your system.
