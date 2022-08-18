---
date: 2022-08-17
author: "Ian Pye and Will Glozer"
title: "The Songs of the Internet"
slug: "latency-as-music"
summary: "Make music with network latency."
---

[Songs of the Internet](https://songs-of-the-internet.kentiklabs.workers.dev/) is Kentik Labs' take on how to make active network checks fun.

Put any url into the box.
On submit, three [HTTP Fetch](https://kb.kentik.com/v4/Ma00.htm) tests will get spun up from three agents distributed globally.
As these report time to last byte numbers music will be created.
Each agent's results are fed to a [Tone.js](https://tonejs.github.io/) Synth and converted to a midi value from 0-127.
As your site's latency increases, your song will go up in pitch.
As jitter increases, more abrupt pitch shifts happen.