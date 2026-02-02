根据目录下的es-module-lexer-architecture.md和es-module-lexer-diagrams.md，完成如下设计：

> **目标**
>
> 基于 `es-module-lexer` 官方源码，实现一个 **行为、输出、测试完全对齐** 的 Rust 版本 `es-module-lexer-rs`，并通过 `napi-rs` 提供 Node.js API。
> 该项目不是重新设计 lexer，而是 **对 es-module-lexer 的忠实 Rust 复刻**，在保证功能一致的前提下，利用 Rust 的内存模型与并发能力实现 **显著性能优势**。

------

### 📌 设计与实现约束（必须遵守）

#### 1. 源码对齐原则（非常重要）

- 以 `es-module-lexer` **当前主分支源码** 为唯一行为基准
- 明确识别并复刻：
  - 核心状态机
  - 字符扫描逻辑
  - import/export 解析规则
  - 边界 case（dynamic import, import.meta, comments 等）
- **允许 Rust 层重构结构，但不允许改变算法语义**

------

#### 2. Rust 侧架构约束

- 禁止使用 parser combinator（如 `nom`）
- 使用 **手写状态机 + 单次线性扫描**
- 优先使用：
  - `&[u8]` / `&str` + index
  - `struct` + enum 状态
- 尽量减少 heap allocation：
  - 禁止不必要的 `String` clone
  - 能 slice 就 slice
- Rust API 设计需清晰区分：
  - lexer 核心逻辑
  - napi 绑定层
  - JS 测试适配层

------

#### 3. Napi-rs & JS 层

- 使用 `napi-rs` 作为唯一 JS/Rust 桥接方案
- JS API 设计 **对齐 es-module-lexer**
- JS 测试层：
  - 使用 `vitest`
  - 可直接复用或移植 es-module-lexer 的测试用例
  - 对比输出结构（imports / exports / spans）

------

#### 4. 性能目标（必须量化）

- 提供 benchmark：
  - 相同输入下，对比 `es-module-lexer` vs `es-module-lexer-rs`
  - 指标至少包括：
    - 吞吐时间
    - 内存分配次数（如可行）
- 目标：
  - 在中大型 JS 文件（100KB+）场景下，性能 **明显优于** 原始 JS 实现

------

#### 5. 工程与发布规范

- monorepo：
  - pnpm
  - TS + Rust
- TS 构建：
  - 使用 `tsdown`
- Rust 构建：
  - 多平台二进制
- npm 发布：
  - 按平台分发预编译产物
- README：
  - 风格、结构、示例 **对齐 es-module-lexer**
  - 清晰说明 Rust 实现优势

------

### ❗ 注意事项（严格遵守）

1. 不要生成无意义的 markdown 总结
2. 所有测试必须通过
3. 功能与 `es-module-lexer` 完整对齐
4. 性能不能只是“理论提升”，需要 benchmark 证明



> “实现过程中保留 `docs/architecture.md`，说明 JS 实现与 Rust 实现的核心差异与性能优化点。”