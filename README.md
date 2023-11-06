# Hello RuFi
This project shows how to setup a RuFi program.

## Running the examples
- Local gradient:
````shell
cargo run --bin local-gradient
````
- Distributed gradient:
You can individually start a node by opening a new terminal shell and executing:
````shell
cargo run --bin distributed-gradient node_id is_source
````
where:
- node_id is the id of the node you are running 
- is_source is -t if the node is a source for the gradient and not present otherwise 