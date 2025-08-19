# snake_to_camel
中文说明请看 **[readme](/readme.md)**

A Rust library for converting snake_case structs to camelCase structs, providing automatic derive macros and type conversion functionality.

## Features
- Automatically converts struct fields from snake_case to camelCase
- Supports recursive conversion of nested types
- Customizable conversion rules at both struct and field levels
- Automatically generates `From`/`Into` trait implementations for bidirectional conversion between original and generated structs
- Supports adding/filtering fields
- Handles generics and where clause constraints

## Usage Instructions
### 1. Import the library
### 2. Use the `#[derive(GenCamelCase)]` macro on structs to automatically implement conversion
### 3. Configure conversion rules
This library includes three attribute macros: gen_camel, gen_field, add_field
#### 3.1 gen_camel attribute macro
gen_camel includes four configurations: name, prefix, suffix, id
 - name: Custom struct name, when set, prefix and suffix will be ignored
 - prefix: Custom prefix, applies to struct when name is not set, default is ""
 - suffix: Custom suffix, applies to struct when name is not set, default is "Vo"
 - derive: Custom derive macros, explicitly specifies derive macros for generated struct, no default
 - id: Custom identifier, works with the following two macros to generate different structs with different ids, default is ""
#### 3.2 gen_field attribute macro
gen_field includes five configurations: type_name, type_prefix, type_suffix, field_skip, id
 - type_name: Custom type name, cannot be used with type_prefix and type_suffix
 - type_prefix: Custom type prefix, cannot be used with type_name, uses gen_camel's prefix if not set
 - type_suffix: Custom type suffix, cannot be used with type_name, uses gen_camel's suffix if not set
 - field_skip: Skip this field during conversion, cannot be used with type_name, type_prefix, type_suffix
 - id: Custom identifier, works with gen_camel's id to generate different structs, default is ""
#### 3.3 add_field attribute macro
add_field includes three configurations: field_name, field_type, id
 - field_name: Custom field name
 - field_type: Custom field type
 - id: Custom identifier, works with gen_camel's id to generate different structs, default is ""

## Installation
Add dependency in `Cargo.toml`:
```toml
[dependencies]
snake_to_camel = "0.1.0"
```

## Basic Usage

### Simple Conversion
```rust
use snake_to_camel::GenCamelCase;

#[derive(GenCamelCase)]
struct User {
    user_id: u64,
    user_name: String,
}
```

The above code will generate a new struct named `UserVo` with field names automatically converted to camelCase:
```rust
struct UserVo {
    userId: u64,
    userName: String,
}

impl From<User> for UserVo {
    // Automatically generated conversion implementation
}
```

## Advanced Configuration

### Struct-level Configuration
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
// This will generate three structs: AdvancedUserVo, AdvancedUserDto, AdvancedUserDto2
```

### Field-level Configuration
```rust
#[derive(GenCamelCase)]
#[gen_camel(name = "CustomFieldDto")]
struct CustomFieldExample {
    special_name: String,
    #[gen_field(field_skip)] // Skip this field
    internal_id: u32,
    count: u32,
}
```

### Adding Extra Fields
```rust
#[derive(GenCamelCase)]
#[gen_camel(name = "WithExtraFieldsDto")]
struct BaseStruct {
    #[add_field(field_name = "isActive", field_type = "bool")]
    #[add_field(field_name = "timestamp", field_type = "u64")]
    field_one: String,
    field_two: i32,
}
```

## Type Conversion Rules
- Basic types remain unchanged
- Standard collection types (`Vec<T>`, `Option<T>`, `HashMap<K, V>`, etc.) recursively convert their generic parameters
- Custom types will attempt to apply the same conversion rules
- Supports conversion of nested structs

## Complete Example
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

## License
**[MIT License](license)**