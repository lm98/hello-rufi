# Hello RuFi
This project shows how to setup a RuFi program.

## Running the examples
In order to run any example, you'll need to open a terminal inside the project root folder `hello-rufi` and follow the instructions below:

- Local gradient:
This example will execute a gradient aggregate algorithm inside a single process.
The node topology is the following: [1] - [2] - [3] - [4] - [5].
In order to launch the program and see the output, run:
````shell
cargo run --bin local-gradient
````
- Distributed gradient:
This example will launch a node executing the gradient aggregate program as an independent process, communicating with his neighbors.
To see an output that is equivalent to the previous example's one, you'll need to have five terminal shells each running a single node.
The topology is the same as the previous example.
In order to start a node and see his output, run:
````shell
cargo run --bin distributed-gradient node_id is_source
````
where:
- `node_id` is the id of the node you are running. This is an int number between 1 and 5;
- `is_source` is `-t` if the node is a source for the gradient and `-f` otherwise;
