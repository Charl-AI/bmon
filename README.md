# bmon -- The Bottleneck Monito{rs}

bmon is a CLI tool for monitoring system metrics and diagnosing potential bottlenecks in GPU-accelerated training.

bmon has 5 features, which can be turned on/off mostly independently:
  - Display key GPU metrics (utilization, memory usage, temperature, power etc.)
  - Display key disk metrics
  - Display key network metrics
  - Display metrics for running GPU compute processes (CPU utilization, RAM etc.)
  - Infer potential problems and bottlenecks from metrics (e.g. suspected disk bottleneck, GPU thermal throttling etc.)

With most features turned off, bmon may be used as a minimalist `nvidia-smi`. With all features enabled, it becomes your personal research assistant.

> bmon is in the alpha stage of development. Until version 1.0, there may be bugs, missing features, and API changes between versions.

TODO: 
 - Improve table alignment and sizing
 - Write bottleneck diagnosis document explaining how to use these metrics
 - Add bottleneck diagnises from the document to the auto-detection

## Motivation

Different labs often have very different compute setups and training pipelines optimized for one setup are likely to not be optimal for others. Defaults in libraries and other people's code are often poor -- to get the most out of our resources, we should all be tuning our training pipelines for our own setups.

When I am prototyping and training deep learning models, I am constantly watching `nvidia-smi` and `htop` to get an idea of how well I am utilising my system resources. This works ok, but has a few downsides:
  1. visual clutter (both tools give large outputs, where only a few key metrics are actually useful for me).
  1. limited insight (these tools on their own do not monitor disk/network, I could use more tools like `iostat`, but that exacerbates problem 1).
  1. interpretation effort (surely we should be able to automate the interpretation of system metrics to diagnose issues!)

bmon attempts to solve these problems by presenting a clean and simple way of monitoring resources. I am particularly interested in its potential to give researchers informed analysis about program performance.

## Installation

TODO

## Usage 

TODO

## Contributing

I would love some help with this project! I am mostly building this tool because I want to use it, but I would be even more happy if other people want to use it :) Suggestions, issues, and PRs are welcome.
