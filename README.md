# tunet-rust
清华大学校园网Rust库与客户端。

这一版本与[TsinghuaNet](https://github.com/Berrysoft/TsinghuaNet)中的TsinghuaNet.CLI功能相同。
## 登录/注销
``` bash
# 使用默认（自动判断）方式登录
./tunet login
# 使用默认（自动判断）方式注销
./tunet logout
# 使用auth4方式登录
./tunet login -s auth4
# 使用auth4方式注销
./tunet logout -s auth4
```
## 在线状态
``` bash
# 使用默认（自动判断）方式
./tunet status
# 使用auth4方式
./tunet status -s auth4
```
## 查询/强制下线在线IP
``` bash
# 查询
./tunet online
# 强制下线
./tunet drop -a IP地址
```
## 流量明细
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
