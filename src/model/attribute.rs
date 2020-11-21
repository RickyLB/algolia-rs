use std::fmt::{self, Display};

// an attribute is `[A-Za-z0-9\. ]+` presumably?
#[derive(Debug, Clone)]
pub struct Attribute(pub String);

#[derive(Debug)]
struct SearchableAttribue {
    unordered: bool,
    // note: all of these share the same priority
    attributes: Vec<Attribute>,
}

impl Display for SearchableAttribue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.unordered {
            write!(f, "unordered(")?;
        }

        let mut attributes = self.attributes.iter();

        if let Some(attr) = attributes.next() {
            f.write_str(&attr.0)?;
        }

        for attr in attributes {
            write!(f, ",")?;
            f.write_str(&attr.0)?;
        }

        if self.unordered {
            write!(f, ")")?;
        }

        Ok(())
    }
}

#[derive(serde::Serialize, Debug)]
pub struct SearchableAttributes(Vec<SearchableAttribue>);

impl SearchableAttributes {
    pub fn build() -> SearchableAttributesBuilder {
        SearchableAttributesBuilder { attrs: vec![] }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

pub struct SearchableAttributesBuilder {
    attrs: Vec<SearchableAttribue>,
}

impl SearchableAttributesBuilder {
    pub fn single_with_order(mut self, attr: Attribute, ordered: bool) -> Self {
        self.attrs.push(SearchableAttribue {
            attributes: vec![attr],
            unordered: !ordered,
        });

        self
    }

    pub fn single(self, attr: Attribute) -> Self {
        self.single_with_order(attr, true)
    }

    pub fn single_unordered(self, attr: Attribute) -> Self {
        self.single_with_order(attr, false)
    }

    pub fn multi_with_order(mut self, attrs: Vec<Attribute>, ordered: bool) -> Self {
        // todo: error if attrs.is_empty (or just skip it in serialization?)

        self.attrs.push(SearchableAttribue {
            attributes: attrs,
            unordered: !ordered,
        });

        self
    }

    pub fn multi(self, attrs: Vec<Attribute>) -> Self {
        self.multi_with_order(attrs, true)
    }

    pub fn multi_unordered(self, attrs: Vec<Attribute>) -> Self {
        self.multi_with_order(attrs, false)
    }

    pub fn finish(self) -> SearchableAttributes {
        SearchableAttributes(self.attrs)
    }
}

impl serde::Serialize for SearchableAttribue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(test)]
mod test {
    use super::{Attribute, SearchableAttributes};

    #[test]
    fn list_of_attributes() {
        insta::assert_json_snapshot!(SearchableAttributes::build()
            .single(Attribute("a".to_owned()))
            .multi(vec![Attribute("b".to_owned()), Attribute("c".to_owned())])
            .single_unordered(Attribute("e".to_owned()))
            .multi_unordered(vec![Attribute("f".to_owned()), Attribute("g".to_owned())])
            .finish())
    }
}
