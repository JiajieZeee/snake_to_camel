use proc_macro2::Span;
use syn::{Attribute, Ident, Lit, LitStr, Path, Type, spanned::Spanned};

#[derive(Default, Clone, PartialEq)]
pub(crate) struct StructConfig {
    pub(crate) id: String,
    pub(crate) prefix: Option<String>,
    pub(crate) suffix: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) derives: Vec<Path>,
}

#[derive(Default, Clone)]
pub(crate) struct GenFieldConfig {
    pub(crate) id: String,
    pub(crate) type_prefix: Option<String>,
    pub(crate) type_suffix: Option<String>,
    pub(crate) type_name: Option<String>,
    pub(crate) field_skip: Option<bool>,
}

#[derive(Clone)]
pub(crate) struct AddFieldConfig {
    pub(crate) id: String,
    pub(crate) ident: Ident,
    pub(crate) ty: Type,
}

// pub(crate) struct OriginalFieldConfig<'a> {
//     pub(crate) vis: &'a Visibility,
//     pub(crate) ty: &'a Type,
// }

impl StructConfig {
    pub(crate) fn from_attr(attr: &Attribute) -> syn::Result<Option<Self>> {
        if attr.path().is_ident("gen_camel") {
            let mut config = StructConfig::default();
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("id") {
                    let value = meta.value()?;
                    let s: LitStr = value.parse()?;
                    config.id = s.value();
                } else if meta.path.is_ident("prefix") {
                    let value = meta.value()?;
                    let s: Lit = value.parse()?;
                    if let Lit::Str(lit) = s {
                        config.prefix = Some(lit.value());
                    }
                } else if meta.path.is_ident("suffix") {
                    let value = meta.value()?;
                    let s: Lit = value.parse()?;
                    if let Lit::Str(lit) = s {
                        config.suffix = Some(lit.value());
                    }
                } else if meta.path.is_ident("name") {
                    let value = meta.value()?;
                    let s: Lit = value.parse()?;
                    if let Lit::Str(lit) = s {
                        config.name = Some(lit.value());
                    }
                } else if meta.path.is_ident("derive") {
                    let value = meta.value()?;
                    let derives_str: LitStr = value.parse()?;

                    config.derives = derives_str
                        .value()
                        .split(',')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|s| syn::parse_str::<Path>(s))
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|e| meta.error(format!("Invalid derive path: {}", e)))?;//无效的 derive 路径:
                } else {
                    return Err(meta.error("gen_camel only supports id, name, prefix, suffix, derive"));
                    //return Err(meta.error("gen_camel 属性只支持 id, name, prefix, suffix, derive"));
                }
                if config.name.is_none()
                    && config.prefix.is_none()
                    && config.suffix.is_none()
                    && config.derives.is_empty()
                {
                    return Err(
                        meta.error("gen_camel must specify name, prefix, suffix, derive at least one")
                        //meta.error("gen_camel 属性必须指定 name, prefix, suffix, derive 中的一个")
                    );
                }
                Ok(())
            })?;
            return Ok(Some(config));
        }
        Ok(None)
    }

    pub(crate) fn merge(&mut self, new_config: StructConfig, span: Span) -> syn::Result<()> {
        if let Some(prefix) = new_config.prefix {
            if self.prefix.is_some() && self.prefix.as_ref() != Some(&prefix) {
                return Err(syn::Error::new(
                    span,
                        "prefix redefined with different values",
                        //"gen_camel id '{}' 的 prefix 属性重复定义且值不同",
                ));
            }
            self.prefix = Some(prefix);
        }
        // 检查并合并suffix
        if let Some(suffix) = new_config.suffix {
            if self.suffix.is_some() && self.suffix.as_ref() != Some(&suffix) {
                return Err(syn::Error::new(
                    span,
                    "suffix redefined with different values",
                //     format!(
                //         "gen_camel id '{}' 的 suffix 属性重复定义且值不同",
                //         new_config.id
                //     ),
                ));
            }
            self.suffix = Some(suffix);
        }
        // 检查并合并name
        if let Some(name) = new_config.name {
            if self.name.is_some() && self.name.as_ref() != Some(&name) {
                return Err(syn::Error::new(
                    span,
                    "name redefined with different values",
                    // format!(
                    //     "gen_camel id '{}' 的 name 属性重复定义且值不同",
                    //     new_config.id
                    // ),
                ));
            }
            self.name = Some(name);
        }
        // 合并derives
        self.derives.extend(new_config.derives);
        Ok(())
    }
}

impl GenFieldConfig {
    pub(crate) fn from_attr(attr: &Attribute) -> syn::Result<Option<Self>> {
        if attr.path().is_ident("gen_field") {
            let mut config = GenFieldConfig::default();
            attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("id") {
                        let value = meta.value()?;
                        let s: LitStr = value.parse()?;
                        config.id = s.value();
                    } else if meta.path.is_ident("field_skip") {
                        config.field_skip = Some(true);
                    } else if meta.path.is_ident("type_prefix") {
                        let value = meta.value()?;
                        let s: Lit = value.parse()?;
                        if let Lit::Str(lit) = s {
                            config.type_prefix = Some(lit.value());
                        }
                    } else if meta.path.is_ident("type_suffix") {
                        let value = meta.value()?;
                        let s: Lit = value.parse()?;
                        if let Lit::Str(lit) = s {
                            config.type_suffix = Some(lit.value());
                        }
                    } else if meta.path.is_ident("type_name") {
                        let value = meta.value()?;
                        let s: Lit = value.parse()?;
                        if let Lit::Str(lit) = s {
                            config.type_name = Some(lit.value());
                        }
                    } else {
                        return Err(meta.error("gen_field only support id, field_skip, type_prefix, type_suffix, type_name"));
                        // return Err(meta.error("gen_field 属性只支持 id, field_skip, type_prefix, type_suffix, type_name"));
                    }
                    if config.field_skip.is_some() {
                        if config.type_prefix.is_some() || config.type_suffix.is_some() || config.type_name.is_some() {
                            return Err(meta.error("field_skip cannot be used with type_prefix, type_suffix, or type_name"));
                            // return Err(meta.error("field_skip 不能和 type_prefix, type_suffix, type_name 同时使用"));
                        }
                    } else {
                        if config.type_prefix.is_none() && config.type_suffix.is_none() && config.type_name.is_none() {
                            return Err(meta.error("gen_field must specify one of field_skip, type_prefix, type_suffix, or type_name"));
                            // return Err(meta.error("gen_field 属性必须指定 field_skip, type_prefix, type_suffix, type_name 中的一个"));
                        }
                    }
                    if config.type_name.is_some() && (config.type_prefix.is_some() || config.type_suffix.is_some()) {
                        return Err(meta.error("type_name cannot be used with type_prefix, type_suffix"));
                        // return Err(meta.error("gen_field 的 type_name 属性不能和 type_prefix, type_suffix 同时使用"));
                    }
                    Ok(())
                })?;
            return Ok(Some(config));
        }
        Ok(None)
    }

    pub(crate) fn merge(&mut self, new_config: GenFieldConfig, span: Span) -> syn::Result<()> {
        // 合并type_prefix
        if let Some(type_prefix) = new_config.type_prefix {
            if self.type_prefix.is_some() && self.type_prefix.as_ref() != Some(&type_prefix) {
                return Err(syn::Error::new(
                    span,
                    "type_prefix redefined with different values",
                    // format!("gen_field 的 type_prefix 属性重复定义且值不同"),
                ));
            }
            self.type_prefix = Some(type_prefix);
        }
        // 合并type_suffix
        if let Some(type_suffix) = new_config.type_suffix {
            if self.type_suffix.is_some() && self.type_suffix.as_ref() != Some(&type_suffix) {
                return Err(syn::Error::new(
                    span,
                    "type_suffix redefined with different values",
                    // format!("gen_field 的 type_suffix 属性重复定义且值不同"),
                ));
            }
            self.type_suffix = Some(type_suffix);
        }
        // 合并name
        if let Some(type_name) = new_config.type_name {
            if self.type_name.is_some() && self.type_name.as_ref() != Some(&type_name) {
                return Err(syn::Error::new(
                    span,
                    "type_name redefined with different values",
                    // format!("gen_field 的 type_name 属性重复定义且值不同"),
                ));
            }
            self.type_name = Some(type_name);
        }
        //合并field_skip
        if let Some(field_skip) = new_config.field_skip {
            if self.field_skip.is_some() && self.field_skip.as_ref() != Some(&field_skip) {
                return Err(syn::Error::new(
                    span,
                    "field_skip redefined with different values",
                    // format!("gen_field 的 field_skip 属性重复定义且值不同"),
                ));
            }
            self.field_skip = Some(field_skip);
        }
        if self.type_name.is_some() && (self.type_prefix.is_some() || self.type_suffix.is_some()) {
            return Err(syn::Error::new(
                span,
                "type_name cannot be used with type_prefix, type_suffix",
                // format!("gen_field 的 type_name 属性不能和 type_prefix, type_suffix 同时使用"),
            ));
        }
        Ok(())
    }
}

impl AddFieldConfig {
    pub(crate) fn from_attr(attr: &Attribute) -> syn::Result<Option<Self>> {
        if attr.path().is_ident("add_field") {
            let mut id = String::new();
            let mut field_ident = None;
            let mut field_type = None;

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("id") {
                    let value = meta.value()?;
                    let s: LitStr = value.parse()?;
                    id = s.value();
                } else if meta.path.is_ident("field_name") {
                    let value = meta.value()?;
                    let s: LitStr = value.parse()?;
                    field_ident = Some(s.parse()?);
                } else if meta.path.is_ident("field_type") {
                    let value = meta.value()?;
                    let s: LitStr = value.parse()?;
                    field_type = Some(s.parse()?);
                } else {
                    return Err(meta.error("add_field only support id, field_name, field_type"));
                    // return Err(meta.error("add_field 只支持 id、field_name 和 field_type 参数"));
                }
                Ok(())
            })?;
            match (field_ident, field_type) {
                (Some(ident), Some(ty)) => {
                    return Ok(Some(AddFieldConfig { id, ident, ty }));
                }
                _ => {
                    return Err(syn::Error::new(
                        attr.span(),
                        "add_field need specify field_name and field_type",
                        // "add_field 需要同时指定 field_name 和 field_type 参数",
                    ));
                }
            }
        }
        Ok(None)
    }

    // pub(crate) fn merge(&mut self, new_config: AddFieldConfig, span: Span) -> syn::Result<()> {
    //     if self.id == new_config.id {
    //         if self.ident != new_config.ident {
    //             return Err(syn::Error::new(
    //                 span,
    //                 format!("add_field 的 field_name 属性重复定义且值不同"),
    //             ));
    //         }
    //         if self.ty != new_config.ty {
    //             return Err(syn::Error::new(
    //                 span,
    //                 format!("add_field 的 field_type 属性重复定义且值不同"),
    //             ));
    //         }
    //     }
    //     Ok(())
    // }
}

// impl<'a> OriginalFieldConfig<'a> {
//     pub(crate) fn from_field(field: &'a Field) -> Self {
//         Self {
//             vis: &field.vis,
//             ty: &field.ty,
//         }
//     }
// }
