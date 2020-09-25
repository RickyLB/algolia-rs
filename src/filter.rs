use sealed::Sealed;
use serde::{Serialize, Serializer};
use std::fmt::Display;

mod sealed {
    pub trait Sealed {}
}

pub trait CommonFilterKind: Display + Sealed {}
pub trait AndFilterable: Display + Sealed {}
pub trait Filterable: Display + Sealed {}

struct Separated<'a, T>(&'a [T], &'static str);

impl<'a, T: Display> Display for Separated<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.0.iter();

        if let Some(item) = iter.next() {
            item.fmt(f)?;
        }

        for item in iter {
            f.write_str(self.1)?;
            item.fmt(f)?;
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum FilterOperator {
    Lt,
    Le,
    Eq,
    Ne,
    Ge,
    Gt,
}

impl FilterOperator {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Eq => "=",
            Self::Ne => "!=",
            Self::Ge => ">=",
            Self::Gt => ">",
        }
    }
}

impl Display for FilterOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

pub struct BooleanFilter {
    pub facet_name: String,
    pub value: bool,
}

impl Display for BooleanFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // example of format: "isEnabled":true
        write!(f, r#""{}":{}"#, &self.facet_name.escape_debug(), self.value)
    }
}

pub struct TagFilter(pub String);

impl Display for TagFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, r#"_tags:"{}""#, self.0.escape_debug())
    }
}

pub struct FacetFilter {
    pub facet_name: String,
    pub value: String,
}

impl Display for FacetFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#""{}":"{}""#,
            self.facet_name.escape_debug(),
            self.value.escape_debug()
        )
    }
}

pub struct RangeFilter {
    pub attribute_name: String,
    pub lower_bound: f64,
    pub upper_bound: f64,
}

impl Display for RangeFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#""{}": {} TO {}"#,
            self.attribute_name.escape_debug(),
            self.lower_bound,
            self.upper_bound
        )
    }
}

pub struct CmpFilter {
    pub attribute_name: String,
    pub operator: FilterOperator,
    pub value: f64,
}

impl Display for CmpFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#""{}" {} {}"#,
            self.attribute_name.escape_debug(),
            self.operator,
            self.value
        )
    }
}

pub struct CommonFilter<T: CommonFilterKind> {
    pub invert: bool,
    pub filter: T,
}

impl<T: CommonFilterKind> Display for CommonFilter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.invert {
            f.write_str("NOT ")?;
        }

        write!(f, "{}", self.filter)
    }
}

pub struct OrFilter<T: CommonFilterKind> {
    pub filters: Vec<CommonFilter<T>>,
}

impl<T: CommonFilterKind> Display for OrFilter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Separated(&self.filters, " OR ").fmt(f)
    }
}

pub struct AndFilter {
    pub filters: Vec<Box<dyn AndFilterable>>,
}

impl Display for AndFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Separated(&self.filters, " OR ").fmt(f)
    }
}

pub struct EmptyFilter;

impl Display for EmptyFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}

pub struct Filter<T: Filterable = EmptyFilter>(pub T);

impl<T: Default + Filterable> Default for Filter<T> {
    fn default() -> Self {
        Self(T::default())
    }
}

impl<T: Filterable> From<T> for Filter<T> {
    fn from(filter: T) -> Self {
        Self(filter)
    }
}

impl<T: Filterable> Serialize for Filter<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(&self.0)
    }
}

macro_rules! mark {
    ($mark:ident; $( $t:ty ),+ $(,)? ) => {
        $(
            impl $mark for $t {}
        )+
    };
}

mark!(Sealed; BooleanFilter, TagFilter, FacetFilter, RangeFilter, CmpFilter, AndFilter, EmptyFilter);
mark!(CommonFilterKind; BooleanFilter, TagFilter, FacetFilter, RangeFilter, CmpFilter);

impl<T: CommonFilterKind> Sealed for CommonFilter<T> {}
impl<T: CommonFilterKind> Sealed for OrFilter<T> {}

impl<T: CommonFilterKind> AndFilterable for OrFilter<T> {}
impl<T: CommonFilterKind> AndFilterable for CommonFilter<T> {}

impl Filterable for AndFilter {}
impl<T: CommonFilterKind> Filterable for OrFilter<T> {}
impl<T: CommonFilterKind> Filterable for CommonFilter<T> {}
impl Filterable for EmptyFilter {}

// todo: add a heckton of tests.
