# arcflash
A little proxy for using Surge and TouchOSC in close harmony

## Configuration
A TOML-file keeps track of options, and two peers: a controller and an instrument. For each peer, two sides of the connection must be defined:
- local:
  - a listening address (local ip)
  - listening port (local port)
- remote:
  - a receiving address (remote ip)
  - a receiving port (remote port)



## Design
We asssume there to be two other OSC peers: an instrument and a controller. Two async handlers are spawned to take care of each peer.

Each handler knows the peer it's listening to as `peer_recv` and the peer it forwards packets to as `peer_send`.

While handling the packets, inspection of the packets is done. Based on address and the type of peer (instrument or controller) messages may be changed.

### Message routing
- Receive packet on handler
- Unbundle packet and pass on (recursive function to unbundle nested packets)
- Pass message through extension filter (optional)
- Send packet
