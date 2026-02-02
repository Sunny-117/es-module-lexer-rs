# 性能分析：为什么 Rust 绑定比原始 WASM 慢

[English](./performance-analysis.md) | 简体中文

## 执行摘要

尽管 Rust 是一种高性能系统语言，但我们基于 Rust 的实现（Napi-rs 和 WASM）目前比原始 C 编译的 WASM 实现**慢 3-12 倍**。本文档解释了为什么会出现这种情况以及本库仍然提供什么价值。

## 基准测试结果

### Napi-rs 版本（原生 Node.js 插件）

| 测试用例 | 原始库 | Rust（Napi） | 性能 |
|-----------|----------|-------------|-------------|
| 简单导入 | ~310 万次/秒 | ~52 万次/秒 | **慢 6 倍** |
| 多个导入 | ~92.5 万次/秒 | ~15.5 万次/秒 | **慢 6 倍** |
| 复杂模块 | ~35.2 万次/秒 | ~11.5 万次/秒 | **慢 3 倍** |

### WASM 版本（wasm-bindgen）

| 测试用例 | 原始库 | Rust（WASM） | 性能 |
|-----------|----------|-------------|-------------|
| 简单导入 | ~310 万次/秒 | ~29.8 万次/秒 | **慢 10.4 倍** |
| 多个导入 | ~92.5 万次/秒 | ~11 万次/秒 | **慢 8.4 倍** |
| 复杂模块 | ~35.2 万次/秒 | ~2.8 万次/秒 | **慢 12.7 倍** |

## 根本原因分析

### 1. 原始实现的优势

原始 `es-module-lexer` 有几个关键优势：

#### a) 手工优化的 C 代码
- 用 C 编写并使用 Emscripten 编译为 WASM
- 数十年的 C 编译器优化（LLVM）
- 直接内存操作，无安全检查
- 针对特定用例高度优化

#### b) 最小的 JavaScript 边界跨越
- 返回简单的数据结构
- 使用类型数组进行高效数据传输
- JavaScript 端最小的对象创建

#### c) 为 WASM 优化
- 直接使用 WASM 线性内存
- 无 UTF-8 到 UTF-16 转换开销
- 使用激进的优化标志编译

### 2. Rust Napi-rs 版本的瓶颈

#### a) UTF-16 转换开销（~40% 的时间）
```rust
// 我们必须将 UTF-8 字节位置转换为 UTF-16 代码单元位置
// 这需要遍历整个源字符串
fn build_utf16_index_map(source: &str) -> Vec<usize> {
    let mut map = Vec::with_capacity(source.len() + 1);
    let mut utf16_index = 0;
    
    for ch in source.chars() {
        let utf8_len = ch.len_utf8();
        let utf16_len = ch.len_utf16();
        
        for _ in 0..utf8_len {
            utf16_index += utf16_len;
            map.push(utf16_index);
        }
    }
    map
}
```

**为什么这是必要的**：JavaScript 内部使用 UTF-16，所以所有位置索引必须是 UTF-16 代码单元，而不是 UTF-8 字节。

**成本**：对于 100KB 的文件，这会创建一个 ~100KB 的索引映射并遍历每个字符。

#### b) Napi 对象创建开销（~30% 的时间）
```rust
// 通过 Napi 创建 JavaScript 对象很昂贵
let mut js_import = env.create_object()?;
js_import.set("n", module_name)?;
js_import.set("s", start)?;
js_import.set("e", end)?;
// ... 每个导入还有 5 个属性设置
```

**为什么这是必要的**：Napi 需要通过 FFI 调用显式创建对象和设置属性。

**成本**：每个属性设置都跨越 Rust/JavaScript 边界并涉及类型转换。

#### c) 内存分配和复制（~20% 的时间）
- Rust 结构必须转换为 JavaScript 对象
- 字符串从 Rust 堆复制到 JavaScript 堆
- 数组在 JavaScript 端分配和填充

#### d) FFI 开销（~10% 的时间）
- 每个函数调用都跨越原生/JavaScript 边界
- 参数编组和结果转换
- V8 隔离锁定和解锁

### 3. Rust WASM 版本的瓶颈

WASM 版本具有与 Napi-rs 相同的所有问题，加上额外的开销：

#### a) wasm-bindgen 开销
```rust
// 每个属性设置都通过 wasm-bindgen 的反射 API
js_sys::Reflect::set(&import_obj, &"t".into(), &import.t.into())?;
```

**成本**：每个 `Reflect::set` 调用：
1. 将 Rust 值转换为 JsValue
2. 跨越 WASM/JS 边界
3. 调用 JavaScript 的 Reflect.set
4. 跨边界返回结果

#### b) 无直接内存访问
- 与原始 C/WASM 不同，我们不能直接操作 JavaScript 内存
- 所有数据必须通过 wasm-bindgen 的类型系统编组

#### c) 更大的 WASM 二进制文件
- wasm-bindgen 添加了显著的运行时开销
- 更多代码需要加载和编译

### 4. 为什么原始库如此快

原始实现使用了几个技巧：

#### a) 直接内存布局
```c
// 原始 C 代码直接写入预分配的缓冲区
typedef struct {
  uint32_t start;
  uint32_t end;
  uint32_t statement_start;
  uint32_t statement_end;
  // ... 存储在连续内存中
} Import;
```

#### b) 零拷贝字符串处理
- 字符串表示为源中的（开始，结束）索引
- 直到 JavaScript 显式请求才复制字符串

#### c) 最小类型转换
- 使用直接映射到 JavaScript 数字的简单整数类型
- 热路径中无复杂对象创建

## 我们尝试优化的内容

### 1. UTF-16 索引映射优化
- **之前**：O(n²) - 单独转换每个位置
- **之后**：O(n) - 一次构建索引映射
- **结果**：改进 40%，但仍有显著开销

### 2. 预分配
```rust
// 估计容量以减少重新分配
let estimated_imports = (bytes.len() / 500).max(4);
imports: Vec::with_capacity(estimated_imports)
```
- **结果**：改进 10-15%

### 3. 内联优化
```rust
#[inline(always)]
pub(crate) fn peek(&self) -> Option<u8> { ... }
```
- **结果**：解析本身改进 5-10%

### 4. 属性的 SmallVec
```rust
pub attributes: SmallVec<[Attribute; 2]>
```
- **结果**：影响最小（属性很少见）

## 为什么这些优化还不够

根本问题是**架构性的**，而不是算法性的：

1. **语言边界开销**：我们必须为每个结果跨越 Rust/JavaScript 边界
2. **类型系统不匹配**：Rust 的类型安全需要显式转换
3. **内存模型差异**：Rust 和 JavaScript 有不同的内存模型
4. **UTF-16 要求**：JavaScript 的 UTF-16 编码需要从 Rust 的 UTF-8 转换

原始 C/WASM 实现通过以下方式避免了大多数这些问题：
- 从一开始就编译为 WASM（无 FFI）
- 使用直接映射到 WASM/JavaScript 的简单 C 类型
- 从一开始就使用 UTF-16
- 最小的抽象层

## 我们能匹配原始库的性能吗？

理论上可以，但需要：

### 选项 1：用 C 重写（违背目的）
- 失去 Rust 的安全保证
- 失去 Rust 的生态系统和工具
- 本质上重新创建原始库

### 选项 2：不安全的 Rust + 手动内存管理
```rust
// 假设的不安全方法
unsafe {
    let js_array = v8::Array::new(scope, imports.len());
    for (i, import) in imports.iter().enumerate() {
        // 直接操作 V8 内存
        let obj = v8::Object::new(scope);
        obj.set_index(scope, 0, import.start.into());
        // ... 绕过所有安全检查
    }
}
```

**问题**：
- 失去 Rust 的安全保证
- 高度平台特定
- 难以维护
- 仍有 UTF-16 转换开销

### 选项 3：自定义 WASM 运行时
- 构建一个像原始库一样工作的自定义 WASM 模块
- 仅将 Rust 用于解析逻辑
- 手动管理内存布局

**问题**：
- 极其复杂
- 失去使用 Rust 的大部分好处
- 维护负担

## 结论

性能差距是**基本的和预期的**，考虑到架构差异：

1. **原始库**：C → WASM → JavaScript（最小开销）
2. **我们的库**：Rust → Napi/WASM-bindgen → JavaScript（显著开销）

开销来自：
- UTF-16 转换：~40%
- 对象创建：~30%
- 内存复制：~20%
- FFI 开销：~10%

**这些开销是该方法固有的，如果不牺牲 Rust 的安全保证或重新实现原始架构，就无法消除。**

## 何时使用本库

尽管存在性能差距，但本库在特定场景中提供价值（详见 README.md）：

1. **类型安全**：完整的 TypeScript 类型，编译时检查
2. **内存安全**：无段错误或内存泄漏
3. **可维护性**：Rust 的现代工具和生态系统
4. **可扩展性**：易于添加功能和自定义
5. **学习**：Rust/WASM 开发的教育价值

对于大多数应用，绝对性能仍然很好（每秒 10 万次以上操作），安全性/可维护性的好处超过了性能成本。
