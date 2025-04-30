# Labwhere

This is a rebuild of [Sanger Labwhere](https://github.com/sanger/labwhere). It is a service which allows laboratory users to track labware.

It is a proof of concept:
- Service only (no User Interface)
- Will it be faster?
- Will it be cleaner?
- Will it be less buggy and clunky?
- Can we build a better API? Rails support for Json API and GraphQL is average and difficult to maintain.

## Module organisation

The project is organised into two crates:

1. Library crate (declared by `lib.rs`) in which all the business logic is implemented.
2. Binary crate (declared by `main.rs`) in which all the execution logic is implemented.

The binary crate imports the business logic using `use labwhere::` construct.