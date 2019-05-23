# tunet-rust
清华大学校园网Rust库与客户端。
## 登录/注销
``` bash
# 使用默认（自动判断）方式登录
./tunet-rust login -u 用户名 -p 密码
# 使用默认（自动判断）方式注销，不需要用户名密码
./tunet-rust logout
# 使用auth4方式登录
./tunet-rust login -s auth4 -u 用户名 -p 密码
# 使用auth4方式注销，需要用户名密码
./tunet-rust logout -s auth4 -u 用户名 -p 密码
```
## 在线状态
``` bash
# 使用默认（自动判断）方式
./tunet-rust status
# 使用auth4方式
./tunet-rust status -s auth4
```
## 查询/强制下线在线IP
``` bash
# 查询
./tunet-rust online -u 用户名 -p 密码
# 强制下线
./tunet-rust drop -a IP地址 -u 用户名 -p 密码
```
## 流量明细
``` bash
# 使用默认排序（注销时间，升序）查询明细
./tunet-rust detail -u 用户名 -p 密码
# 使用登录时间（升序）查询明细
./tunet-rust detail -o login -u 用户名 -p 密码
# 使用流量降序查询明细
./tunet-rust detail -o flux -d -u 用户名 -p 密码
# 使用流量降序查询明细，并按注销日期组合
./tunet-rust detail -o flux -dg -u 用户名 -p 密码
```

