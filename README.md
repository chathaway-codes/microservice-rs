# microservice-rs

A toolset to help create and manage servers deployed to Kubernetes. This package provides the base functionality;
consider using microservice-base which wraps this package with a collection of default modules.

The main value-add in this framework is the Module lifecycle. Modules are reusable components shared across crates that
add functionality to a server. Each module implements its own API, but conforms to this life cycle:

1. Register the module with ModuleCollector

- Each module has a unique key which prevents registering duplicate modules, but allows a transitive inclusion.
- Registration should be very lightweight, but should include parsing/validating flags, grabbing system resources
   (think TCP ports), and preparing the configuration struct

2. Configuration with ModuleConfigurator

- Modules can access things they depend on and perform configuration; for example, adding a HTTP route to the HTTP
    server module, or taking a copy of the arc-mutex-wrapped database
- Modules have the option of registering a callback with one of the ServerHealth slots:
  - on_healthy: called when all modules have finished configuring and the server is ready to accept traffic
  - on_shutdown: called when the server is shutdown
