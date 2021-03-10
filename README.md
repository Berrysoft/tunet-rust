# tunet-rust
清华大学校园网 Rust 库与客户端。

这一版本与 [TsinghuaNet](https://github.com/Berrysoft/TsinghuaNet) 中的 TsinghuaNet.CLI 功能相同。

[![Azure DevOps builds](https://strawberry-vs.visualstudio.com/tunet-rust/_apis/build/status/Berrysoft.tunet-rust?branch=master)](https://strawberry-vs.visualstudio.com/tunet-rust/_build)

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

## tunet-native
暴露了适用于C语言的接口，提供了一个头文件。

## 自动判断方式说明
针对 Windows, Linux, macOS 使用了平台特定的方式尝试获得当前的网络连接方式，如果是无线网连接还会获取 SSID。
如果无法获取，则尝试连接特定的网址来判断。

|平台|方法|
|-|-|
|Windows|`Windows::Networking::Connectivity`|
|Linux|`libiw`|
|macOS|System Configuration 与 Core WLAN|
