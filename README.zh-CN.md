# ES Module Lexer (Rust)

> ⚠️ **开发中** - 本项目正在积极开发中，请勿在生产环境使用。

[es-module-lexer](https://github.com/guybedford/es-module-lexer) 的 Rust 实现，通过 napi-rs 提供 Node.js 绑定。

快速的 JavaScript ES 模块词法分析器，输出导出列表和导入说明符的位置，包括动态导入和 import meta 处理。使用 Rust 构建，保证内存安全和可维护性。

[English](./README.md) | 简体中文

## 为什么选择 Rust？

本库提供了原始 es-module-lexer 的**内存安全、可维护的替代方案**：

- 🔒 **内存安全**：Rust 的所有权系统防止段错误和内存泄漏
- 📘 **类型安全**：完整的 TypeScript 定义，编译时保证
- 🛠️ **可维护性**：现代 Rust 代码库，配备 cargo、clippy、rustfmt
- 🎯 **API 兼容**：es-module-lexer 的直接替代
- 🧪 **充分测试**：单元测试、集成测试和基于属性的测试

## 功能特性

- ✅ 静态和动态 import/export 解析
- ✅ Import attributes（with 语法）
- ✅ Source phase imports
- ✅ Import meta 检测
- ✅ Facade 模块检测
- ✅ 跨平台预构建二进制文件
- ✅ 零运行时依赖

## 安装

```bash
npm install es-module-lexer-rs
```

预构建二进制文件支持 Linux、macOS、Windows 和 FreeBSD（x64、ARM64）。

## API

```typescript
interface ImportSpecifier {
  n?: string;      // 模块说明符
  t: number;       // import 类型（1=静态，2=动态，3=import.meta 等）
  s: number;       // 模块说明符开始位置
  e: number;       // 模块说明符结束位置
  ss: number;      // 语句开始位置
  se: number;      // 语句结束位置
  d: number;       // 动态 import 位置（如果不是动态则为 -1）
  a: number;       // assert/with 子句位置（如果没有则为 -1）
  at?: [string, string][] | null;  // 解析的 import attributes
}

interface ExportSpecifier {
  n: string;       // 导出名称
  ln?: string;     // 本地名称（用于重新导出）
  s: number;       // 导出名称开始位置
  e: number;       // 导出名称结束位置
  ls: number;      // 本地名称开始位置（如果没有则为 -1）
  le: number;      // 本地名称结束位置（如果没有则为 -1）
}

interface ParseResult {
  imports: ImportSpecifier[];
  exports: ExportSpecifier[];
  facade: boolean;          // 如果模块只包含 imports/exports 则为 true
  hasModuleSyntax: boolean; // 如果模块有任何 import/export 则为 true
}

function parse(source: string): ParseResult;
```

详细使用示例请参见[文档](./docs/)。

## 支持的语法

- 静态和动态 imports/exports
- Import attributes（`with` 语法）
- Source phase imports
- Import meta
- 字符串导出名称
- 重新导出
- 所有 ES 模块语法变体

## 文档

- [性能分析](./docs/performance-analysis.zh-CN.md)
- [架构](./docs/architecture.md)
- [贡献指南](./CONTRIBUTING.md)

## 许可证

MIT

## 致谢

受 Guy Bedford 的 [es-module-lexer](https://github.com/guybedford/es-module-lexer) 启发。

---

**使用 Rust 🦀 和 napi-rs 构建**
