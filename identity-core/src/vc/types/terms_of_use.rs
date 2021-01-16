use crate::common::{Object, OneOrMany, Url};

/// Information used to express obligations, prohibitions, and permissions about
/// a `Credential` or `Presentation`.
///
/// [More Info](https://www.w3.org/TR/vc-data-model/#terms-of-use)
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct TermsOfUse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Url>,
    #[serde(rename = "type")]
    pub types: OneOrMany<String>,
    #[serde(flatten)]
    pub properties: Object,
}

impl TermsOfUse {
    pub fn new<T>(types: T) -> Self
    where
        T: Into<OneOrMany<String>>,
    {
        Self {
            id: None,
            types: types.into(),
            properties: Object::new(),
        }
    }

    pub fn with_id<T, U>(types: T, id: Url) -> Self
    where
        T: Into<OneOrMany<String>>,
    {
        Self {
            id: Some(id),
            types: types.into(),
            properties: Object::new(),
        }
    }

    pub fn with_properties<T, U>(types: T, properties: Object) -> Self
    where
        T: Into<OneOrMany<String>>,
    {
        Self {
            id: None,
            types: types.into(),
            properties,
        }
    }

    pub fn with_id_and_properties<T, U, V>(types: T, id: Url, properties: Object) -> Self
    where
        T: Into<OneOrMany<String>>,
    {
        Self {
            id: Some(id),
            types: types.into(),
            properties,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{convert::FromJson as _, vc::TermsOfUse};

    const JSON1: &str = include_str!("../../../tests/fixtures/vc/terms-of-use-1.json");
    const JSON2: &str = include_str!("../../../tests/fixtures/vc/terms-of-use-2.json");

    #[test]
    #[rustfmt::skip]
    fn test_from_json() {
        let policy: TermsOfUse = TermsOfUse::from_json(JSON1).unwrap();
        assert_eq!(policy.id.unwrap(), "http://example.com/policies/credential/4");
        assert_eq!(policy.types.as_slice(), ["IssuerPolicy"]);
        assert_eq!(policy.properties["profile"], "http://example.com/profiles/credential");
        assert_eq!(policy.properties["prohibition"][0]["assigner"], "https://example.edu/issuers/14");
        assert_eq!(policy.properties["prohibition"][0]["assignee"], "AllVerifiers");
        assert_eq!(policy.properties["prohibition"][0]["target"], "http://example.edu/credentials/3732");
        assert_eq!(policy.properties["prohibition"][0]["action"][0], "Archival");


        let policy: TermsOfUse = TermsOfUse::from_json(JSON2).unwrap();
        assert_eq!(policy.id.unwrap(), "http://example.com/policies/credential/6");
        assert_eq!(policy.types.as_slice(), ["HolderPolicy"]);
        assert_eq!(policy.properties["profile"], "http://example.com/profiles/credential");
        assert_eq!(policy.properties["prohibition"][0]["assigner"], "did:example:ebfeb1f712ebc6f1c276e12ec21");
        assert_eq!(policy.properties["prohibition"][0]["assignee"], "https://wineonline.example.org/");
        assert_eq!(policy.properties["prohibition"][0]["target"], "http://example.edu/credentials/3732");
        assert_eq!(policy.properties["prohibition"][0]["action"][0], "3rdPartyCorrelation");
    }
}
