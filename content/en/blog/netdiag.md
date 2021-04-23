---
date: 2021-04-21
author: "Will Glozer"
title: "Open-Sourcing Our Netdiag Crate"
linkTitle: "netdiag"
---

Kentik Labs is happy to announce the release of our first open-source
[Rust][rust] crate, [netdiag][netdiag], which provides scalable,
asynchronous, implementations of a number of low-level network
diagnostics including [ping](#ping), [trace](#trace), and a custom
diagnostic we call [knock](#knock). Netdiag is at
the core of our [synthetics][kentik_synthetics] product which monitors
application and network performance using a global network of public and
private agents.

Netdiag is built on top of [tokio][tokio] and the
[raw-socket][raw-socket] crate. Diagnostic instances are `Send` and
`Sync` so concurrent tasks can share the same underlying open sockets
and state. This provides for efficient execution of many concurrent
diagnostic tasks. Both [IPv4](#ipv4--ipv6) and [IPv6](#ipv4--ipv6) are
supported for all diagnostics.

## Example

The following async code snippet from netdiag's [ping example][example_ping]
demonstrates using [Pinger][rustdoc_pinger] to stream ping results:

```rust
let pinger = Pinger::new(&Bind::default()).await?;
let ping   = Ping { addr, count, expiry };
let stream = pinger.ping(&ping).enumerate();
pin_mut!(stream);

while let Some((n, item)) = stream.next().await {
    match item? {
        Some(d) => println!("seq {} RTT {:0.2?} ", n, d),
        None    => println!("seq {} timeout", n),
    }
    sleep(delay).await;
}
```

[Pinger::new][rustdoc_pinger_new] takes a [Bind][rustdoc_bind]
reference that allows using a specific IPv4 and/or IPv6 source
address. This can be useful on a host with multiple routable
interfaces.

```rust
let pinger = Pinger::new(&Bind::default()).await?;
```

[Pinger.ping][rustdoc_pinger_ping] returns a [Stream][rustdoc_stream]
of `Some(Duration)` or `None` results to indicate no response was
received before the expiry time elapsed.
[StreamExt.enumerate][rustdoc_enumerate] is a convenient extension to
`Stream` that returns the current iteration count in addition to the
next value.

```rust
let stream = pinger.ping(&ping).enumerate();
```

Writing async code in Rust is quite pleasant in general, however it
isn't uncommon to run into ergonomics issues around
[pinning][async_pinning]. `stream` must be pinned via
[futures::pin_mut][rustdoc_pin_mut] prior to calling `stream.next()`.

```rust
pin_mut!(stream);
```

## Ping

[Pinger][rustdoc_pinger] and the [ping][rustdoc_ping] module implement
the classic [ping][ping_wiki] diagnostic which uses ICMP echo request
& reply packets to estimate round-trip-time (RTT) and packet loss to a
host.

Ping is a simple and well-known diagnostic tool that gives a
reasonably good view of network latency and packet loss in the general
case. However it may not accurately reflect application performance as
ICMP traffic can be handled differently than typical application
protocols such as TCP and UDP.

## Trace

[Tracer][rustdoc_tracer] and the [trace][rustdoc_trace] module
implement the classic [traceroute][trace_wiki] diagnostic which sends
UDP, or TCP, packets with an increasing time-to-live (TTL) to
determine each hop in the route between source and destination.
Examining latency and packet loss for each node in a route can help
pinpoint network performance issues.

The netdiag trace implementation discovers multiple paths between
source and destination by sending multiple probes per hop while
varying the packet header. UDP probes increment the destination port
for each probe while TCP probes increment the sequence number instead
to allow targeting a specific destination port.

Unix implementations of traceroute usually default to sending UDP
probes, however networks may block, rate-limit, or otherwise alter
the profile of UDP traffic.  TCP probes can bypass some or all of
these issues.

## Knock

[Knocker][rustdoc_knocker] and the [knock][rustdoc_knock] module
implement a custom diagnostic which performs a partial TCP handshake
to estimate RTT and packet loss. Knock can provide a more accurate
view of application performance since most application traffic uses
TCP and networks often block or rate-limit ICMP traffic.

## IPv4 & IPv6

IPv4 and IPv6 support for ping is relatively simple. ICMP raw sockets
do not need the IP header when sending, and the ICMPv4 and ICMPv6 echo
packet format is identical. However the ICMP type differs, the ICMP
checksum must be calculated for ICMPv4, and a IPv4 raw socket
receives the IP header while a IPv6 raw socket does not.

IPv4 and IPv6 support for knock & trace is more complicated.  Sending
and receiving TCP & UDP packets with an IPv4 raw socket require
encoding & decoding the IP header. With [IPv6 raw sockets][rfc2292]
the kernel manages the IP header and ancillary data is used to send
and receive IP fields such as hop limit and destination address.

The [raw-socket][raw-socket] crate supports ancillary data via
[CMsg][rustdoc_cmsg] and the control buffer passed to the
[send_msg][rustdoc_send_msg] & [recv_msg][rustdoc_recv_msg] methods of
[RawSocket][rustdoc_rawsocket].

## Rust at Kentik

[Rust][rust] is one of the core languages used for systems development
at Kentik. We rely on a significant amount of open-source software and
are happy to be able to contribute back to that community. Our
[netdiag][netdiag] crate is in active use in hundreds of locations
worldwide, powering tens of thousands of active diagnostic tasks, and
we hope others find it useful too. And if you are a programmer
interested in Rust or any of the topics covered in this post, [we're
hiring!][hiring]

[rust]: https://www.rust-lang.org/
[kentik]: https://www.kentik.com/
[kentik_synthetics]: https://www.kentik.com/product/synthetics/
[netdiag]: https://crates.io/crates/netdiag
[tokio]: https://crates.io/crates/tokio
[raw-socket]: https://crates.io/crates/raw-socket
[example_ping]: https://github.com/kentik/netdiag/blob/master/examples/ping.rs
[async_pinning]: https://rust-lang.github.io/async-book/04_pinning/01_chapter.html
[rustdoc_stream]: https://docs.rs/futures/0.3.14/futures/stream/trait.Stream.html
[rustdoc_enumerate]: https://docs.rs/futures/0.3.14/futures/stream/trait.StreamExt.html#method.enumerate
[rustdoc_pin_mut]: https://docs.rs/futures/0.3.14/futures/macro.pin_mut.html
[rustdoc_bind]: https://docs.rs/netdiag/0.1.0/netdiag/struct.Bind.html
[rustdoc_knock]: https://docs.rs/netdiag/0.1.0/netdiag/knock/
[rustdoc_knocker]: https://docs.rs/netdiag/0.1.0/netdiag/knock/struct.Knocker.html
[rustdoc_ping]: https://docs.rs/netdiag/0.1.0/netdiag/ping/
[rustdoc_pinger]: https://docs.rs/netdiag/0.1.0/netdiag/ping/struct.Pinger.html
[rustdoc_pinger_new]: https://docs.rs/netdiag/0.1.0/netdiag/ping/struct.Pinger.html#method.new
[rustdoc_pinger_ping]: https://docs.rs/netdiag/0.1.0/netdiag/ping/struct.Pinger.html#method.ping
[rustdoc_trace]: https://docs.rs/netdiag/0.1.0/netdiag/trace/
[rustdoc_tracer]: https://docs.rs/netdiag/0.1.0/netdiag/trace/struct.Tracer.html
[rustdoc_rawsocket]: https://docs.rs/raw-socket/0.0.2/raw_socket/prelude/struct.RawSocket.html
[rustdoc_cmsg]: https://docs.rs/raw-socket/0.0.2/raw_socket/control/enum.CMsg.html
[rustdoc_send_msg]: https://docs.rs/raw-socket/0.0.2/raw_socket/prelude/struct.RawSocket.html#method.send_msg
[rustdoc_recv_msg]: https://docs.rs/raw-socket/0.0.2/raw_socket/prelude/struct.RawSocket.html#method.recv_msg
[rustdoc_cmsg]: https://docs.rs/raw-socket/0.0.2/raw_socket/control/enum.CMsg.html
[ping_wiki]: https://en.wikipedia.org/wiki/Ping_(networking_utility)
[trace_wiki]: https://en.wikipedia.org/wiki/Traceroute
[icmpv4]: https://en.wikipedia.org/wiki/Internet_Control_Message_Protocol
[icmpv6]: https://en.wikipedia.org/wiki/Internet_Control_Message_Protocol_for_IPv6
[rfc2292]: https://tools.ietf.org/html/rfc2292
[hiring]: https://www.kentik.com/careers/#postings
