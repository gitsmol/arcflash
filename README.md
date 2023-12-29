# arcflash
A little proxy for using Surge and TouchOSC in close harmony

## Design
We asssume there to be two other OSC peers: an instrument and a controller. Two async handlers are spawned to take care of each peer.

Each handler knows the peer it's listening to as `peer_recv` and the peer it forwards packets to as `peer_send`.

While handling the packets, inspection of the packets is done. Based on address and the type of peer (instrument or controller) messages may be changed.
