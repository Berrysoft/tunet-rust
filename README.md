# tunet-rust
清华大学校园网 Rust 库与客户端。

[![Azure DevOps builds](https://strawberry-vs.visualstudio.com/tunet-rust/_apis/build/status/Berrysoft.tunet-rust?branch=master)](https://strawberry-vs.visualstudio.com/tunet-rust/_build)

## 平台支持
支持并提供如下目标的预编译程序：

* i686-unknown-linux-gnu
* x86_64-unknown-linux-gnu
* arm-unknown-linux-gnueabihf
* aarch64-unknown-linux-gnu
* riscv64gc-unknown-linux-gnu
* mips-unknown-linux-gnu
* mipsel-unknown-linux-gnu
* mips64-unknown-linux-gnuabi64
* mips64el-unknown-linux-gnuabi64
* powerpc-unknown-linux-gnu
* powerpc64le-unknown-linux-gnu
* s390x-unknown-linux-gnu
* x86_64-apple-darwin
* aarch64-apple-darwin
* i686-pc-windows-msvc
* x86_64-pc-windows-msvc
* aarch64-pc-windows-msvc

## tunet
### 登录/注销
``` bash
# 使用默认（自动判断）方式登录
./tunet login
# 使用默认（自动判断）方式注销
./tunet logout
# 使用 auth4 方式登录
./tunet login -s auth4
# 使用 auth4 方式注销
./tunet logout -s auth4
```
### 在线状态
``` bash
# 使用默认（自动判断）方式
./tunet status
# 使用 auth4 方式
./tunet status -s auth4
```
### 查询/强制下线在线IP
``` bash
# 查询
./tunet online
# IP 上线
./tunet connect -a IP地址
# IP 下线
./tunet drop -a IP地址
```
### 流量明细
``` bash
# 使用默认排序（注销时间，升序）查询明细
./tunet detail
# 使用登录时间（升序）查询明细
./tunet detail -o login
# 使用流量降序查询明细
./tunet detail -o flux -d
# 使用流量降序查询明细，并按注销日期组合
./tunet detail -o flux -dg
```

## keyring
用户名和密码在第一次登录时根据提示输入，不同平台管理密码方法如下：

|平台|方法|
|-|-|
|Windows|[Windows Credential Manager](https://docs.microsoft.com/en-us/windows/win32/api/wincred/)|
|Linux|[Keyrings](https://man7.org/linux/man-pages/man7/keyrings.7.html)|
|macOS|[Keychain](https://developer.apple.com/documentation/security/keychain_services)|

对于不支持密码管理的 Linux 发行版，会回退到**明文**密码。

请不要在不信任的电脑上保存密码。可以使用如下命令删除：
``` bash
./tunet deletecred
```

## netstatus
针对 Windows, Linux, macOS 使用了平台特定的方式尝试获得当前的网络连接方式，如果是无线网连接还会获取 SSID。
如果无法获取，则尝试连接特定的网址来判断。

|平台|方法|
|-|-|
|Windows|`Windows::Networking::Connectivity`|
|Linux|[Netlink](https://wiki.linuxfoundation.org/networking/generic_netlink_howto)|
|macOS|System Configuration 与 Core WLAN|
