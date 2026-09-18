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

三段式和旧格式配置：`/etc/nat.conf`

TOML 配置：`/etc/nat.toml`

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

## 与 Realm 的关系

- 两个项目在三段式模式下共用 `/etc/nat.conf`，切换时会先备份并保留该文件。
- `nat.service` 与 `realm.service` 互斥，只应启用其中一个。
- nftables-nat-rust 使用 nftables DNAT 规则；Realm 使用用户态进程转发。
- 三段式 `本地端口:远程地址:远程端口` 可以在两个项目之间直接切换。
- `SINGLE`、`RANGE`、`REDIRECT`、`DROP` 和 TOML 配置仅适用于 nftables-nat-rust，Realm 不会应用这些规则。

切换到 Realm：

```bash
bash <(curl -sSLf https://raw.githubusercontent.com/Taylor000/realm/main/switch-to-realm.sh)
```

切回 nftables-nat-rust：

```bash
bash <(curl -sSLf https://raw.githubusercontent.com/Taylor000/nftables-nat-rust/master/switch-to-nft.sh)
```

切换后检查：

```bash
forward-status
systemctl status realm
systemctl status nat
```

## 清理与卸载

```bash
bash <(curl -sSLf https://raw.githubusercontent.com/Taylor000/nftables-nat-rust/master/clear-rules.sh)
bash <(curl -sSLf https://raw.githubusercontent.com/Taylor000/nftables-nat-rust/master/uninstall.sh)
```

同时删除配置：

```bash
REMOVE_NAT_CONFIG=1 bash <(curl -sSLf https://raw.githubusercontent.com/Taylor000/nftables-nat-rust/master/uninstall.sh)
```
