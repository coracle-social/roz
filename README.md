# Roz

A notary for nostr.

The idea is that if many parties run independent notaries, you can get a fairly high level of confidence when an event first appeared on the network. This can be useful for many things, such as key rotation schemes.

Long-term, it would make more sense to add this functionality to relays rather than run independent service providers. To that end, see [this draft NIP](https://github.com/nostr-protocol/nips/pull/1737).

![roz](./roz.jpg)

# Usage

To build and run:

`cargo build --release && ./target/release/roz`

To make requests:

`curl localhost:3981/notary/a331492f1cb3b923feb4b707f3a202ea8cbe121e399bc2614304be98cd3bc3bb`

This will return either a JSON object with a `seen` key containing the unix timestamp (seconds granularity) the event was first seen, or an `error` key containing more information.

# Roadmap

This project is a proof of concept. Possible improvements:

- [ ] Allow configuring the port (currently it uses 3981)
- [ ] Allow configuring the relay urls the notary listens to
- [ ] Create a nostr event that advertises notary services
