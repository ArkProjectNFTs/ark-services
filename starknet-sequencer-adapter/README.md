# fast-indexer
## Installation
Once you clone the repo do:

```sh
cargo build && cargo deb
sudo dpkg -i target/debian/fast-indexer_0.1.0_amd64.deb
```

It will install a `fast-indexer` binary under `/usr/bin/` and also a `fast-indexer.service` under
`/lib/systemd/system/`. Once installed the service will be enabled and started.


## Usage
You can see the status using:
```sh
sudo systemctl status fast-indexer
```

[systemctl](https://manpages.debian.org/stretch/systemd/systemctl.1.en.html) is a wrapper around systemd services so it offers many goodies (stop/restart etc).

To see the logs you can use the [journalctl](https://manpages.debian.org/stretch/systemd/journalctl.1.en.html):
```sh
sudo journalctl -u fast-indexer.service
```


## Uninstall
In order to completely remove the binary and the systemd all you need to do is:

```sh
sudo dpkg --purge fast-indexer
```