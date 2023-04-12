# Runik: A ELF binary-compilant unikernel written in Rust

This is a work as a part of the bachelor's thesis of Junyang Zhang.

Runik is a unikernel written in Rust. It aims to provide a posix-compliant interface for prebuilt binaries, enabling a single application running above a hypervisor. The application and the library os code will run in the same address space.

## Set up the development environment using docker

Using a Docker container is the only tested way to build and run this project. You can setup your local environment according to the Dockerfile.

To build the environment image:

```bash
make docker_build
```

The way entering the environment is:

```bash
make docker_run
```

The command will start an interactive session in a docker container, and the current working directory is mapped into `/mnt`.

## Building and running this project

To build and run the image using qemu, simply do this inside the container:

```bash
make build
make run
```

## Project structure

### Building with arbitrary ELF binary

You should put the target app under the `app/target` directory. I made some test cases in `app` that could be compiled and placed into that directory using makefile.
