# 实现计划：es-module-lexer-rs

## 概述

本实现计划将 es-module-lexer-rs 的设计转换为一系列可执行的编码任务。每个任务都建立在前面的任务之上，最终完成一个功能完整、性能优异的 Rust 实现。

## 任务列表

- [x] 1. 项目结构和基础设施搭建
  - 创建 monorepo 结构（pnpm workspace）
  - 设置 Rust crate（es-module-lexer）
  - 设置 napi-rs 绑定 crate
  - 配置 TypeScript 包结构
  - 设置测试框架（Rust: cargo test, JS: vitest）
  - 配置 CI/CD 基础设施
  - _需求：14.4, 14.5_

- [x] 2. 核心数据结构实现
  - [x] 2.1 定义 Import、Export、Attribute 结构体
    - 在 `types.rs` 中定义所有核心数据结构
    - 实现 ImportType 枚举（7 种类型）
    - 实现 OpenTokenState 枚举和 OpenToken 结构
    - 实现 ParseResult 结构
    - 添加必要的 derive 宏（Debug, Clone 等）
    - _需求：1.1, 1.2, 3.1, 4.1, 5.1_
  
  - [x] 2.2 为核心数据结构编写单元测试
    - 测试结构体创建和字段访问
    - 测试 ImportType 枚举的所有变体
    - _需求：1.1, 1.2_

- [x] 3. Lexer 基础框架
  - [x] 3.1 实现 Lexer 结构体和基本方法
    - 创建 `lexer.rs` 文件
    - 实现 Lexer::new() 构造函数
    - 实现基本的字符访问方法（peek, advance 等）
    - 实现位置跟踪逻辑
    - 初始化内部状态（栈、标志等）
    - _需求：1.1, 1.5_
  
  - [x] 3.2 编写 Lexer 基础功能的单元测试
    - 测试构造函数
    - 测试字符访问方法
    - 测试位置跟踪
    - _需求：1.1_

- [x] 4. 注释和空白处理
  - [x] 4.1 实现注释跳过功能
    - 在 `scanner/comment.rs` 中实现单行注释跳过
    - 实现多行注释跳过
    - 实现 comment_whitespace() 方法
    - 处理注释中的特殊字符
    - _需求：9.1, 9.2, 9.3_
  
  - [x] 4.2 编写属性测试：注释跳过完整性
    - **属性 11：注释跳过完整性**
    - **验证需求：9.1, 9.2, 9.3, 9.4**
    - 生成包含各种注释的代码
    - 验证注释内容不影响解析
    - _需求：9.1, 9.2, 9.3, 9.4_

- [x] 5. 字符串字面量解析
  - [x] 5.1 实现字符串扫描功能
    - 在 `scanner/string.rs` 中实现 string_literal() 方法
    - 处理单引号和双引号字符串
    - 实现转义字符处理（\n, \r, \t, \xHH, \uHHHH, \u{HHHHHH}）
    - 实现 read_string() 方法（提取字符串值）
    - 处理未闭合字符串错误
    - _需求：8.1, 8.2, 8.3_
  
  - [x] 5.2 编写属性测试：字符串转义 Round-Trip
    - **属性 6：字符串转义 Round-Trip**
    - **验证需求：5.2, 5.3, 8.3**
    - 生成包含各种转义字符的字符串
    - 验证解析后的值正确处理转义
    - _需求：5.2, 5.3, 8.3_
  
  - [x] 5.3 编写属性测试：字符串解析完整性
    - **属性 9：字符串解析完整性**
    - **验证需求：8.1, 8.2**
    - 生成各种引号字符串
    - 验证正确扫描到闭引号
    - _需求：8.1, 8.2_

- [x] 6. 检查点 - 基础扫描功能
  - 确保所有测试通过，如有问题请询问用户。

- [x] 7. 正则表达式处理
  - [x] 7.1 实现正则表达式扫描
    - 在 `scanner/regex.rs` 中实现 regular_expression() 方法
    - 处理正则表达式标志（g, i, m 等）
    - 处理正则表达式中的转义字符
    - 处理字符类 [...]
    - _需求：6.1, 6.2, 6.3, 6.4_
  
  - [x] 7.2 实现正则/除法歧义判断
    - 实现 is_expression_punctuator() 方法
    - 实现 is_paren_keyword() 方法（检测 while/for/if）
    - 实现 is_expression_terminator() 方法
    - 实现 is_expression_keyword() 方法
    - 实现 handle_slash() 方法（决策逻辑）
    - _需求：6.1, 6.2, 6.3, 6.4, 6.5_
  
  - [x] 7.3 编写属性测试：正则 vs 除法上下文判断
    - **属性 8：正则表达式 vs 除法运算符上下文判断**
    - **验证需求：6.1, 6.2, 6.3, 6.4, 6.5**
    - 生成各种上下文的 / 字符
    - 验证正确识别为正则或除法
    - _需求：6.1, 6.2, 6.3, 6.4, 6.5_

- [x] 8. 模板字符串处理
  - [x] 8.1 实现模板字符串扫描
    - 在 `scanner/string.rs` 中实现 template_string() 方法
    - 处理 ${} 表达式插值
    - 跟踪嵌套层级（使用 OpenToken 栈）
    - 处理嵌套的模板字符串
    - _需求：8.4, 8.5_
  
  - [x] 8.2 编写属性测试：模板字符串嵌套处理
    - **属性 10：模板字符串嵌套处理**
    - **验证需求：8.4, 8.5**
    - 生成包含嵌套插值的模板字符串
    - 验证正确跟踪嵌套层级
    - _需求：8.4, 8.5_
  
  - [x] 8.3 编写属性测试：模板字符串括号匹配
    - **属性 13：模板字符串括号匹配**
    - **验证需求：7.6**
    - 生成包含复杂嵌套的模板字符串
    - 验证 ${} 正确匹配
    - _需求：7.6_

- [x] 9. Import 语句解析
  - [x] 9.1 实现静态 import 解析
    - 在 `parser/import.rs` 中实现 try_parse_import_statement()
    - 处理 import ... from "module" 语法
    - 处理命名 import { x, y as z }
    - 处理默认 import
    - 处理命名空间 import * as ns
    - 提取模块说明符和位置信息
    - _需求：3.1_
  
  - [x] 9.2 实现动态 import 解析
    - 处理 import(...) 表达式
    - 跟踪括号匹配（使用 OpenToken 栈）
    - 区分字符串字面量和表达式参数
    - 设置 safe 标志
    - 检测 import attributes（逗号后）
    - _需求：3.2, 3.3, 3.4_
  
  - [x] 9.3 实现 import.meta 解析
    - 检测 import.meta 语法
    - 设置 ImportType::ImportMeta
    - 记录位置信息
    - _需求：3.5_
  
  - [x] 9.4 实现 source phase 和 defer phase import 解析
    - 检测 import source 语法
    - 检测 import defer 语法
    - 检测 import.source() 和 import.defer()
    - 设置正确的 ImportType
    - _需求：3.7, 3.8_
  
  - [x] 9.5 编写属性测试：Import 类型标记正确性
    - **属性 2：Import 类型标记正确性**
    - **验证需求：3.1, 3.2, 3.5, 3.7, 3.8**
    - 生成各种类型的 import 语句
    - 验证类型标记正确
    - _需求：3.1, 3.2, 3.5, 3.7, 3.8_
  
  - [x] 9.6 编写属性测试：动态 Import 安全性标记
    - **属性 3：动态 Import 安全性标记**
    - **验证需求：3.3, 3.4**
    - 生成字符串字面量和表达式动态 import
    - 验证 safe 标志正确
    - _需求：3.3, 3.4_
  
  - [x] 9.7 编写属性测试：动态 Import 括号匹配
    - **属性 12：动态 Import 括号匹配**
    - **验证需求：7.5**
    - 生成包含嵌套括号的动态 import
    - 验证 statement_end 位置正确
    - _需求：7.5_

- [x] 10. Import Attributes 解析
  - [x] 10.1 实现 import attributes 解析
    - 在 `parser/attributes.rs` 中实现 parse_import_attributes()
    - 检测 with 关键字
    - 解析 { key: "value" } 语法
    - 处理多个键值对
    - 处理字符串键和值的转义
    - 创建 Attribute 结构并添加到 Import
    - _需求：5.1, 5.2, 5.3_
  
  - [x] 10.2 编写属性测试：Import Attributes 解析完整性
    - **属性 5：Import Attributes 解析完整性**
    - **验证需求：5.1, 5.4**
    - 生成包含各种 attributes 的 import
    - 验证所有键值对被解析
    - 验证顺序保持
    - _需求：5.1, 5.4_

- [x] 11. 检查点 - Import 解析完成
  - 确保所有 import 相关测试通过，如有问题请询问用户。

- [x] 12. Export 语句解析
  - [x] 12.1 实现命名导出解析
    - 在 `parser/export.rs` 中实现 try_parse_export_statement()
    - 处理 export { a, b as c } 语法
    - 实现 parse_export_list() 方法
    - 提取导出名称和本地名称
    - 处理字符串导出名称
    - _需求：4.1, 4.5_
  
  - [x] 12.2 实现默认导出解析
    - 检测 export default 语法
    - 处理 export default function/class
    - 处理 export default 表达式
    - 设置导出名称为 "default"
    - _需求：4.2_
  
  - [x] 12.3 实现重导出解析
    - 处理 export { a } from "module"
    - 处理 export * from "module"
    - 处理 export * as ns from "module"
    - 提取源模块信息
    - _需求：4.3, 4.4_
  
  - [x] 12.4 实现声明导出解析
    - 处理 export var/let/const 声明
    - 处理 export function 声明
    - 处理 export class 声明
    - 处理 export async function 声明
    - 处理解构声明（export const { a, b } = ...）
    - 提取所有声明的标识符
    - _需求：4.6, 4.7_
  
  - [x] 12.5 编写属性测试：Export 提取完整性
    - **属性 4：Export 提取完整性**
    - **验证需求：4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7**
    - 生成各种类型的 export 语句
    - 验证导出名称和本地名称正确提取
    - _需求：4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7_

- [x] 13. 两阶段解析实现
  - [x] 13.1 实现 Facade 模式解析
    - 实现 parse_facade() 方法
    - 只处理 import/export/注释/空白
    - 遇到其他语法时设置 facade = false 并返回
    - _需求：2.1, 2.2_
  
  - [x] 13.2 实现完整解析模式
    - 实现 parse_full() 方法
    - 处理所有 JavaScript 语法结构
    - 跟踪括号和大括号匹配
    - 处理所有 token 类型
    - _需求：2.2_
  
  - [x] 13.3 实现主 parse() 方法
    - 协调两阶段解析
    - 返回 ParseResult
    - _需求：1.1, 1.2_
  
  - [x] 13.4 编写属性测试：Facade 模式检测
    - **属性 7：Facade 模式检测**
    - **验证需求：2.1, 2.2**
    - 生成纯模块文件和混合文件
    - 验证 facade 标志正确
    - _需求：2.1, 2.2_
  
  - [x] 13.5 编写属性测试：解析完整性
    - **属性 1：解析完整性**
    - **验证需求：1.1, 1.2**
    - 生成包含多个 import/export 的代码
    - 验证所有语句都被解析
    - 验证位置信息准确
    - _需求：1.1, 1.2_

- [x] 14. 错误处理实现
  - [x] 14.1 定义错误类型
    - 在 `error.rs` 中定义 LexerError 枚举
    - 实现所有错误变体
    - 实现 Display 和 Error trait
    - _需求：10.1, 10.4_
  
  - [x] 14.2 添加错误处理逻辑
    - 在各个解析方法中添加错误检查
    - 返回适当的错误类型
    - 实现错误恢复（如果可能）
    - _需求：10.1, 10.2, 10.3, 10.4_
  
  - [x] 14.3 编写错误处理测试
    - 测试各种语法错误
    - 测试未闭合字符串
    - 测试错误恢复
    - _需求：10.1, 10.2, 10.3, 10.4_

- [x] 15. 检查点 - Rust 核心完成
  - 确保所有 Rust 核心测试通过，如有问题请询问用户。

- [x] 16. Napi 绑定实现
  - [x] 16.1 设置 napi-rs 项目
    - 在 `packages/es-module-lexer-rs/native/` 中创建 napi 项目
    - 配置 Cargo.toml 依赖
    - 设置构建脚本
    - _需求：11.1, 14.1, 14.2, 14.3_
  
  - [x] 16.2 实现数据结构转换
    - 定义 JsImport、JsExport、JsParseResult 结构
    - 实现 Rust → JavaScript 转换函数
    - 处理 UTF-8 → UTF-16 位置索引转换
    - 处理 Option 和 Vec 转换
    - _需求：11.2, 11.3, 11.4_
  
  - [x] 16.3 实现 parse() napi 函数
    - 使用 #[napi] 宏导出函数
    - 调用 Rust lexer
    - 转换结果为 JavaScript 对象
    - 处理错误并转换为 JavaScript Error
    - _需求：11.1, 11.2, 11.5_
  
  - [x] 16.4 编写属性测试：UTF-16 位置索引转换
    - **属性 14：UTF-16 位置索引转换**
    - **验证需求：11.4**
    - 生成包含多字节 Unicode 字符的代码
    - 验证位置索引转换正确
    - _需求：11.4_

- [x] 17. TypeScript 包装和类型定义
  - [x] 17.1 创建 TypeScript API
    - 在 `packages/es-module-lexer-rs/src/` 中创建 index.ts
    - 导出 parse 函数
    - 添加类型定义
    - _需求：11.6_
  
  - [x] 17.2 创建类型定义文件
    - 定义 ImportSpecifier 接口
    - 定义 ExportSpecifier 接口
    - 定义 ImportType 枚举
    - 确保与 es-module-lexer 类型兼容
    - _需求：11.6_

- [x] 18. JavaScript 测试套件
  - [x] 18.1 移植 es-module-lexer 测试用例
    - 在 `packages/es-module-lexer-rs/tests/` 中创建测试文件
    - 移植所有单元测试
    - 使用 vitest 框架
    - _需求：13.1, 13.3_
  
  - [x] 18.2 实现对比测试
    - 创建 integration.test.ts
    - 对比 Rust 和原始 JS 实现的输出
    - 使用真实库代码（react, d3.js 等）
    - _需求：13.2, 13.7_
  
  - [x] 18.3 编写属性测试：输出对齐
    - **属性 15：输出对齐（与原始实现）**
    - **验证需求：13.2, 13.7**
    - 对于相同输入，验证 Rust 和 JS 输出一致
    - _需求：13.2, 13.7_

- [x] 19. 性能优化
  - [x] 19.1 实现零拷贝优化
    - 使用 &[u8] 和 &str 切片
    - 避免不必要的 String 分配
    - 使用 SmallVec 优化小集合
    - _需求：12.2, 12.3, 12.4_
  
  - [x] 19.2 实现内联优化
    - 为小函数添加 #[inline] 属性
    - 优化热路径代码
    - _需求：12.2_
  
  - [x] 19.3 实现预分配优化
    - 预估容器大小
    - 使用 with_capacity 创建 Vec
    - _需求：12.2_

- [x] 20. 性能基准测试
  - [x] 20.1 创建 Rust benchmark
    - 使用 criterion 创建 benchmark
    - 测试各种大小的文件
    - 测量吞吐量和内存使用
    - _需求：12.1, 12.6, 13.4_
  
  - [x] 20.2 创建 JavaScript benchmark
    - 在 `packages/es-module-lexer-rs/bench/` 中创建 benchmark
    - 对比 Rust 和原始实现
    - 使用 vitest bench
    - _需求：12.1, 13.4_
  
  - [x] 20.3 验证性能目标
    - 确保 Rust 版本比 Wasm 版本快 ≥20%
    - 确保内存使用减少 ≥20%
    - 记录性能数据到文档
    - _需求：12.1, 12.6_

- [x] 21. 检查点 - 功能和性能验证
  - 确保所有测试通过，性能目标达成，如有问题请询问用户。

- [x] 22. 多平台构建
  - [x] 22.1 配置多平台构建
    - 设置 GitHub Actions workflow
    - 配置 Linux (x64, arm64) 构建
    - 配置 macOS (x64, arm64) 构建
    - 配置 Windows (x64) 构建
    - _需求：14.1, 14.2, 14.3_
  
  - [x] 22.2 配置 npm 包发布
    - 设置 package.json 脚本
    - 配置平台特定的二进制文件
    - 设置自动平台选择逻辑
    - _需求：14.6, 14.7_

- [ ] 23. 文档编写
  - [x] 23.1 编写 README
    - 说明项目目标和优势
    - 提供安装和使用示例
    - 包含 API 文档
    - 包含性能对比数据
    - 对齐 es-module-lexer 文档风格
    - _需求：15.1, 15.2, 15.3, 15.4, 15.5_
  
  - [x] 23.2 编写架构文档
    - 创建 docs/architecture.md
    - 说明 Rust 实现与 JS 实现的差异
    - 说明性能优化技术
    - 提供设计决策说明
    - _需求：15.6, 15.7_
  
  - [x] 23.3 编写贡献指南
    - 创建 CONTRIBUTING.md
    - 说明开发环境设置
    - 说明测试和 benchmark 运行方法
    - 说明代码风格和最佳实践
    - _需求：17.3, 17.4, 17.5_

- [ ] 24. 最终验证和发布准备
  - [ ] 24.1 运行完整测试套件
    - 运行所有 Rust 测试
    - 运行所有 JavaScript 测试
    - 运行所有属性测试
    - 运行所有 benchmark
    - _需求：13.1, 13.2, 13.5, 13.6_
  
  - [ ] 24.2 代码质量检查
    - 运行 clippy 并修复所有警告
    - 运行 rustfmt 格式化代码
    - 检查 unsafe 代码使用
    - 审查文档注释
    - _需求：17.3, 17.4, 17.7_
  
  - [ ] 24.3 准备发布
    - 更新版本号
    - 更新 CHANGELOG
    - 创建 git tag
    - 准备 npm 发布
    - _需求：14.6_

## 注意事项

- 所有任务都是必需的，确保从一开始就进行全面测试
- 每个任务都引用了相关的需求编号，确保可追溯性
- 检查点任务用于验证阶段性成果，确保增量验证
- 属性测试任务明确标注了对应的设计文档属性编号和验证的需求
- 每一步要保证所有的历史的单元测试/逻辑的正确性，杜绝完成后面task的时候把前面的task的功能破坏
- Github UserName: Sunny-117 ; mail: zhiqiangfu6@gmail.com
---

**文档版本**: 1.0  
**创建日期**: 2025-01-27  
**作者**: Kiro AI Assistant
