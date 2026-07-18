use std::collections::BTreeSet;

use action_definition::{
    TranslationKey,
    actions::ACTION_DEFINITIONS,
    parameters::{Parameter, ParameterKind},
};
use fluent::{FluentBundle, FluentResource};
use fluent_syntax::ast;
use unic_langid::LanguageIdentifier;

fn bundle() -> FluentBundle<FluentResource> {
    let ftl = "
action-message-box-title =
    .name = Title
    .description = The title of the message box

enum-message-box-buttons =
    .ok = OK
    .ok-cancel = OK / Cancel

action-code = Run code
";

    let resource = match FluentResource::try_new(ftl.to_owned()) {
        Ok(resource) => resource,
        Err((_, errors)) => panic!("invalid FTL: {errors:?}"),
    };

    let langid: LanguageIdentifier = "en-US".parse().expect("valid English language tag");
    let mut bundle = FluentBundle::new(vec![langid]);
    bundle.add_resource(resource).expect("add resource");
    bundle
}

fn english_resource() -> FluentResource {
    let ftl = include_str!("../locales/en-US.ftl");
    match FluentResource::try_new(ftl.to_owned()) {
        Ok(resource) => resource,
        Err((_, errors)) => panic!("invalid English FTL: {errors:?}"),
    }
}

fn english_bundle() -> FluentBundle<FluentResource> {
    let resource = english_resource();

    let langid: LanguageIdentifier = "en-US".parse().expect("valid English language tag");
    let mut bundle = FluentBundle::new(vec![langid]);
    bundle.add_resource(resource).expect("add English resource");
    bundle
}

fn parameter_translation_keys(parameter: &Parameter) -> Vec<TranslationKey> {
    let mut keys = vec![parameter.name, parameter.description];

    if let ParameterKind::Enum(enum_parameter) = &parameter.kind {
        keys.extend(enum_parameter.variants.iter().map(|variant| variant.name));
    }

    keys
}

fn expected_action_metadata_keys() -> BTreeSet<String> {
    ACTION_DEFINITIONS
        .iter()
        .flat_map(|action| {
            [action.name, action.description].into_iter().chain(
                action
                    .parameters
                    .iter()
                    .flat_map(parameter_translation_keys),
            )
        })
        .map(display_key)
        .collect()
}

fn resource_keys(resource: &FluentResource) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();

    for entry in resource.entries() {
        let ast::Entry::Message(message) = entry else {
            continue;
        };

        if message.value.is_some() {
            assert!(
                keys.insert(message.id.name.to_owned()),
                "duplicate English translation for {}",
                message.id.name
            );
        }

        for attribute in &message.attributes {
            let key = format!("{}.{}", message.id.name, attribute.id.name);
            assert!(
                keys.insert(key.clone()),
                "duplicate English translation for {key}"
            );
        }
    }

    keys
}

fn display_key(key: TranslationKey) -> String {
    match key.attribute {
        Some(attribute) => format!("{}.{}", key.id, attribute),
        None => key.id.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_an_attribute() {
        let bundle = bundle();
        let key = TranslationKey::with_attribute("action-message-box-title", "name");
        assert_eq!(key.resolve(&bundle).as_deref(), Some("Title"));
    }

    #[test]
    fn resolves_a_hyphenated_attribute() {
        let bundle = bundle();
        let key = TranslationKey::with_attribute("enum-message-box-buttons", "ok-cancel");
        assert_eq!(key.resolve(&bundle).as_deref(), Some("OK / Cancel"));
    }

    #[test]
    fn resolves_a_message_without_attribute() {
        let bundle = bundle();
        let key = TranslationKey::new("action-code");
        assert_eq!(key.resolve(&bundle).as_deref(), Some("Run code"));
    }

    #[test]
    fn missing_attribute_resolves_to_none() {
        let bundle = bundle();
        let key = TranslationKey::with_attribute("action-message-box-title", "nope");
        assert_eq!(key.resolve(&bundle), None);
    }

    #[test]
    fn missing_message_resolves_to_none() {
        let bundle = bundle();
        let key = TranslationKey::with_attribute("does-not-exist", "name");
        assert_eq!(key.resolve(&bundle), None);
    }

    #[test]
    fn english_locale_contains_all_action_metadata_keys() {
        let bundle = english_bundle();

        for key in ACTION_DEFINITIONS.iter().flat_map(|action| {
            [action.name, action.description].into_iter().chain(
                action
                    .parameters
                    .iter()
                    .flat_map(parameter_translation_keys),
            )
        }) {
            let value = key
                .resolve(&bundle)
                .unwrap_or_else(|| panic!("missing English translation for {}", display_key(key)));

            assert!(
                !value.trim().is_empty(),
                "empty English translation for {}",
                display_key(key)
            );
        }
    }

    #[test]
    fn english_locale_has_exactly_the_action_metadata_keys() {
        let resource = english_resource();
        let expected = expected_action_metadata_keys();
        let actual = resource_keys(&resource);
        let missing = expected.difference(&actual).collect::<Vec<_>>();
        let unexpected = actual.difference(&expected).collect::<Vec<_>>();

        assert!(
            missing.is_empty() && unexpected.is_empty(),
            "English locale keys do not match action metadata.\nmissing: {missing:#?}\nunexpected: {unexpected:#?}",
        );
    }
}
