# 需求文档：es-module-lexer-rs

## 简介

es-module-lexer-rs 是 es-module-lexer 的 Rust 实现版本，旨在提供与原始 JavaScript/WebAssembly 实现完全对齐的功能，同时利用 Rust 的内存安全性和性能优势。本项目通过 napi-rs 提供 Node.js API，确保与原始实现的 API 兼容性。

## 术语表

- **Lexer（词法分析器）**：扫描源代码并识别 import/export 语句的组件
- **Import_Specifier（导入说明符）**：描述一个 import 语句的数据结构
- **Export_Specifier（导出说明符）**：描述一个 export 语句的数据结构
- **Facade_Mode（门面模式）**：快速解析纯模块文件的优化模式
- **Dynamic_Import（动态导入）**：使用 import() 函数的运行时导入
- **Import_Attributes（导入属性）**：使用 with 语法的导入元数据
- **Source_Phase（源阶段）**：使用 import source 语法的导入
- **Defer_Phase（延迟阶段）**：使用 import defer 语法的导入
- **State_Machine（状态机）**：基于字符扫描的解析引擎
- **Napi_Binding（Napi 绑定）**：通过 napi-rs 提供的 Node.js 接口

## 需求

### 需求 1：核心词法分析功能

**用户故事**：作为开发者，我想要快速解析 JavaScript 模块的 import/export 语句，以便进行依赖分析和模块加载。

#### 验收标准

1. WHEN 提供有效的 JavaScript 源代码 THEN Lexer SHALL 解析所有静态 import 语句并返回其位置和模块说明符
2. WHEN 提供有效的 JavaScript 源代码 THEN Lexer SHALL 解析所有 export 语句并返回导出名称和位置
3. WHEN 源代码包含动态 import() 表达式 THEN Lexer SHALL 识别并标记这些表达式
4. WHEN 源代码包含 import.meta 引用 THEN Lexer SHALL 识别并记录这些引用
5. THE Lexer SHALL 使用单次线性扫描完成解析，时间复杂度为 O(n)
6. THE Lexer SHALL 使用手写状态机而非 parser combinator 实现

### 需求 2：两阶段解析策略

**用户故事**：作为开发者，我想要对纯模块文件进行快速解析，以便提高常见场景的性能。

#### 验收标准

1. WHEN 源代码仅包含 import/export 语句 THEN Lexer SHALL 使用 Facade 模式进行快速解析
2. WHEN 在 Facade 模式下遇到非模块语法 THEN Lexer SHALL 自动切换到完整解析模式
3. WHEN 使用完整解析模式 THEN Lexer SHALL 正确处理所有 JavaScript 语法结构
4. THE Lexer SHALL 在 Facade 模式下跳过不必要的语法分析以提高性能

### 需求 3：Import 语句解析

**用户故事**：作为开发者，我想要准确解析各种形式的 import 语句，以便完整理解模块依赖关系。

#### 验收标准

1. WHEN 遇到静态 import 语句 THEN Lexer SHALL 提取模块说明符、语句范围和导入类型
2. WHEN 遇到动态 import() 表达式 THEN Lexer SHALL 提取表达式范围并标记为动态导入
3. WHEN 动态 import 使用字符串字面量 THEN Lexer SHALL 标记为安全导入并提取模块名称
4. WHEN 动态 import 使用表达式 THEN Lexer SHALL 标记为不安全导入
5. WHEN 遇到 import.meta 引用 THEN Lexer SHALL 记录其位置并标记类型为 ImportMeta
6. WHEN import 语句包含 with 子句 THEN Lexer SHALL 解析 import attributes
7. WHEN 遇到 import source 语法 THEN Lexer SHALL 标记类型为 StaticSourcePhase 或 DynamicSourcePhase
8. WHEN 遇到 import defer 语法 THEN Lexer SHALL 标记类型为 StaticDeferPhase 或 DynamicDeferPhase

### 需求 4：Export 语句解析

**用户故事**：作为开发者，我想要准确解析各种形式的 export 语句，以便了解模块的公共接口。

#### 验收标准

1. WHEN 遇到命名导出 THEN Lexer SHALL 提取导出名称和本地名称
2. WHEN 遇到默认导出 THEN Lexer SHALL 记录导出名称为 "default"
3. WHEN 遇到重导出语句 THEN Lexer SHALL 提取源模块和导出名称
4. WHEN 遇到 export * from 语句 THEN Lexer SHALL 记录通配符导出和源模块
5. WHEN 遇到 export { a as b } 语法 THEN Lexer SHALL 正确映射本地名称和导出名称
6. WHEN 遇到 export var/let/const/function/class 声明 THEN Lexer SHALL 提取声明的标识符
7. WHEN 遇到解构导出 THEN Lexer SHALL 提取所有解构的标识符

### 需求 5：Import Attributes 解析

**用户故事**：作为开发者，我想要解析 import attributes（with 语法），以便支持 JSON 模块和其他类型的导入。

#### 验收标准

1. WHEN import 语句包含 with { key: "value" } 子句 THEN Lexer SHALL 解析所有键值对
2. WHEN attribute 键是字符串字面量 THEN Lexer SHALL 正确解码转义字符
3. WHEN attribute 值是字符串字面量 THEN Lexer SHALL 正确解码转义字符
4. WHEN attributes 包含多个键值对 THEN Lexer SHALL 保持解析顺序
5. WHEN attribute 语法不正确 THEN Lexer SHALL 忽略该 attribute 并继续解析
6. THE Lexer SHALL 将 attributes 存储为键值对数组

### 需求 6：正则表达式与除法运算符歧义处理

**用户故事**：作为开发者，我想要 lexer 正确区分正则表达式和除法运算符，以便避免解析错误。

#### 验收标准

1. WHEN '/' 前面是表达式标点符号 THEN Lexer SHALL 将其解析为正则表达式
2. WHEN '/' 前面是 ')' 且对应 while/for/if 关键字 THEN Lexer SHALL 将其解析为正则表达式
3. WHEN '/' 前面是 '}' 且是表达式终结符 THEN Lexer SHALL 将其解析为正则表达式
4. WHEN '/' 前面是表达式关键字（return/throw/typeof 等）THEN Lexer SHALL 将其解析为正则表达式
5. WHEN '/' 前面是标识符或数字 THEN Lexer SHALL 将其解析为除法运算符
6. THE Lexer SHALL 使用回溯分析确定 '/' 的语义

### 需求 7：括号和大括号匹配

**用户故事**：作为开发者，我想要 lexer 正确跟踪嵌套的括号和大括号，以便准确解析复杂表达式。

#### 验收标准

1. WHEN 遇到开括号 '(' THEN Lexer SHALL 将其压入匹配栈
2. WHEN 遇到闭括号 ')' THEN Lexer SHALL 从匹配栈弹出对应的开括号
3. WHEN 遇到开大括号 '{' THEN Lexer SHALL 将其压入匹配栈并记录上下文
4. WHEN 遇到闭大括号 '}' THEN Lexer SHALL 从匹配栈弹出对应的开大括号
5. WHEN 动态 import 的闭括号匹配 THEN Lexer SHALL 完成该 import 的解析
6. WHEN 模板字符串的 ${} 匹配 THEN Lexer SHALL 正确处理嵌套表达式
7. THE Lexer SHALL 使用固定大小栈（1024 深度）跟踪嵌套结构

### 需求 8：字符串和模板字符串处理

**用户故事**：作为开发者，我想要 lexer 正确处理字符串字面量和模板字符串，以便准确提取模块说明符。

#### 验收标准

1. WHEN 遇到单引号字符串 THEN Lexer SHALL 扫描到匹配的闭引号
2. WHEN 遇到双引号字符串 THEN Lexer SHALL 扫描到匹配的闭引号
3. WHEN 字符串包含转义字符 THEN Lexer SHALL 正确处理转义序列
4. WHEN 遇到模板字符串 THEN Lexer SHALL 正确处理 ${} 表达式插值
5. WHEN 模板字符串嵌套 THEN Lexer SHALL 正确跟踪嵌套层级
6. THE Lexer SHALL 使用 Acorn 兼容的转义字符处理逻辑

### 需求 9：注释处理

**用户故事**：作为开发者，我想要 lexer 正确跳过注释，以便不影响 import/export 解析。

#### 验收标准

1. WHEN 遇到单行注释 // THEN Lexer SHALL 跳过到行尾
2. WHEN 遇到多行注释 /* */ THEN Lexer SHALL 跳过到注释结束
3. WHEN 注释出现在 import/export 语句中 THEN Lexer SHALL 正确处理空白和注释
4. WHEN 注释包含看似代码的内容 THEN Lexer SHALL 不解析注释内容
5. THE Lexer SHALL 在需要时保留注释前后的空白语义

### 需求 10：错误处理和容错

**用户故事**：作为开发者，我想要 lexer 在遇到语法错误时提供清晰的错误信息，以便快速定位问题。

#### 验收标准

1. WHEN 源代码包含语法错误 THEN Lexer SHALL 返回错误并指示错误位置
2. WHEN import/export 语句不完整 THEN Lexer SHALL 尝试恢复并继续解析
3. WHEN 遇到不支持的语法 THEN Lexer SHALL 跳过该部分并继续解析
4. WHEN 字符串未闭合 THEN Lexer SHALL 返回错误
5. THE Lexer SHALL 提供有意义的错误消息以帮助调试

### 需求 11：Node.js API 集成（napi-rs）

**用户故事**：作为 Node.js 开发者，我想要通过熟悉的 JavaScript API 使用 Rust lexer，以便无缝替换原始实现。

#### 验收标准

1. THE Napi_Binding SHALL 提供 parse(source: string, name?: string) 函数
2. WHEN 调用 parse 函数 THEN Napi_Binding SHALL 返回 [imports, exports, facade, hasModuleSyntax] 元组
3. THE Napi_Binding SHALL 将 Rust 数据结构转换为 JavaScript 对象
4. THE Napi_Binding SHALL 处理 UTF-16 字符串编码转换
5. WHEN Rust 代码发生错误 THEN Napi_Binding SHALL 抛出 JavaScript Error
6. THE Napi_Binding SHALL 提供与 es-module-lexer 完全兼容的类型定义

### 需求 12：性能优化

**用户故事**：作为开发者，我想要 Rust 实现比原始 JavaScript/WebAssembly 实现更快，以便提高构建工具性能。

#### 验收标准

1. WHEN 解析中大型文件（100KB+）THEN Rust_Lexer SHALL 比 WebAssembly 版本快至少 20%
2. THE Rust_Lexer SHALL 最小化堆内存分配
3. THE Rust_Lexer SHALL 优先使用 &[u8] 和 &str 切片而非 String
4. THE Rust_Lexer SHALL 使用零拷贝技术处理字符串
5. THE Rust_Lexer SHALL 避免不必要的 UTF-8 到 UTF-16 转换
6. WHEN 解析相同文件 THEN Rust_Lexer SHALL 使用更少的内存

### 需求 13：测试对齐

**用户故事**：作为开发者，我想要 Rust 实现通过所有原始测试用例，以便确保功能完全对齐。

#### 验收标准

1. THE Test_Suite SHALL 包含所有 es-module-lexer 的测试用例
2. WHEN 运行测试 THEN Rust_Lexer SHALL 产生与原始实现相同的输出
3. THE Test_Suite SHALL 使用 vitest 作为测试框架
4. THE Test_Suite SHALL 包含性能基准测试
5. THE Test_Suite SHALL 测试所有 ImportType 变体
6. THE Test_Suite SHALL 测试边缘情况和错误条件
7. WHEN 对比输出 THEN imports/exports 数组结构 SHALL 完全匹配

### 需求 14：多平台构建和发布

**用户故事**：作为包维护者，我想要为多个平台构建预编译二进制文件，以便用户无需编译即可使用。

#### 验收标准

1. THE Build_System SHALL 为 Linux (x64, arm64) 构建二进制文件
2. THE Build_System SHALL 为 macOS (x64, arm64) 构建二进制文件
3. THE Build_System SHALL 为 Windows (x64) 构建二进制文件
4. THE Build_System SHALL 使用 pnpm 管理 monorepo
5. THE Build_System SHALL 使用 tsdown 构建 TypeScript
6. WHEN 发布到 npm THEN Package SHALL 包含所有平台的预编译二进制文件
7. THE Package SHALL 在安装时自动选择正确的平台二进制文件

### 需求 15：文档和示例

**用户故事**：作为用户，我想要清晰的文档和示例，以便快速上手使用。

#### 验收标准

1. THE README SHALL 说明项目目标和 Rust 实现的优势
2. THE README SHALL 提供安装和使用示例
3. THE README SHALL 包含 API 文档
4. THE README SHALL 包含性能对比数据
5. THE README SHALL 对齐 es-module-lexer 的文档风格
6. THE Documentation SHALL 包含架构文档说明 Rust 实现与 JS 实现的差异
7. THE Documentation SHALL 提供性能优化技术的说明

### 需求 16：源码对齐原则

**用户故事**：作为维护者，我想要确保 Rust 实现忠实复刻原始行为，以便保证兼容性。

#### 验收标准

1. THE Rust_Implementation SHALL 以 es-module-lexer 主分支源码为唯一行为基准
2. THE Rust_Implementation SHALL 复刻核心状态机逻辑
3. THE Rust_Implementation SHALL 复刻字符扫描逻辑
4. THE Rust_Implementation SHALL 复刻 import/export 解析规则
5. THE Rust_Implementation SHALL 处理所有边界情况（dynamic import, import.meta, comments 等）
6. THE Rust_Implementation SHALL 允许结构重构但不改变算法语义
7. WHEN 原始实现更新 THEN Rust_Implementation SHALL 同步更新行为

### 需求 17：内存安全和 Rust 最佳实践

**用户故事**：作为 Rust 开发者，我想要代码遵循 Rust 最佳实践，以便保证内存安全和可维护性。

#### 验收标准

1. THE Rust_Code SHALL 不使用 unsafe 代码（除非绝对必要且有充分文档说明）
2. THE Rust_Code SHALL 使用 Rust 标准库的安全抽象
3. THE Rust_Code SHALL 通过 clippy 检查且无警告
4. THE Rust_Code SHALL 通过 rustfmt 格式化
5. THE Rust_Code SHALL 使用明确的生命周期标注
6. THE Rust_Code SHALL 使用 Result 类型处理错误
7. THE Rust_Code SHALL 提供清晰的文档注释

---

**文档版本**: 1.0  
**创建日期**: 2025-01-27  
**作者**: Kiro AI Assistant
