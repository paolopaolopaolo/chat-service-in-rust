# chat-server

## About

Simple CLI chat functionality built on TCP. Has server-side and client-side components.

```
>> You are Dan!
┌─────────────────────────────────────────────────────────────┐
│  Loretta: What are you wearing?                             │
│  Dan: That is very inappropriate. This is unencrypted you   │
│    know!                                                    │
│  Loretta: Oh no! And I also forgot that there's no app-lay  │
│    er logic. So all requests to the right port will show up │
│    here.                                                    │
│  GET / HTTP/1.1                                             │
│  Host: 0.0.0.0:9000                                         │
├─────────────────────────────────────────────────────────────┤
│  Oh poo                                                     │
└─────────────────────────────────────────────────────────────┘
```

## Approach

### Server-side
Start at `examples/server.rs` for the server implementation.

Run `cargo run --examples server` to run the server implementation

```
ChatServer(port 9000)─┬─ChatLog(port8000)                        
       ┌──────────────┼──────────────┐
ChatClient     ChatClient     ChatClient
```

This uses a basic approach involving having 2 threads.

`ChatServer` accepts TCP connections from clients at port 9000. Clients that
  write on this connection will be writing directly to the chat log.

`ChatLog` has a public shared-state text-log. The example creates a TCP Listener that   
  accepts connections from clients at port 8000 and writes the latest lines from the text-log to it. Clients can only read off this connection, and will see the latest log items.

There is no App-Level data-framing. Connections on 8000 will show everything that gets written to 9000. You can write anything to 9000.

### Client-side

Start at `src/main.rs` for the CLI "windowed" implementation.

Run with `cargo run`.

This uses 3 threads:

```
ChatWindow
 ├ Listen on port 8000 for new lines of text and render the chat feed with text
 └ Listen on an mpsc channel for key-strokes from ChatInput (scroll up/down)

ChatInput
 └ Uses `crossterm` to update input text and writes data (on pressing Enter) 
   to port 9000
```

## Things left to-do
* [x] Implement a blocking fancy UI/UX flow for entering the name.
* [x] Implement screen-resize actions and have a dynamic screen-size.
    * [x] Bug where changing terminal size disconnects the client.
* [ ] Bug where pressing CTRL + C once doesn't kill the outstanding TCP connection.
* [ ] Implement something on the clients for when the server closes the connections.
