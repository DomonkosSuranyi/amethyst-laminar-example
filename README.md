# amethyst-laminar-example
Testing and demoing the laminar protocol using amethyst's ECS framework

## Usage
`cargo run server` - starts the server (server should be started before starting the client)  
`cargo run client` - starts the client

## Behavior
The client sends a message in every frame, the server responds to it. Between 5 and 10 seconds the client stops sending those messages,
here laminar-hartbeats are being sent only.
