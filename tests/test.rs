use snake_to_camel::GenCamelCase;
use std::collections::HashMap;

// 1. 基本转换测试 - 验证snake_case到camelCase的转换
#[derive(Debug, PartialEq, GenCamelCase)]
#[gen_camel(name = "BasicUserDto")]
struct BasicUser {
    user_id: u64,
    user_name: String,
    is_active: bool,
}

// 2. 嵌套结构体测试
#[derive(Debug, PartialEq, GenCamelCase)]
#[gen_camel(suffix="Vo")]
struct OrderItem {
    product_id: u64,
    unit_price: f64,
    quantity: u32,
}

#[derive(Debug, PartialEq, GenCamelCase)]
#[gen_camel(name = "OrderDto")]
struct Order {
    order_id: u64,
    customer_name: String,
    order_items: Vec<OrderItem>,
    total_amount: f64,
}

// 3. 字段级配置测试
#[derive(Debug, PartialEq, GenCamelCase)]
#[gen_camel(name = "FieldConfigDto")]
struct FieldConfigExample {
    special_name: String,
    #[gen_field(field_skip)]
    internal_id: u32,
    item_count: u32,
}

// 4. 额外字段测试
#[derive(Debug, PartialEq, GenCamelCase)]
#[gen_camel(name = "WithExtraFieldsDto")]
struct BaseStruct {
    #[add_field(field_name = "isValid", field_type = "bool")]
    #[add_field(field_name = "timestamp", field_type = "u64")]
    field_one: String,
    field_two: i32,
    }
// struct WithExtraFieldsDto {
//     fieldOne: String,
//     fieldTwo: i32,
//     isValid: bool,
//     timestamp: u64,
// }

// 5. 泛型结构体测试
#[derive(Debug, PartialEq, GenCamelCase)]
#[gen_camel(name = "GenericDataDto")]
struct GenericData<T, U>
where
    T: Clone + PartialEq,
    U: PartialEq,
{
    data_id: u64,
    payload: T,
    metadata: Option<U>,
}
// struct GenericDataDto<T, U>
// where
//     T: Clone + PartialEq,
//     U: PartialEq, 
// {
//     dataId: u64,
//     payload: T,
//     metadata: Option<U>,
// }

// 6. 集合类型测试
#[derive(Debug, PartialEq, GenCamelCase)]
#[gen_camel(name = "CollectionTypesDto")]
struct CollectionTypes {
    ids: Vec<u64>,
    names: Option<Vec<String>>,
    properties: HashMap<String, i32>,
}

// struct CollectionTypesDto {
//     ids: Vec<u64>,
//     names: Option<Vec<String>>,
//     properties: HashMap<String, i32>,
// }

// 7. 递归类型转换测试
#[derive(Debug, PartialEq, GenCamelCase, Clone)]
#[gen_camel(name = "RecursiveChildDto", derive="Clone, Debug, Default")]
struct RecursiveChild {
    child_id: u32,
    value: String,
}
//#[derive(Clone, Debug, Default)]
// struct RecursiveChildDto {
//     childId: u32,
//     value: String,
// }

#[derive(Debug, PartialEq, GenCamelCase)]
#[gen_camel(name = "RecursiveParentDto", derive="Debug, Clone")]
struct RecursiveParent {
    parent_id: u32,
    #[gen_field(type_name = "RecursiveChild")]
    child: RecursiveChild,
    #[gen_field(type_name = "RecursiveChild")]
    grandchildren: Vec<RecursiveChild>,
    #[gen_field(type_name="DateTime")]
    pub test_name: chrono::DateTime<chrono::Utc>,
}
// #[derive(Clone)]
// struct RecursiveParentDto {
//     parentId: u32,
//     child: RecursiveChild,
//     grandchildren: Vec<RecursiveChild>,
//     pub testName: DateTime<Utc>,
// }

#[test]
fn test_basic_struct_conversion() {
    let original = BasicUser {
        user_id: 1,
        user_name: "Test User".to_string(),
        is_active: true,
    };

    let converted: BasicUserDto = original.into();

    assert_eq!(converted.userId, 1);
    assert_eq!(converted.userName, "Test User");
    assert_eq!(converted.isActive, true);
}

#[test]
fn test_nested_struct_conversion() {
    let original_items = vec![OrderItem {
        product_id: 100,
        unit_price: 19.99,
        quantity: 2,
    }];

    let original = Order {
        order_id: 1001,
        customer_name: "John Doe".to_string(),
        order_items: original_items,
        total_amount: 39.98,
    };

    let converted: OrderDto = original.into();

    assert_eq!(converted.orderId, 1001);
    assert_eq!(converted.customerName, "John Doe");
    assert_eq!(converted.orderItems.len(), 1);
    assert_eq!(converted.orderItems[0].productId, 100);
    assert_eq!(converted.totalAmount, 39.98);
}

#[test]
fn test_field_level_configurations() {
    let original = FieldConfigExample {
        special_name: "test".to_string(),
        internal_id: 42,
        item_count: 5,
    };

    let converted: FieldConfigDto = original.into();

    // 测试重命名
    assert_eq!(converted.specialName, "test");
    // 测试类型转换
    assert_eq!(converted.itemCount, 5u32);
    // 测试跳过字段（应该使用默认值）
    //assert_eq!(converted.internalId, 0);//no field `internalId` on type `FieldConfigDto`
}

#[test]
fn test_extra_fields_addition() {
    let original = BaseStruct {
        field_one: "value1".to_string(),
        field_two: 42,
    };

    let converted: WithExtraFieldsDto = original.into();

    assert_eq!(converted.fieldOne, "value1");
    assert_eq!(converted.fieldTwo, 42);
    // 测试额外字段
    assert_eq!(converted.isValid, false);
    assert_eq!(converted.timestamp, 0);
}

#[test]
fn test_generic_struct_conversion() {
    let original = GenericData {
        data_id: 1,
        payload: "test payload".to_string(),
        metadata: Some(42),
    };

    let converted: GenericDataDto<String, i32> = original.into();

    assert_eq!(converted.dataId, 1);
    assert_eq!(converted.payload, "test payload");
    assert_eq!(converted.metadata, Some(42));
}

#[test]
fn test_collection_types_conversion() {
    let mut properties = HashMap::new();
    properties.insert("length".to_string(), 10);
    properties.insert("width".to_string(), 20);

    let original = CollectionTypes {
        ids: vec![1, 2, 3],
        names: Some(vec!["a".to_string(), "b".to_string()]),
        properties,
    };

    let converted: CollectionTypesDto = original.into();

    assert_eq!(converted.ids, vec![1, 2, 3]);
    assert_eq!(converted.names, Some(vec!["a".to_string(), "b".to_string()]));
    assert_eq!(converted.properties.get("length"), Some(&10));
}

#[test]
fn test_recursive_type_conversion() {
    let child = RecursiveChild {
        child_id: 1,
        value: "child".to_string(),
    };

    let original = RecursiveParent {
        parent_id: 10,
        child: child.clone(),
        grandchildren: vec![child],
        test_name: chrono::Utc::now(),
    };

    let converted: RecursiveParentDto = original.into();
    println!("{:?}", converted);
    assert_eq!(converted.parentId, 10);
    assert_eq!(converted.child.child_id, 1);
    assert_eq!(converted.grandchildren.len(), 1);
    assert_eq!(converted.grandchildren[0].value, "child");
}