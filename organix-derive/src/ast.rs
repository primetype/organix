use crate::symbol::*;
use syn::{Attribute, Data, DataStruct, DeriveInput, Error, Fields, Ident, Member, Result, Type};

pub enum Input<'a> {
    Struct(Struct<'a>),
}

pub struct Struct<'a> {
    pub original: &'a DeriveInput,
    pub ident: Ident,
    pub fields: Vec<Field<'a>>,
    pub attrs: Attrs,
}

#[derive(Default)]
pub struct Attrs {
    pub shared: Option<bool>,
    pub skip: Option<bool>,
    pub io_driver: Option<bool>,
    pub time_driver: Option<bool>,
    pub core_threads: Option<usize>,
    pub max_threads: Option<usize>,
    pub thread_stack_size: Option<usize>,
}

pub struct Field<'a> {
    pub original: &'a syn::Field,
    pub attrs: Attrs,
    pub member: Member,
    pub ty: &'a Type,
}

impl<'a> Input<'a> {
    pub fn from_syn(node: &'a DeriveInput) -> Result<Self> {
        match &node.data {
            Data::Struct(data) => Struct::from_syn(node, data).map(Input::Struct),
            _ => Err(Error::new_spanned(node, "only structure are supported")),
        }
    }
}

impl<'a> Struct<'a> {
    fn from_syn(node: &'a DeriveInput, data: &'a DataStruct) -> Result<Self> {
        let fields = Field::multiple_from_syn(&data.fields)?;
        let attrs = Attrs::get(&node.attrs)?;

        Ok(Struct {
            original: node,
            ident: node.ident.clone(),
            fields,
            attrs,
        })
    }

    pub fn default_is_shared(&self) -> bool {
        self.attrs.shared(false)
    }
}

impl<'a> Field<'a> {
    fn multiple_from_syn(fields: &'a Fields) -> Result<Vec<Self>> {
        fields.iter().map(|field| Field::from_syn(field)).collect()
    }

    fn from_syn(node: &'a syn::Field) -> Result<Self> {
        Ok(Self {
            original: node,
            attrs: Attrs::get(&node.attrs)?,
            ty: &node.ty,
            member: node
                .ident
                .clone()
                .map(Member::Named)
                .ok_or_else(|| Error::new_spanned(node, "unnamed fields are not supported"))?,
        })
    }

    pub fn skip(&self) -> bool {
        self.attrs.skip()
    }

    pub fn shared(&self, default_value: bool) -> bool {
        self.attrs.shared(default_value)
    }

    pub fn io_driver(&self) -> bool {
        self.attrs.io_driver()
    }

    pub fn time_driver(&self) -> bool {
        self.attrs.time_driver()
    }
}

impl Attrs {
    fn skip(&self) -> bool {
        self.skip.unwrap_or_default()
    }

    fn shared(&self, default_value: bool) -> bool {
        self.shared.unwrap_or(default_value)
    }

    fn io_driver(&self) -> bool {
        self.io_driver.unwrap_or_default()
    }

    fn time_driver(&self) -> bool {
        self.time_driver.unwrap_or_default()
    }

    fn get(input: &[Attribute]) -> Result<Self> {
        let mut attrs = Self::default();

        for attr in input.iter().filter(|f| f.path == RUNTIME) {
            match attr.parse_meta()? {
                syn::Meta::List(meta_list) => {
                    for element in meta_list.nested {
                        use syn::{Meta::*, NestedMeta::*};
                        match &element {
                            // Parse `#[runtime(shared)]`
                            Meta(Path(word)) if word == SHARED => {
                                if attrs.shared.replace(true).is_some() {
                                    return Err(Error::new_spanned(
                                        element,
                                        "duplicated #[runtime(shared)]",
                                    ));
                                }
                            }
                            // Parse `#[runtime(skip)]`
                            Meta(Path(word)) if word == SKIP => {
                                if attrs.skip.replace(true).is_some() {
                                    return Err(Error::new_spanned(
                                        element,
                                        "duplicated #[runtime(skip)]",
                                    ));
                                }
                            }
                            // Parse `#[runtime(io)]`
                            Meta(Path(word)) if word == IO_DRIVER => {
                                if attrs.io_driver.replace(true).is_some() {
                                    return Err(Error::new_spanned(
                                        element,
                                        "duplicated #[runtime(io)]",
                                    ));
                                }
                            }
                            // Parse `#[runtime(time)]`
                            Meta(Path(word)) if word == TIME_DRIVER => {
                                if attrs.time_driver.replace(true).is_some() {
                                    return Err(Error::new_spanned(
                                        element,
                                        "duplicated #[runtime(time)]",
                                    ));
                                }
                            }
                            _ => return Err(Error::new_spanned(element, "unexpected attribute")),
                        }
                    }
                }
                other => return Err(Error::new_spanned(other, "expected #[runtime(...)]")),
            }
        }

        Ok(attrs)
    }
}
