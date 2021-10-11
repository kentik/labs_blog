---
date: 2021-10-11
author: "Will Glozer"
title: "Convis - Open Source Container Visibility"
slug: "container-visibility"
summary: "Convis is an open source network visibility project demonstrating the use of BPF kprobes and tracepoints to attribute network traffic to process, container, and node."
---

[Convis][convis] is [Kentik Labs'][kentik-labs] open source container visibility
tool, first published for [eBPF Summit 2021][ebpf-summit]. We developed convis
as a demonstration of using BPF kprobes and tracepoints to attribute network
traffic to source and destination process, container, and node. This has
numerous useful applications including determining which processes and
containers receive or generate the most traffic and security forensics and
alerting on traffic from unexpected IP addresses.

![pod traffic](/static/convis-image-1.png)

Traditional network visibility tools like [NetFlow][netflow] and [sFlow][sflow]
rely on network hardware such as switches and routers inspecting network traffic
and generating flow records. This worked well when most applications were
running directly on a physical or virtual node, communicating over the
network. However the rise of containers has resulted in significant amounts of
traffic occurring between containers within a single node, no longer visible on
the network. Container orchestrators like Kubernetes may also use overlay
networks and/or encryption for traffic between nodes, further reducing
network-level visibility.

Convis captures traffic at the lowest possible level, when an application calls
into the kernel to accept or initiate a connection. Using [eBPF][ebpf] these
programs can inspect the arguments to the call, extracting for example the
source and destination IP address and ports of a new TCP connection.

## Process & Container Info

In addition to the standard 5-tuple used to identify network traffic: protocol,
source address, source port, destination address, destination port, convis also
tracks the process and container that initiated or accepted a connection. This
provides visibility into exactly which processes and containers are
communicating with each other, and with the internet. Convis uses eBPF's
[bpf_get_current_pid_tgid()][bpf-helpers] helper function to extract the calling
pid when a kprobe or tracepoint is hit, and then looks up the rest of the
process information via `procfs`.

Container details are requested from either the Docker API or Container Runtime
Interface (CRI), using the container ID which is extracted from the cgroup paths
assigned to the container. When running in a non-containerized environment
convis will output process details only.

## eBPF Programs

Convis attaches [BPF programs][bpf-programs] to the process exec & exit
tracepoints and kprobes to the kernel functions to connect, accept, and close
TCP connections. Example:

The `sched_process_exec` and `sched_process_exit` tracepoints generate an event
whenever a process is executed or exits, allowing convis to efficiently maintain
a cache of all processes executing on the node.

```c
SEC("tracepoint/sched/sched_process_exec")
int bpf_trace_sched_process_exec(sched_process_exec_ctx *ctx) {
    struct header event = {
        .kind = EXEC,
        .pid  = ctx->pid,
    };

    int rc = bpf_perf_event_output(ctx, &events, BPF_F_CURRENT_CPU, &event, sizeof(event));
    if (rc != 0) {
        bpf_printk("exec event output failure: %d\n", rc);
    }

    return 0;
}
```

Entry and exit of the `tcp_v4_connect` kernel function is intercepted via
kprobes to track whenever a TCP connection is initiated. The `inet_csk_accept`
and `tcp_close` functions are intercepted via kprobes to track when a new
connection is accepted and when a TCP connection is closed. The number of bytes
sent and received is also output when the connection is closed. Example:

```c
SEC("kretprobe/inet_csk_accept")
int bpf_call_inet_csk_accept(struct pt_regs *ctx) {
    struct sock *sk = (void *) PT_REGS_RC(ctx);

    if (sk == NULL) {
        return 0;
    }

    u64 pid_tgid = bpf_get_current_pid_tgid();
    u32 pid = pid_tgid >> 32;
    u32 tid = pid_tgid;

    struct sock_common sc;
    bpf_probe_read(&sc, sizeof(sc), &sk->__sk_common);

    struct accept event = {
        .header = {
            .kind = ACCEPT,
            .pid  = pid,
        },
        .socket = {
            .proto = 6,
            .saddr = sc.skc_rcv_saddr,
            .sport = sc.skc_num,
            .daddr = sc.skc_daddr,
            .dport = ntohs(sc.skc_dport),
        },
    };

    int rc = bpf_perf_event_output(ctx, &events, BPF_F_CURRENT_CPU, &event, sizeof(event));
    if (rc != 0) {
        bpf_printk("accept event output failure: %d\n", rc);
    }

    return 0;
}
```

## Prometheus Sink

Convis can use the [Prometheus][prometheus] remote write protocol to write
metrics to any Prometheus instance, including [Grafana
Cloud][grafana-cloud]. Simply specify `prometheus` as the sink and provide a URL
to the remote write endpoint: `--sink
prometheus,endpoint=<url>,[username=<user-name>],[password=<password>]`

Valid arguments for the Prometheus sink are:

* endpoint: URL of the Prometheus server's remote write endpoint
* username: optional username to use for HTTP authentication
* password: optional password to use for HTTP authentication

## New Relic Sink

Convis can also output directly to New Relic's Event API. Simply specify
`newrelic` as the sink and provide your New Relic account number and insert key:
`--sink newrelic,account=<account-ID>,key=<insert-key>`.

Valid arguments for the New Relic sink are:

* account: New Relic account ID
* key: Event API insert key
* region: optional New Relic region (US or EU)

## Open Source

Convis is open source and Kentik Labs welcomes comments and contributions. Join
us on [GitHub][github] and [Discord][discord] and help build the future of open
source network observability!

[convis]: https://github.com/kentik/convis
[kentik-labs]: https://kentiklabs.com/
[ebpf-summit]: https://ebpf.io/summit-2021/
[netflow]: https://en.wikipedia.org/wiki/NetFlow
[sflow]: https://en.wikipedia.org/wiki/SFlow
[ebpf]: https://ebpf.io/
[bpf-helpers]: https://man7.org/linux/man-pages/man7/bpf-helpers.7.html
[bpf-programs]: https://github.com/kentik/convis/blob/master/bpf/bytecode.c
[prometheus]: https://prometheus.io/
[grafana-cloud]: https://grafana.com/products/cloud/
[github]: https://github.com/kentik/convis
[discord]: https://discord.gg/kentik
