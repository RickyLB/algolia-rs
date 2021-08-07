use sealed::Sealed;
use std::fmt::Display;

mod sealed {
    pub trait Sealed {}
}

pub trait CommonFilterKind: Display + Sealed {}
pub trait AndFilterable: Display + Sealed {}
pub trait Filterable: Display + Sealed {}

// todo: consider making a Filter DSL

macro_rules! make_number_ty {
    ($number:ident; $( $( #[cfg($attrs:meta)] )? $num:ident($t:ty) ),* $(,)? ) => {
        #[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
        pub enum $number {
            $(
                $( #[cfg($attrs)] )?
                $num($t),
            )*
        }

        impl std::fmt::Display for $number {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match *self {
                    $(
                        $( #[cfg($attrs)] )?
                        Self::$num(num) => num.fmt(f),
                    )*
                }
            }
        }

        $(
            $( #[cfg($attrs)])?
            impl std::convert::From<$t> for $number {
                #[inline(always)]
                fn from(t: $t) -> Self {
                    Self::$num(t)
                }
            }
        )*
    };
}

make_number_ty!(Number;
    U8(u8),
    U16(u16),
    U32(u32),
    #[cfg(not(target_pointer_width = "64"))]
    Usize(usize),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    Isize(isize),
    F32(f32),
    F64(f64),
);

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

/// Scored facet filtering. Is *not* `AndFilterable`, see algolia docs:
/// https://www.algolia.com/doc/guides/managing-results/refine-results/filtering/in-depth/filter-scoring/
pub struct ScoredFacetFilter {
    pub facet_name: String,
    pub value: String,
    // NOTE (accurate as of 2021/08/06):
    //
    // `score` must be in range 0...i64::MAX.
    // Queries will return with 400 BAD_REQUEST if given any of "-123", "-0", or i64::MAX+1.
    pub score: i64,
}

impl Display for ScoredFacetFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.score > i64::MAX {
            return Err(std::fmt::Error);
        }

        write!(
            f,
            r#""{}":"{}"<score={}>"#,
            self.facet_name.escape_debug(),
            self.value.escape_debug(),
            self.score,
        )
    }
}

pub struct RangeFilter {
    pub attribute_name: String,
    pub lower_bound: Number,
    pub upper_bound: Number,
}

impl RangeFilter {
    pub fn new<T: Into<Number>>(attribute_name: String, lower_bound: T, upper_bound: T) -> Self {
        Self {
            attribute_name,
            lower_bound: lower_bound.into(),
            upper_bound: upper_bound.into(),
        }
    }
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
    pub value: Number,
}

impl CmpFilter {
    pub fn new<T: Into<Number>>(
        attribute_name: String,
        operator: FilterOperator,
        value: T,
    ) -> Self {
        Self {
            attribute_name,
            operator,
            value: value.into(),
        }
    }
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

#[derive(Default)]
pub struct OrFilter<T: CommonFilterKind> {
    pub filters: Vec<CommonFilter<T>>,
}

impl<T: CommonFilterKind> Display for OrFilter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Separated(&self.filters, " OR ").fmt(f)
    }
}

#[derive(Default)]
pub struct AndFilter {
    pub filters: Vec<Box<dyn AndFilterable>>,
}

impl Display for AndFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Separated(&self.filters, " AND ").fmt(f)
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct EmptyFilter;

impl Display for EmptyFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}
macro_rules! mark {
    ($mark:ident; $( $t:ty ),+ $(,)? ) => {
        $(
            impl $mark for $t {}
        )+
    };
}

mark!(Sealed; BooleanFilter, TagFilter, FacetFilter, ScoredFacetFilter, RangeFilter, CmpFilter, AndFilter, EmptyFilter);
mark!(CommonFilterKind; BooleanFilter, TagFilter, FacetFilter, ScoredFacetFilter, RangeFilter, CmpFilter);

impl<T: CommonFilterKind> Sealed for CommonFilter<T> {}
impl<T: CommonFilterKind> Sealed for OrFilter<T> {}

impl<T: CommonFilterKind> AndFilterable for OrFilter<T> {}
impl<T: CommonFilterKind> AndFilterable for CommonFilter<T> {}

impl Filterable for AndFilter {}
impl<T: CommonFilterKind> Filterable for OrFilter<T> {}
impl<T: CommonFilterKind> Filterable for CommonFilter<T> {}
impl Filterable for EmptyFilter {}

// todo: add a heckton of tests.
