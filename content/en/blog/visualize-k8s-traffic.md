---
date: 2021-10-12
author: "Evan Hazlett"
title: "Visualizing Kubernetes Traffic"
slug: "visualize-k8s-traffic"
summary: "Using the Kentik Labs open source projects you can visualize Kubernetes network traffic without modifications."
---

Kubernetes has taken the container orchestration world by storm. It makes the
complex task of multi-tenet application and service deployment easier
and more accessible to teams. But underneath is a non-trivial set of components
that must be configured and tuned to operate smoothly. Container networking
can be complex and management of a container network is vital to application
health. Using the [Container Network Interface](https://github.com/containernetworking/cni#cni---the-container-network-interface)
that provides a framework for interfacing network providers with pod networking,
containers can communicate using a variety of methods including VXLAN or BGP.
As application density increases, so does the complexity of identifying what
network resources are being utilized, how much data is being sent and received,
and where is the traffic going.

[Convis][convis] is [Kentik Labs'][kentik-labs] open source container visibility
tool that uses BPF to identify network traffic. Using Convis, we can add
context to the traffic to attribute the Kubernetes namespace, pod, and container
information. Using a few Kubernetes constructs we can create a powerful system
to visualize application traffic across our platform with no modifications or
application changes.

To enable the gathering of BPF level information for the traffic and pod, we will
create a [DaemonSet](https://kubernetes.io/docs/concepts/workloads/controllers/daemonset/)
that will schedule Convis on every one of our worker nodes.

```yaml
apiVersion: apps/v1
kind: DaemonSet
spec:
  template:
    spec:
      hostPID: true
      containers:
        - name: convis
          image: docker.io/kentiklabs/convis:latest
          env:
            - name: CONVIS_ARGS
              value: "--sink prometheus,endpoint=http://prometheus:9090/api/v1/write"
          securityContext:
            privileged: true
```

Notably to the mostly standard `DaemonSet` configuration are a couple of things.
First we will use the `hostPID` configuration option to set the container to run
in the host PID namespace. We also configure the pod to be [privileged](https://kubernetes.io/docs/concepts/policy/pod-security-policy/#privileged)
to enable access to host level resources like devices and networking.

To provide simple location based information for the reported IP addresses from Convis
we will create a [Deployment](https://kubernetes.io/docs/concepts/workloads/controllers/deployment/)
and [Service](https://kubernetes.io/docs/concepts/services-networking/service/) that
will expose a metrics endpoint that can be scraped for the location data.

```yaml
apiVersion: apps/v1
kind: Deployment
spec:
  template:
    spec:
      containers:
        - name: geoip
          image: docker.io/kentiklabs/prometheus-geoip:latest
          env:
            - name: QUERY
              value: "sum by (k8s_namespace, k8s_pod, destination_ip) (bytes_tx{})"
          command: [
            "/usr/local/bin/prometheus-geoip",
            "-D", "-p", "http://prometheus:9090",
            "-i", "destination_ip",
            "-d", "/etc/GeoLite2-City.mmdb",
            "-l", "k8s_namespace",
            "-l",
            "k8s_pod",
          ]
          ports:
          - containerPort: 8080
            name: geoip
---
apiVersion: v1
kind: Service
spec:
  ports:
    - name: geoip
      port: 8080
      targetPort: 8080
      protocol: TCP
```

This service will query Prometheus using the `QUERY` to add Geo latitude and longitude
information for each reported IP address.

Next we will configure a [Deployment](https://kubernetes.io/docs/concepts/workloads/controllers/deployment/)
for [Prometheus](https://prometheus.io/) to receive the network data from Convis.
We will also leverage a [ConfigMap](https://kubernetes.io/docs/concepts/configuration/configmap/)
for the Prometheus configuration.  This configures Prometheus to scrape the GeoIP
service to gather basic location information for the reported IP addresses.

```yaml
apiVersion: v1
kind: ConfigMap
data:
  prometheus.yml: |
    global:
      scrape_interval: 10s
    rule_files:
      - '/etc/rules/rules.yml'
    scrape_configs:
      - job_name: 'geo'
        static_configs:
          - targets: [
              'geoip:8080',
            ]
---
apiVersion: apps/v1
kind: Deployment
spec:
  template:
    spec:
      containers:
        - name: prometheus
          image: docker.io/prom/prometheus:v2.30.0
          command: [
            "/bin/prometheus",
            "--config.file=/etc/prometheus/prometheus.yml",
            "--storage.tsdb.path=/prometheus",
            "--web.console.libraries=/usr/share/prometheus/console_libraries",
            "--web.console.templates=/usr/share/prometheus/consoles/prometheus",
            "--enable-feature=remote-write-receiver",
          ]
          ports:
          - containerPort: 9090
            name: prometheus
---
apiVersion: v1
kind: Service
spec:
  ports:
    - name: prometheus
      port: 9090
      targetPort: prometheus
      protocol: TCP
```

We will also create a [Grafana](https://grafana.com/grafana/) deployment to have some graphical visualizations
for the Kubernetes traffic.

```yaml
apiVersion: apps/v1
kind: Deployment
spec:
  template:
    spec:
      containers:
        - name: grafana
          image: docker.io/grafana/grafana:8.1.4
          ports:
          - name: grafana
            containerPort: 3000
---
apiVersion: v1
kind: Service
spec:
  ports:
    - name: grafana
      port: 3000
      targetPort: 3000
      protocol: TCP
```

Using Grafana, we can create some powerful dashboards to show and filter Kubernetes
traffic by Namespace, Pod, and Container.

![namespace traffic](/static/visualize-k8s-traffic-ns-pod-container-traffic.png)

We can also use the GeoIP information to visualize external traffic. This is incredibly
valuable in identifying rogue connections to locations that are not expected or forbidden.

![external connections](/static/visualize-k8s-traffic-external-connections.png)

You can find the Kentik Labs stack on [GitHub](https://github.com/kentik/kentik-lite) to
deploy it on your own infra.

# Open Source

If this is interesting to you please join us on [GitHub][github] and [Discord][discord]
to help build the future of open source network observability!

[kentik-labs]: https://kentiklabs.com/
[kentiklabs-stack]: https://github.com/kentik/kentik-lite/
[convis]: https://github.com/kentik/convis
[prometheus]: https://prometheus.io/
[grafana]: https://grafana.com/grafana/
[discord]: https://discord.gg/kentik
[github]: https://github.com/kentik
