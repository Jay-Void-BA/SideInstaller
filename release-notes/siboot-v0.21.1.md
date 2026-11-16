# siboot 0.21.1 — rootshell 性能诊断日志

维护：**Jay-Void-BA**。原作：**FrizzleM/siboot** 与 **FrizzleM/SideInstaller**。

新增可选参数 `--perf-log`：

```sh
./siboot.wasm --perf-log --ipa App.ipa
./siboot.wasm renew --perf-log
```

每次运行在 `siboot.wasm` 同目录创建独立的 `siboot-perf-*.jsonl`，包含步骤／重试、用户输入等待、HTTP／WebSocket／TLS、解压、密钥生成、完整签名与哈希、逐文件上传核对、iOS 安装进度及最终统计摘要。网络收发记录次数、字节数、累计／最长耗时和延迟分桶，区分短暂轮询超时与实际错误。

日志仅记录操作与数字，不保存密码、账号、验证码、令牌、密钥、文件名、URL、请求或响应正文。默认关闭；详细事件达到 16 MiB 后继续统计并在正常结束时写摘要。日志写入异常会明确提示。

计时为墙钟时间，WASI 时钟通常约 1 ms 精度，嵌套事件会重叠；开启日志会增加采集和写入开销。强制中断可能缺少最后的缓冲记录和摘要。WASM 加载时间不在计时范围内，内存字段指线性内存容量。

保留 0.21.0 的完整 rootshell 安装／续签功能与双方作者署名，仍需要 LocalDevVPN。CI 增加日志上限、落盘、错误／取消摘要、计数器及敏感数据过滤验证；真实 Apple 账号至 iPhone 的端到端安装仍需真机验证。
