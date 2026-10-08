# JSON 输入通道合同

## 目的与范围（Q → owner P）

依据用户裁决：增加 JSON 输入，不替换原 CLI；AI 可用结构化请求操作同一 WorkingStack，每次修改记录理由。不增加尚未触发的防御机制。本次修正只承接已复现的字段丢失、错误回退、原因丢失以及 CLI/Web 转换漂移。

## 设计（D）

- 进程入口读取完整 stdin；终端或空 stdin 使用原 argv；非空 stdin 必须完整解析为一个请求，失败直接报错，不能回退执行 argv。库 `cli::run` 不读取进程 stdin，Web 调用它不会再等待 stdin。
- 请求 `{stack?, op, args?, reason?, json?}` 转换成 CLI 参数，由既有 dispatch 完成状态变换、validate、save。Web 使用同一转换器，并将 stack 绑定为该服务的目标。
- 字符串列表字段接受单字符串或字符串数组；不丢弃数组元素。字段缺席表示不修改；空 title/body 原样传入；update 的 goal/origin 空数组表示清空。追加字段空数组表示不追加。
- 未消费的参数、已提供但类型不匹配的字段报错；依据是已复现的拼错字段仍返回成功及非数组 goal 被静默丢弃，不是新增通用安全框架。
- 写操作 reason 必填；note 的 CLI 增加可选 `-r`，保留旧 CLI 不传理由时的默认值。JSON note 写必须提供 reason，读 note 不落盘。
- Web 成功响应仍为状态快照；CLI 成功响应仍为 `{ok,diff,version,warnings,text}`。两端共用请求转换，不宣称响应格式相同。CLI 错误仍使用 stderr 和非零退出码。

## 函数合同与推理（叶子 D → I）

`Request::cli_args`：成功后所有 args key 已被该 op 消费；每个参数的文本值保持原样，空值规则如上。位置字段按 CLI 文法顺序输出；flags 使用 `--flag=value` 保留以 `-` 开头的内容；循环每步移除一个字段，终止后有剩余则错误。转换全程不访问栈，因此转换失败无状态副作用。

`process_args`：stdin 空或为终端 ⇒ argv 不变；stdin 非空 ⇒ 完整 JSON 转换或错误。只有成功返回的 argv 才传给 dispatch，因此解析失败不执行原 argv。

`cli::run`：复用既有变换与保存链；note 读取分支提前返回，note 写将调用者原因传入 `note_put`；origin 清空与 goal 清空相同。

`web::handle_op`：共享请求转换 → CLI dispatch → 读取实际解析后的 file。读取失败不标成功，修正此前托管名被当路径造成的错误状态。

## SCCO 与证据边界

- Sound：用户请求语义由上述合同落实，不用下层“exit 0”反证输入已保留。
- Complete：转换覆盖现有 CLI 动词及其字段；功能测试检查存储值、日志、失败无修改以及 CLI 兼容。
- Concise：单一转换器取代 CLI/Web 两份映射，不增加重试、自动修复或后备执行机制。
- Optimality：保留既有 CLI dispatch，避免另造一套 WorkingStack 操作实现。

测试：`python3 tests/json-channel.py target/debug/working-stack`。这是直接功能测试，不是机械证明。WS-MAIN 的 Markdown 保真是独立的 import/print 合同；JSON 修正不能据此声称其也已成立。
