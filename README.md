# nftables-nat-rust

## 安装

```bash
bash <(curl -sSLf https://raw.githubusercontent.com/Taylor000/nftables-nat-rust/master/setup.sh)
```

TOML 模式：

```bash
bash <(curl -sSLf https://raw.githubusercontent.com/Taylor000/nftables-nat-rust/master/setup.sh) toml
```

## 配置

配置文件：`/etc/nat.conf`

```text
本地端口:远程IP或域名:远程端口
33351:node.example.com:33344
33352:[2001:db8::1]:443
```

## 管理

```bash
systemctl status nat
systemctl restart nat
journalctl -fu nat
forward-status
nft list ruleset
```

## 切换与卸载

```bash
bash <(curl -sSLf https://raw.githubusercontent.com/Taylor000/nftables-nat-rust/master/switch-to-nft.sh)
bash <(curl -sSLf https://raw.githubusercontent.com/Taylor000/nftables-nat-rust/master/clear-rules.sh)
bash <(curl -sSLf https://raw.githubusercontent.com/Taylor000/nftables-nat-rust/master/uninstall.sh)
```

同时删除配置：

```bash
REMOVE_NAT_CONFIG=1 bash <(curl -sSLf https://raw.githubusercontent.com/Taylor000/nftables-nat-rust/master/uninstall.sh)
```
