#![forbid(unsafe_code)]
mod config;

use config::{AddFieldConfig, GenFieldConfig, StructConfig};
use fxhash::FxHashMap as HashMap;
use fxhash::FxHashSet as HashSet;
use heck::ToLowerCamelCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    Attribute, Data, DataStruct, DeriveInput, Field, Fields, Generics, Ident, Type, TypePath,
    Visibility, WherePredicate, parse_macro_input, punctuated::Punctuated, spanned::Spanned,
};

#[proc_macro_derive(GenCamelCase, attributes(gen_camel, gen_field, add_field))]
pub fn derive_generate_struct(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_derive_generate_struct(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

fn expand_derive_generate_struct(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let DeriveInput {
        attrs,
        ident: original_struct_ident,
        generics,
        data,
        vis,
        ..
    } = input;
    let mut generated_code = Vec::new();
    if let Data::Struct(DataStruct { fields, .. }) = &data {
        // 处理结构体配置
        let (struct_config_map, filtered_struct_attrs) = parse_struct_config(&attrs)?;
        // 处理字段宏
        let (add_fields_map, gen_field_vec) = parse_field_config(fields, &struct_config_map)?;
        // 生成代码
        for struct_config in struct_config_map.values() {
            generated_code.push(generate_struct(
                struct_config,
                &filtered_struct_attrs,
                &gen_field_vec,
                &add_fields_map,
                &original_struct_ident,
                &generics,
                &vis,
            )?);
        }
    }
    Ok(quote! {
        #(#generated_code)*
    })
}

fn generate_struct<'a>(
    struct_config: &StructConfig,
    filtered_struct_attrs: &Vec<proc_macro2::TokenStream>,
    gen_field_vec: &Vec<(&Field, Vec<GenFieldConfig>, Vec<&Attribute>)>,
    add_fields_map: &HashMap<String, Vec<AddFieldConfig>>,
    original_struct_ident: &Ident,
    original_struct_generics: &Generics,
    original_struct_vis: &Visibility,
) -> syn::Result<proc_macro2::TokenStream> {
    // 生成结构体的字段定义
    let mut new_fields_def = Vec::new();
    // from 实现中将原始字段转换为新字段
    let mut field_conversions = Vec::new();
    // into 实现中将新字段转换为原始字段
    let mut reverse_conversions = Vec::new();
    // 处理from实现
    let mut from_impls = Vec::new();
    // 处理into实现
    let mut into_impls = Vec::new();
    // 跳过字段的from实现中设定默认值
    let mut skipped_defaults = Vec::new();

    let new_ident = generate_new_struct_ident(original_struct_ident, struct_config)?;

    // 处理泛型和where子句
    let (impl_generics, ty_generics, where_clause) = original_struct_generics.split_for_impl();
    let mut new_struct_generics = original_struct_generics.clone();
    {
        // 先处理where子句约束
        add_where_clauses(&mut new_struct_generics, &from_impls, &into_impls);

        // 获取或创建where子句
        let where_clause =
            new_struct_generics
                .where_clause
                .get_or_insert_with(|| syn::WhereClause {
                    where_token: syn::Token![where](Span::call_site()),
                    predicates: Punctuated::new(),
                });

        //组装字段
        for (field, field_config_vec, field_attrs) in gen_field_vec {
            let original_ident = field.ident.as_ref().unwrap();
            let new_field_ident = Ident::new(
                &original_ident.to_string().to_lower_camel_case(),
                original_ident.span(),
            );
            let field_vis = &field.vis;
            let original_ty = &field.ty;
            let mut global_field_config = None;
            let mut field_config = None;
            for fc in field_config_vec {
                if fc.id == struct_config.id {
                    field_config = Some(fc);
                }
                if fc.id == "" {
                    global_field_config = Some(fc);
                }
            }

            let mut merged_config = &GenFieldConfig {
                id: struct_config.id.clone(),
                ..Default::default()
            };
            if let Some(field_config) = field_config {
                merged_config = field_config;
            } else if let Some(global_field_config) = global_field_config {
                merged_config = global_field_config;
            }

            if let Some(true) = merged_config.field_skip {
                // 添加跳过字段的默认值
                skipped_defaults.push(quote! {
                    #original_ident: <#original_ty as Default>::default()
                });
                // 添加跳过字段的Default约束（去重）
                let predicate: WherePredicate = syn::parse_quote! { #original_ty: Default };
                if !where_clause.predicates.iter().any(|p| p == &predicate) {
                    where_clause.predicates.push(predicate);
                }
                continue;
            }
            if is_basic_type(&original_ty) {
                if merged_config.type_prefix.is_some()
                    || merged_config.type_suffix.is_some()
                    || merged_config.type_name.is_some()
                {
                    return Err(syn::Error::new(
                        original_ty.span(),
                        "Basic types cannot use type_prefix, type_suffix, or type_name",
                        // "基础类型不能使用 type_prefix、type_suffix 或 type_name 配置",
                    ));
                }
            }
            let new_ty = transform_type(
                &original_ty,
                &struct_config,
                &merged_config,
                &mut from_impls,
                &mut into_impls,
            )?;

            new_fields_def.push(quote! {
                #(#field_attrs)*
                #field_vis #new_field_ident: #new_ty
            });

            if is_std_collection_type(&original_ty) {
                field_conversions.push(quote! {
                    #new_field_ident: original.#original_ident.into_iter().map(Into::into).collect::<#new_ty>()
                });
                reverse_conversions.push(quote! {
                    #original_ident: new.#new_field_ident.into_iter().map(Into::into).collect::<#original_ty>()
                });
            } else {
                field_conversions.push(quote! {
                    #new_field_ident: original.#original_ident.into()
                });
                reverse_conversions.push(quote! {
                    #original_ident: new.#new_field_ident.into()
                });
            }
        }
        // 处理新增字段
        {
            let mut add_fields = vec![];
            if let Some(extra_fields) = add_fields_map.get(&struct_config.id) {
                add_fields.extend(extra_fields);
            }
            // 生成新增字段
            for extra_field in add_fields {
                let ident = &extra_field.ident;
                let ty = &extra_field.ty;
                new_fields_def.push(quote! {
                    #original_struct_vis #ident: #ty
                });
                field_conversions.push(quote! {
                    #ident: Default::default()
                });
                // 添加新增字段的Default约束（去重）
                let predicate: WherePredicate = syn::parse_quote! { #ty: Default };
                if !where_clause.predicates.iter().any(|p| p == &predicate) {
                    where_clause.predicates.push(predicate);
                }
            }
        }
    }

    //生成派生宏
    let derive_attrs = if !struct_config.derives.is_empty() {
        let derives = &struct_config.derives;
        quote! {
            #[derive(#(#derives),*)]
        }
    } else {
        quote! {}
    };

    // 生成结构体定义
    let new_struct_generics = if new_struct_generics.params.is_empty() {
        quote! {}
    } else {
        quote! { #new_struct_generics }
    };
    let new_struct = quote! {
        #derive_attrs
        #[allow(non_snake_case, non_camel_case_types)]
        #(#filtered_struct_attrs)*
        #original_struct_vis struct #new_ident #new_struct_generics {
            #(#new_fields_def,)*
        }
    };

    // 生成From转换实现
    let conversions = quote! {
        impl #impl_generics From<#original_struct_ident #ty_generics> for #new_ident #ty_generics #where_clause {
            fn from(original: #original_struct_ident #ty_generics) -> Self {
                Self {
                    #(#field_conversions,)*
                }
            }
        }

        impl #impl_generics From<#new_ident #ty_generics> for #original_struct_ident #ty_generics #where_clause {
            fn from(new: #new_ident #ty_generics) -> Self {
                Self {
                    #(#reverse_conversions,)*
                    #(#skipped_defaults,)*
                }
            }
        }
    };
    Ok(quote! {
        #new_struct
        #conversions
    })
}

fn parse_field_config<'a>(
    fields: &'a Fields,
    struct_config_map: &HashMap<String, StructConfig>,
) -> syn::Result<(
    HashMap<String, Vec<AddFieldConfig>>,
    Vec<(&'a Field, Vec<GenFieldConfig>, Vec<&'a Attribute>)>,
)> {
    let mut add_field_map: HashMap<String, Vec<AddFieldConfig>> = HashMap::default();
    let mut gen_field_vec = Vec::new();
    for field in fields.iter() {
        if field.attrs.is_empty() {
            gen_field_vec.push((field, Vec::new(), Vec::new()));
            continue;
        }
        let mut gen_field_configs: Vec<GenFieldConfig> = Vec::new();
        let mut field_attrs: Vec<&'a Attribute> = Vec::new();
        for attr in &field.attrs {
            // 解析新增的字段
            if let Some(add_field) = AddFieldConfig::from_attr(&attr)? {
                // 校验extra_field.id的有效性
                if !struct_config_map.contains_key(&add_field.id) {
                    return Err(syn::Error::new(
                        add_field.ident.span(),
                        format!("add_field's id '{}' not in gen_camel", add_field.id),
                        // format!("add_field 配置的 id '{}' 不存在", add_field.id),
                    ));
                }
                if let Some(extra_fields) = add_field_map.get_mut(&add_field.id) {
                    extra_fields.push(add_field);
                } else {
                    add_field_map.insert(add_field.id.clone(), vec![add_field]);
                }
                continue;
            }
            // 解析转换的字段
            if let Some(_iden) = &field.ident {
                if let Some(field_config) = GenFieldConfig::from_attr(&attr)? {
                    // 校验field_config.id的有效性
                    if !struct_config_map.contains_key(&field_config.id) {
                        return Err(syn::Error::new(
                            attr.span(),
                            format!(
                                "gen_field's id {} not in gen_camel",
                                // "gen_field 配置的 id '{}' 在 gen_camel 中不存在",
                                field_config.id
                            ),
                        ));
                    }
                    // 合并gen_field_config
                    if let Some(gen_field_config) = gen_field_configs
                        .iter_mut()
                        .find(|f| f.id == field_config.id)
                    {
                        gen_field_config.merge(field_config, attr.span())?;
                    } else {
                        gen_field_configs.push(field_config);
                    }
                } else if attr.path().is_ident("gen_camel") {
                    return Err(syn::Error::new(
                        attr.span(),
                        format!("gen_camel can't use in field"),
                        // format!("gen_camel 不能用在字段上"),
                    ));
                } else {
                    field_attrs.push(attr);
                }
            } else {
                return Err(syn::Error::new(
                    attr.span(),
                    format!("gen_field can't use in unnamed field"),
                    // format!("gen_field 配置的字段不是命名字段"),
                ));
            }
        }
        if let Some(_iden) = &field.ident {
            gen_field_vec.push((field, gen_field_configs, field_attrs));
        }
    }
    Ok((add_field_map, gen_field_vec))
}

fn parse_struct_config(
    attrs: &Vec<Attribute>,
) -> syn::Result<(HashMap<String, StructConfig>, Vec<proc_macro2::TokenStream>)> {
    let mut struct_config_map: HashMap<String, StructConfig> = HashMap::default();
    let mut filtered_attrs: Vec<proc_macro2::TokenStream> = Vec::new();
    // 处理结构体宏
    for attr in attrs {
        // 解析struct_config
        if let Some(struct_config) = StructConfig::from_attr(attr)? {
            if struct_config_map.contains_key(&struct_config.id) {
                if let Some(config) = struct_config_map.get_mut(&struct_config.id) {
                    config.merge(struct_config, attr.span())?;
                } else {
                    struct_config_map.insert(struct_config.id.clone(), struct_config);
                }
            } else {
                struct_config_map.insert(struct_config.id.clone(), struct_config);
            }
        }
        if attr.path().is_ident("add_field") {
            return Err(syn::Error::new(
                attr.span(),
                "add_field can't use in struct",
                // format!("add_field 不能用在 struct 上"),
            ));
        } else if attr.path().is_ident("gen_field") {
            return Err(syn::Error::new(
                attr.span(),
                "gen_field can't use in struct",
                // format!("gen_field 不能用在 struct 上"),
            ));
        } else if !attr.path().is_ident("gen_camel") {
            filtered_attrs.push(quote! { #attr });
        }
    }
    if struct_config_map.is_empty() {
        struct_config_map.insert("".to_string(), StructConfig::default());
    }
    Ok((struct_config_map, filtered_attrs))
}

fn generate_new_struct_ident(original: &Ident, config: &StructConfig) -> syn::Result<Ident> {
    if let Some(name) = &config.name {
        Ok(Ident::new(name, original.span()))
    } else {
        let prefix = config.prefix.as_deref().unwrap_or("");
        // 如果有配置的suffix则使用，否则对空id使用"Vo"，对非空id不使用默认后缀
        let suffix = config
            .suffix
            .as_deref()
            .unwrap_or(if config.prefix.is_none() { "Vo" } else { "" });
        // 存储原始标识符的字符串表示，延长其生命周期
        let original_str = original.to_string();

        // 预分配字符串容量
        let mut ident_str = String::with_capacity(prefix.len() + original_str.len() + suffix.len());
        ident_str.push_str(prefix);
        ident_str.push_str(&original_str);
        ident_str.push_str(suffix);

        if ident_str.is_empty() {
            return Err(syn::Error::new(
                original.span(),
                "gen_camel's name is empty, please check prefix, suffix and id",
                // "生成的结构体名称为空, 请检查prefix、suffix和id配置",
            ));
        }
        Ok(Ident::new(&ident_str, original.span()))
    }
}

fn transform_type<'a>(
    ty: &Type,
    struct_config: &StructConfig,
    field_config: &GenFieldConfig,
    from_impls: &mut Vec<WherePredicate>,
    into_impls: &mut Vec<WherePredicate>,
) -> syn::Result<Type> {
    let result = match ty {
        Type::Path(_) if is_basic_type(ty) => Ok(ty.clone()),

        Type::Path(type_path) if is_std_collection_type(ty) => {
            let mut new_path = type_path.path.clone();
            for segment in &mut new_path.segments {
                if let syn::PathArguments::AngleBracketed(args) = &mut segment.arguments {
                    for arg in &mut args.args {
                        if let syn::GenericArgument::Type(ty) = arg {
                            *ty = transform_type(
                                ty,
                                struct_config,
                                field_config,
                                from_impls,
                                into_impls,
                            )?;
                        }
                    }
                }
            }
            Ok(Type::Path(TypePath {
                qself: type_path.qself.clone(),
                path: new_path,
            }))
        }

        // 添加智能指针类型处理分支
        Type::Path(type_path) if is_smart_pointer_type(ty) => {
            let mut new_path = type_path.path.clone();
            // 递归转换内部泛型参数类型
            for segment in &mut new_path.segments {
                if let syn::PathArguments::AngleBracketed(args) = &mut segment.arguments {
                    for arg in &mut args.args {
                        if let syn::GenericArgument::Type(ty) = arg {
                            *ty = transform_type(
                                ty,
                                struct_config,
                                field_config,
                                from_impls,
                                into_impls,
                            )?;
                        }
                    }
                }
            }
            Ok(Type::Path(TypePath {
                qself: type_path.qself.clone(),
                path: new_path,
            }))
        }

        Type::Reference(r) => {
            let mut new_ref = r.clone();
            new_ref.elem = Box::new(transform_type(
                &r.elem,
                struct_config,
                field_config,
                from_impls,
                into_impls,
            )?);
            Ok(Type::Reference(new_ref))
        }

        Type::Array(a) => {
            let mut new_array = a.clone();
            new_array.elem = Box::new(transform_type(
                &a.elem,
                struct_config,
                field_config,
                from_impls,
                into_impls,
            )?);
            Ok(Type::Array(new_array))
        }

        Type::Tuple(t) => {
            let mut new_elems = Punctuated::new();
            for elem in &t.elems {
                new_elems.push(transform_type(
                    elem,
                    struct_config,
                    field_config,
                    from_impls,
                    into_impls,
                )?);
            }
            Ok(Type::Tuple(syn::TypeTuple {
                elems: new_elems,
                ..t.clone()
            }))
        }

        Type::Path(type_path) => {
            // 克隆原始路径以避免临时值问题
            let mut new_path = type_path.path.clone();

            if let Some(last) = new_path.segments.last_mut() {
                let ident = &last.ident;
                let ident_str = ident.to_string();
                // 检查是否为泛型参数（单个大写字母）
                let new_ident = if ident_str.len() == 1
                    && ident_str.chars().next().unwrap().is_uppercase()
                {
                    // 保留原始泛型参数名称
                    ident.clone()
                } else if let Some(name) = &field_config.type_name {
                    Ident::new(name, ident.span())
                } else {
                    if field_config.type_prefix.is_none() && field_config.type_suffix.is_none() {
                        let prefix = struct_config.prefix.as_deref().unwrap_or("");
                        let suffix = struct_config.suffix.as_deref().unwrap_or("Vo");
                        // 预分配字符串
                        let mut ident_str = String::with_capacity(
                            prefix.len() + ident.to_string().len() + suffix.len(),
                        );
                        ident_str.push_str(prefix);
                        ident_str.push_str(&ident.to_string());
                        ident_str.push_str(suffix);
                        Ident::new(&ident_str, ident.span())
                    } else {
                        let prefix = field_config.type_prefix.as_deref().unwrap_or("");
                        let suffix = field_config.type_suffix.as_deref().unwrap_or("");
                        // 预分配字符串
                        let mut ident_str = String::with_capacity(
                            prefix.len() + ident.to_string().len() + suffix.len(),
                        );
                        ident_str.push_str(prefix);
                        ident_str.push_str(&ident.to_string());
                        ident_str.push_str(suffix);
                        Ident::new(&ident_str, ident.span())
                    }
                };
                last.ident = new_ident;
            }

            // 克隆类型以延长其生命周期
            let orig_ty = ty.clone();
            let new_ty = Type::Path(TypePath {
                qself: None,
                path: new_path,
            });

            // 解决临时值生命周期问题：使用单独的作用域和变量存储
            let from_predicate = {
                // 将生成的字符串存储在变量中，延长其生命周期
                let orig_ty_str = quote!(#orig_ty).to_string();
                let new_ty_str = quote!(#new_ty).to_string();
                let predicate_str = format!("{}: Into<{}>", orig_ty_str, new_ty_str);
                syn::parse_str(&predicate_str)
                    .map_err(|e| syn::Error::new(ty.span(), format!("parse from predicate failed: {}", e)))?//解析From约束失败
            };

            let into_predicate = {
                // 同样处理Into约束
                let orig_ty_str = quote!(#orig_ty).to_string();
                let new_ty_str = quote!(#new_ty).to_string();
                let predicate_str = format!("{}: Into<{}>", new_ty_str, orig_ty_str);
                syn::parse_str(&predicate_str)
                    .map_err(|e| syn::Error::new(ty.span(), format!("parse into predicate failed: {}", e)))?//解析Into约束失败
            };

            // 添加转换约束（去重）
            if !from_impls
                .iter()
                .any(|p| quote!(#p).to_string() == quote!(#from_predicate).to_string())
            {
                from_impls.push(from_predicate);
            }

            if !into_impls
                .iter()
                .any(|p| quote!(#p).to_string() == quote!(#into_predicate).to_string())
            {
                into_impls.push(into_predicate);
            }

            Ok(new_ty)
        }

        _ => Err(syn::Error::new(
            ty.span(),
            format!("unsupport type: {:?}", ty),
            // format!("不支持的类型: {:?}", ty),
        )),
    };
    result
}

fn is_basic_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let ident = segment.ident.to_string();
            matches!(
                ident.as_str(),
                "i8" | "i16"
                    | "i32"
                    | "i64"
                    | "i128"
                    | "isize"
                    | "u8"
                    | "u16"
                    | "u32"
                    | "u64"
                    | "u128"
                    | "usize"
                    | "f32"
                    | "f64"
                    | "bool"
                    | "char"
                    | "str"
                    | "String"
            )
        } else {
            false
        }
    } else {
        false
    }
}

fn is_std_collection_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        // 检查是否是标准库集合类型
        if let Some(segment) = type_path.path.segments.last() {
            let ident = segment.ident.to_string();
            matches!(
                ident.as_str(),
                "Vec" | "HashMap" | "HashSet" | "BTreeMap" | "BTreeSet" | "LinkedList"
            )
        } else {
            false
        }
    } else {
        false
    }
}

fn is_smart_pointer_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let ident = segment.ident.to_string();
            // 支持常见智能指针类型
            matches!(
                ident.as_str(),
                "Option" | "Arc" | "Box" | "Rc" | "Mutex" | "RwLock"
            )
        } else {
            false
        }
    } else {
        false
    }
}

fn add_where_clauses(
    generics: &mut Generics,
    from_impls: &[WherePredicate],
    into_impls: &[WherePredicate],
) {
    let where_clause = generics
        .where_clause
        .get_or_insert_with(|| syn::WhereClause {
            where_token: syn::Token![where](Span::call_site()),
            predicates: Punctuated::new(),
        });

    // 添加From约束（去重）
    let mut existing_preds = HashSet::default();
    for p in &where_clause.predicates {
        existing_preds.insert(quote!(#p).to_string());
    }

    for pred in from_impls {
        let pred_str = quote!(#pred).to_string();
        if !existing_preds.contains(&pred_str) {
            where_clause.predicates.push(pred.clone());
            existing_preds.insert(pred_str);
        }
    }

    // 添加Into约束（去重）
    for pred in into_impls {
        let pred_str = quote!(#pred).to_string();
        if !existing_preds.contains(&pred_str) {
            where_clause.predicates.push(pred.clone());
            existing_preds.insert(pred_str);
        }
    }
}
