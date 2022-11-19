# About

Simple CLI chat functionality. P2P.

### Diagrams

## 👎 Central Server Approach
```
              ChatLog (Storage)
                   │
               ChatServer
                   │
    ┌──────────────┼──────────────┐
ChatClient     ChatClient     ChatClient
```

This is a basic approach involving having a server
process manage connections and chat state. We aren't going 
with this one. Blech, so boring.


## 👍 P2P Server
```
    ┌──────────────┬──────────────┐
ChatPeer        ChatPeer       ChatPeer
--------        --------       --------
ChatClient     ChatClient     ChatClient
ChatServer     ChatServer     ChatServer
ChatLog         ChatLog        ChatLog
```

This is an approach where each process is its own self-contained
client and server (and consensus-based log). Less boring, and waaay better!





