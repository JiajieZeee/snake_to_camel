# snake_to_camel
for English, see **[readmeEn](readmeEn.md)**

一个用于将蛇形命名(snake_case)结构体转换为驼峰命名(camelCase)结构体的Rust库，提供自动派生宏和类型转换功能。

## 功能特点
- 自动将结构体字段从snake_case转换为camelCase
- 支持嵌套类型的递归转换
- 可自定义结构体和字段级别的转换规则
- 自动生成`From`/`Into` trait实现，实现原始结构体与生成结构体之间的互相转换
- 支持添加/过滤字段
- 处理泛型和where子句约束

## 使用说明
### 1. 引入库
### 2. 在struct上使用`#[derive(GenCamelCase)]`宏自动实现转换
### 3. 配置转换规则
本库包含三个属性宏: gen_camel, gen_field, add_field
#### 3.1 gen_camel属性宏
gen_camel包含四个配置：name, prefix, suffix, id
 - name: 自定义结构体名称, 此配置被设置时, 生成结构体名称时prefix和suffix将被忽略
 - prefix: 自定义前缀, name未设置时对struct生效, 默认为""
 - suffix: 自定义后缀, name未设置时对struct生效, 默认为"Vo"
 - derive: 自定义派生宏, 显式指定被生成的结构体的派生宏, 无默认值
 - id: 自定义id, 与下面两个宏配合, 设置不同的id用于生成不同的结构体, 默认为""
#### 3.2 gen_field属性宏
gen_field包含五个配置：type_name, type_prefix, type_suffix, field_skip, id
 - type_name: 自定义类型名称, 此配置不能和type_prefix和type_suffix同时使用
 - type_prefix: 自定义类型前缀, 此配置不能和type_name同时使用, 未配置时使用gen_camel的prefix
 - type_suffix: 自定义类型后缀, 此配置不能和type_name同时使用, 未配置时使用gen_camel的suffix
 - field_skip: 转换时跳过此字段, 此配置不能和type_name, type_prefix, type_suffix同时使用
 - id: 自定义id, 与gen_camel的id配合, 生成不同的结构体, 默认为""
#### 3.3 add_field属性宏
add_field包含三个配置：field_name, field_type, id
 - field_name: 自定义字段名称
 - field_type: 自定义字段类型
 - id: 自定义id, 与gen_camel的id配合, 生成不同的结构体, 默认为""

## 安装
在`Cargo.toml`中添加依赖：
```toml
[dependencies]
snake_to_camel = "0.1.0"
```

## 基本用法

### 简单转换
```rust
use snake_to_camel::GenCamelCase;

#[derive(GenCamelCase)]
struct User {
    user_id: u64,
    user_name: String,
}
```

上述代码将生成一个名为`UserVo`的新结构体，字段名称自动转换为camelCase：
```rust
struct UserVo {
    userId: u64,
    userName: String,
}

impl From<User> for UserVo {
    // 自动生成的转换实现
}
```

## 高级配置

### 结构体级别配置
```rust
#[derive(GenCamelCase)]
#[gen_camel(name = "AdvancedUserDto")]
struct AdvancedUser<T> {
    user_id: u64,
    user_data: T,
}
```

```rust
#[derive(GenCamelCase)]
#[gen_camel(name = "AdvancedUserDto", id = "vo")]
#[gen_camel(suffix = "Dto", id = "dto")]
#[gen_camel(prefix = "Add", id = "dto2")]
struct AdvancedUser<T> {
    user_id: u64,
    user_data: T,
}
//这将会生成三个结构体: AdvancedUserVo, AdvancedUserDto, AdvancedUserDto2
```

### 字段级别配置
```rust
#[derive(GenCamelCase)]
#[gen_camel(name = "CustomFieldDto")]
struct CustomFieldExample {
    special_name: String,
    #[gen_field(field_skip)] // 跳过此字段
    internal_id: u32,
    count: u32,
}
```

### 添加额外字段
```rust
#[derive(GenCamelCase)]
#[gen_camel(name = "WithExtraFieldsDto")]
#[add_field(field_name = "isActive", field_type = "bool")]
#[add_field(field_name = "timestamp", field_type = "u64")]
struct BaseStruct {
    field_one: String,
    field_two: i32,
}
```

## 类型转换规则
- 基本类型保持不变
- 标准集合类型(`Vec<T>`, `Option<T>`, `HashMap<K, V>`等)会递归转换其泛型参数
- 自定义类型会尝试应用相同的转换规则
- 支持嵌套结构体的转换

## 完整示例
```rust
use snake_to_camel::GenStruct;

#[derive(GenCamelCase)]
#[gen_camel(new_name = "OrderDto")]
struct Order {
    order_id: u64,
    customer_name: String,
    #[gen_field(type_suffix = "Dto")]
    order_items: Vec<OrderItem>,
    #[gen_field(field_skip)]
    internal_notes: String,
}

#[derive(GenCamelCase)]
#[gen_camel(name = "OrderItemDto")]
struct OrderItem {
    product_id: u64,
    quantity: u32,
    unit_price: f64,
}

fn main() {
    let order = Order {
        order_id: 1,
        customer_name: "John Doe".to_string(),
        order_items: vec![OrderItem {
            product_id: 100,
            quantity: 2,
            unit_price: 19.99,
        }],
        internal_notes: "Confidential".to_string(),
    };

    let order_dto: OrderDto = order.into();
    println!("Converted DTO: {:?}", order_dto);
}
```

## 许可证
**[MIT License](license)**