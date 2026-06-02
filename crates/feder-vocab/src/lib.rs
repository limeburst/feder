//! Minimal Activity Vocabulary types for Feder.
//!
//! This crate models ActivityPub/ActivityStreams protocol data only. It does
//! not fetch remote objects, read or write storage, deliver activities, or own
//! core decision logic.

use serde::{Deserialize, Serialize};

/// The canonical Activity Streams JSON-LD context URL.
pub const ACTIVITYSTREAMS_CONTEXT: &str = "https://www.w3.org/ns/activitystreams";

/// An absolute ActivityPub/ActivityStreams identifier.
pub type Iri = String;

/// A non-scalar ActivityStreams property value.
///
/// ActivityStreams object slots can contain either an embedded object or the
/// object's IRI. Phase 1 keeps both forms explicit and avoids dereferencing.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Reference<T> {
    Id(Iri),
    Object(Box<T>),
}

impl<T> Reference<T> {
    #[must_use]
    pub fn id(id: impl Into<Iri>) -> Self {
        Self::Id(id.into())
    }

    #[must_use]
    pub fn object(object: T) -> Self {
        Self::Object(Box::new(object))
    }
}

/// A property value that can appear either once or multiple times.
///
/// ActivityStreams commonly allows fields to be absent, scalar, or arrays.
/// Absence is represented by `Option<OneOrMany<T>>` on the containing type.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> OneOrMany<T> {
    #[must_use]
    pub fn one(value: T) -> Self {
        Self::One(value)
    }

    #[must_use]
    pub fn many(values: impl Into<Vec<T>>) -> Self {
        Self::Many(values.into())
    }
}

macro_rules! activitystreams_type {
    ($name:ident, $variant:ident) => {
        #[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
        pub enum $name {
            #[default]
            $variant,
        }
    };
}

activitystreams_type!(PersonType, Person);
activitystreams_type!(NoteType, Note);
activitystreams_type!(FollowType, Follow);
activitystreams_type!(AcceptType, Accept);
activitystreams_type!(CreateType, Create);

/// A minimal ActivityPub actor for Phase 1 core tests.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Actor {
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<Iri>,
    #[serde(rename = "type")]
    pub kind: PersonType,
    pub id: Iri,
    pub inbox: Iri,
    pub outbox: Iri,
    #[serde(rename = "preferredUsername", skip_serializing_if = "Option::is_none")]
    pub preferred_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl Actor {
    #[must_use]
    pub fn person(id: impl Into<Iri>, inbox: impl Into<Iri>, outbox: impl Into<Iri>) -> Self {
        Self {
            context: Some(ACTIVITYSTREAMS_CONTEXT.to_string()),
            kind: PersonType::default(),
            id: id.into(),
            inbox: inbox.into(),
            outbox: outbox.into(),
            preferred_username: None,
            name: None,
        }
    }
}

/// A minimal ActivityStreams Note object.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Note {
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<Iri>,
    #[serde(rename = "type")]
    pub kind: NoteType,
    pub id: Iri,
    #[serde(rename = "attributedTo", skip_serializing_if = "Option::is_none")]
    pub attributed_to: Option<Reference<Actor>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<String>,
}

impl Note {
    #[must_use]
    pub fn new(id: impl Into<Iri>) -> Self {
        Self {
            context: Some(ACTIVITYSTREAMS_CONTEXT.to_string()),
            kind: NoteType::default(),
            id: id.into(),
            attributed_to: None,
            content: None,
            published: None,
        }
    }
}

/// A minimal Follow activity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Follow {
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<Iri>,
    #[serde(rename = "type")]
    pub kind: FollowType,
    pub id: Iri,
    pub actor: Reference<Actor>,
    pub object: Reference<Actor>,
}

impl Follow {
    #[must_use]
    pub fn new(id: impl Into<Iri>, actor: Reference<Actor>, object: Reference<Actor>) -> Self {
        Self {
            context: Some(ACTIVITYSTREAMS_CONTEXT.to_string()),
            kind: FollowType::default(),
            id: id.into(),
            actor,
            object,
        }
    }
}

/// A minimal Accept activity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Accept {
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<Iri>,
    #[serde(rename = "type")]
    pub kind: AcceptType,
    pub id: Iri,
    pub actor: Reference<Actor>,
    pub object: Reference<Follow>,
}

impl Accept {
    #[must_use]
    pub fn new(id: impl Into<Iri>, actor: Reference<Actor>, object: Reference<Follow>) -> Self {
        Self {
            context: Some(ACTIVITYSTREAMS_CONTEXT.to_string()),
            kind: AcceptType::default(),
            id: id.into(),
            actor,
            object,
        }
    }
}

/// A minimal Create activity for a concrete object type.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Create<T> {
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    pub context: Option<Iri>,
    #[serde(rename = "type")]
    pub kind: CreateType,
    pub id: Iri,
    pub actor: Reference<Actor>,
    pub object: Reference<T>,
}

impl<T> Create<T> {
    #[must_use]
    pub fn new(id: impl Into<Iri>, actor: Reference<Actor>, object: Reference<T>) -> Self {
        Self {
            context: Some(ACTIVITYSTREAMS_CONTEXT.to_string()),
            kind: CreateType::default(),
            id: id.into(),
            actor,
            object,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::DeserializeOwned;
    use serde_json::json;

    fn roundtrip<T>(value: &T) -> T
    where
        T: DeserializeOwned + Serialize,
    {
        let json = serde_json::to_string(value).expect("serialize activitystreams value");
        serde_json::from_str(&json).expect("deserialize activitystreams value")
    }

    #[test]
    fn actor_roundtrips_json() {
        let mut actor = Actor::person(
            "https://example.com/users/alice",
            "https://example.com/users/alice/inbox",
            "https://example.com/users/alice/outbox",
        );
        actor.preferred_username = Some("alice".to_string());
        actor.name = Some("Alice".to_string());

        assert_eq!(roundtrip(&actor), actor);
    }

    #[test]
    fn actor_deserializes_basic_activitypub_json() {
        let actor: Actor = serde_json::from_value(json!({
            "@context": ACTIVITYSTREAMS_CONTEXT,
            "type": "Person",
            "id": "https://example.com/users/alice",
            "inbox": "https://example.com/users/alice/inbox",
            "outbox": "https://example.com/users/alice/outbox",
            "preferredUsername": "alice",
            "name": "Alice"
        }))
        .expect("deserialize actor from json");

        assert_eq!(actor.id, "https://example.com/users/alice");
        assert_eq!(actor.preferred_username, Some("alice".to_string()));
    }

    #[test]
    fn follow_and_accept_roundtrip_json() {
        let follow = Follow::new(
            "https://remote.example/activities/follow/1",
            Reference::id("https://remote.example/users/bob"),
            Reference::id("https://example.com/users/alice"),
        );
        let accept = Accept::new(
            "https://example.com/activities/accept/1",
            Reference::id("https://example.com/users/alice"),
            Reference::object(follow),
        );

        assert_eq!(roundtrip(&accept), accept);
    }

    #[test]
    fn create_note_roundtrips_json() {
        let mut note = Note::new("https://example.com/notes/1");
        note.attributed_to = Some(Reference::id("https://example.com/users/alice"));
        note.content = Some("Hello, fediverse.".to_string());
        note.published = Some("2026-05-29T06:30:00Z".to_string());

        let create = Create::new(
            "https://example.com/activities/create/1",
            Reference::id("https://example.com/users/alice"),
            Reference::object(note),
        );

        assert_eq!(roundtrip(&create), create);
    }

    #[test]
    fn concrete_types_reject_wrong_activitystreams_type() {
        let result = serde_json::from_value::<Follow>(json!({
            "type": "Accept",
            "id": "https://remote.example/activities/follow/1",
            "actor": "https://remote.example/users/bob",
            "object": "https://example.com/users/alice"
        }));

        assert!(result.is_err());
    }

    #[test]
    fn one_or_many_deserializes_scalar_and_array() {
        let one: OneOrMany<Iri> = serde_json::from_value(json!("https://example.com/users/alice"))
            .expect("deserialize scalar one-or-many value");
        let many: OneOrMany<Iri> = serde_json::from_value(json!([
            "https://example.com/users/alice",
            "https://example.com/users/bob"
        ]))
        .expect("deserialize array one-or-many value");

        assert_eq!(
            one,
            OneOrMany::one("https://example.com/users/alice".to_string())
        );
        assert_eq!(
            many,
            OneOrMany::many([
                "https://example.com/users/alice".to_string(),
                "https://example.com/users/bob".to_string()
            ])
        );
    }
}
