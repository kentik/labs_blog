---
date: 2021-08-18
author: "Kentik"
title: "Announcing Kentik Labs"
slug: "announcing-kentik-labs"
summary: "Today we announce the launch of Kentik Labs, our new hub for the developer, DevOps and SRE community. With the tools we’re open sourcing, you’ll be able to observe key network telemetry for a competitive advantage."
---

Distributed applications, by nature, rely on the network to function. As applications go from single-host, single software stack to multi-host, heterogeneous environments, being able to observe key network telemetry becomes a competitive advantage. That's why we're announcing the launch of Kentik Labs.

Using the tools we're open sourcing today, you can generate network metrics using our agents, like [convis](https://github.com/kentik/convis) (eBPF) or [kprobe](https://github.com/kentik/kprobe) (packet capture), and convert them to a common format using [kTranslate](https://github.com/kentik/ktranslate). These metrics can then be stored and leveraged in the observability tools you already have deployed, including New Relic, Kafka, Influx and Prometheus. From there, you can use your favorite visualization tool like Grafana or the InfluxDB UI.

[Convis](https://github.com/kentik/convis) (container visualization) is a small eBPF and Rust tool showing how to use eBPF to track TCP connections on a Linux host. It's small enough to get into as a tutorial but also provides useful data about who every process on your system is talking to. Watch our [eBPF Summit](https://ebpf-summit-2021.sessionize.com/session/276419) video where we explore how to output network traffic statistics to JSON.

This is just the beginning - we're looking to work with the community to expand the different types of telemetry that kTranslate accepts and different backends that it supports, including OpenTelemetry.

Curious? Come kick the tires. Check out the quickstart guides for listening to [SNMP](https://github.com/kentik/ktranslate/wiki/SNMP-Quickstart) and [NetFlow](https://github.com/kentik/ktranslate/wiki/New-Relic-Flow-Collection-Quickstart). These will get you running collecting passive information about how devices on your network are doing. Then go further and create some alerting around things like when your NAS disk is getting full or a non-white listed IP is sending data from inside your house.

If you want to get started with eBPF, you can build and run convis with:

```bash
cargo build --release
sudo target/release/convis -v
```

...And then watch the world of network connections fly by.

[We want to talk to you](mailto:labs@kentik.com). Come hack with us! We're looking for people interested in Go, Rust and C (eBPF) languages. Code will all be open sourced and we're working to shape how the next generation of networks are run. Learn more at [kentiklabs.com](http://www.kentiklabs.com).
