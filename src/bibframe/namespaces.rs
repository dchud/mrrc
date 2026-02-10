//! BIBFRAME namespace definitions and constants.
//!
//! This module defines the RDF namespace prefixes used in BIBFRAME conversion,
//! following the Library of Congress BIBFRAME 2.0 vocabulary.

/// BIBFRAME 2.0 namespace URI.
pub const BF: &str = "http://id.loc.gov/ontologies/bibframe/";

/// BIBFRAME Library of Congress extensions namespace URI.
pub const BFLC: &str = "http://id.loc.gov/ontologies/bflc/";

/// MADS/RDF namespace (Metadata Authority Description Schema).
pub const MADSRDF: &str = "http://www.loc.gov/mads/rdf/v1#";

/// RDF namespace.
pub const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";

/// RDF Schema namespace.
pub const RDFS: &str = "http://www.w3.org/2000/01/rdf-schema#";

/// XML Schema namespace.
pub const XSD: &str = "http://www.w3.org/2001/XMLSchema#";

/// LOC relators vocabulary namespace.
pub const RELATORS: &str = "http://id.loc.gov/vocabulary/relators/";

/// LOC languages vocabulary namespace.
pub const LANGUAGES: &str = "http://id.loc.gov/vocabulary/languages/";

/// LOC countries vocabulary namespace.
pub const COUNTRIES: &str = "http://id.loc.gov/vocabulary/countries/";

/// LOC content types vocabulary namespace.
pub const CONTENT_TYPES: &str = "http://id.loc.gov/vocabulary/contentTypes/";

/// LOC media types vocabulary namespace.
pub const MEDIA_TYPES: &str = "http://id.loc.gov/vocabulary/mediaTypes/";

/// LOC carrier types vocabulary namespace.
pub const CARRIER_TYPES: &str = "http://id.loc.gov/vocabulary/carriers/";

/// LOC names authority namespace.
pub const LC_NAMES: &str = "http://id.loc.gov/authorities/names/";

/// LOC subjects authority namespace.
pub const LC_SUBJECTS: &str = "http://id.loc.gov/authorities/subjects/";

/// Common BIBFRAME class local names.
pub mod classes {
    /// Work - the conceptual essence of a resource.
    pub const WORK: &str = "Work";
    /// Instance - a material embodiment of a Work.
    pub const INSTANCE: &str = "Instance";
    /// Item - a specific copy of an Instance.
    pub const ITEM: &str = "Item";
    /// Hub - groups related expressions (translations, versions).
    pub const HUB: &str = "Hub";

    // Work types
    /// Text work type.
    pub const TEXT: &str = "Text";
    /// `NotatedMusic` work type.
    pub const NOTATED_MUSIC: &str = "NotatedMusic";
    /// Cartography work type.
    pub const CARTOGRAPHY: &str = "Cartography";
    /// `MovingImage` work type.
    pub const MOVING_IMAGE: &str = "MovingImage";
    /// `StillImage` work type.
    pub const STILL_IMAGE: &str = "StillImage";
    /// Audio work type.
    pub const AUDIO: &str = "Audio";
    /// `MusicAudio` work type.
    pub const MUSIC_AUDIO: &str = "MusicAudio";
    /// Multimedia work type.
    pub const MULTIMEDIA: &str = "Multimedia";
    /// `MixedMaterial` work type.
    pub const MIXED_MATERIAL: &str = "MixedMaterial";
    /// Object work type (3D artifact).
    pub const OBJECT: &str = "Object";
    /// Kit work type.
    pub const KIT: &str = "Kit";

    // Instance types
    /// Serial instance type.
    pub const SERIAL: &str = "Serial";
    /// Manuscript instance type.
    pub const MANUSCRIPT: &str = "Manuscript";
    /// Electronic instance type.
    pub const ELECTRONIC: &str = "Electronic";
    /// Print instance type.
    pub const PRINT: &str = "Print";

    // Agent types
    /// Person agent type.
    pub const PERSON: &str = "Person";
    /// Organization agent type.
    pub const ORGANIZATION: &str = "Organization";
    /// Meeting agent type.
    pub const MEETING: &str = "Meeting";
    /// Family agent type.
    pub const FAMILY: &str = "Family";
    /// Jurisdiction agent type.
    pub const JURISDICTION: &str = "Jurisdiction";

    // Subject types
    /// Topic subject type.
    pub const TOPIC: &str = "Topic";
    /// Place subject/geographic type.
    pub const PLACE: &str = "Place";
    /// Temporal subject type.
    pub const TEMPORAL: &str = "Temporal";
    /// `GenreForm` type.
    pub const GENRE_FORM: &str = "GenreForm";

    // Other classes
    /// Title class.
    pub const TITLE: &str = "Title";
    /// Contribution class.
    pub const CONTRIBUTION: &str = "Contribution";
    /// Publication provision activity.
    pub const PUBLICATION: &str = "Publication";
    /// Production provision activity.
    pub const PRODUCTION: &str = "Production";
    /// Distribution provision activity.
    pub const DISTRIBUTION: &str = "Distribution";
    /// Manufacture provision activity.
    pub const MANUFACTURE: &str = "Manufacture";
    /// `AdminMetadata` class.
    pub const ADMIN_METADATA: &str = "AdminMetadata";

    // Identifier types
    /// ISBN identifier type.
    pub const ISBN: &str = "Isbn";
    /// ISSN identifier type.
    pub const ISSN: &str = "Issn";
    /// LCCN identifier type.
    pub const LCCN: &str = "Lccn";
    /// Local identifier type.
    pub const LOCAL: &str = "Local";

    // Classification types
    /// Classification (general).
    pub const CLASSIFICATION: &str = "Classification";
    /// `ClassificationLcc` - Library of Congress Classification.
    pub const CLASSIFICATION_LCC: &str = "ClassificationLcc";
    /// `ClassificationDdc` - Dewey Decimal Classification.
    pub const CLASSIFICATION_DDC: &str = "ClassificationDdc";
    /// `ClassificationNlm` - National Library of Medicine Classification.
    pub const CLASSIFICATION_NLM: &str = "ClassificationNlm";
    /// `ClassificationUdc` - Universal Decimal Classification.
    pub const CLASSIFICATION_UDC: &str = "ClassificationUdc";
}

/// Common BIBFRAME property local names.
pub mod properties {
    // Core relationships
    /// hasInstance - links Work to Instance (or Hub to Instance when Hub present).
    pub const HAS_INSTANCE: &str = "hasInstance";
    /// instanceOf - links Instance to Work (or Instance to Hub when Hub present).
    pub const INSTANCE_OF: &str = "instanceOf";
    /// hasExpression - links Work to Hub (expression-level grouping).
    pub const HAS_EXPRESSION: &str = "hasExpression";
    /// expressionOf - links Hub to Work.
    pub const EXPRESSION_OF: &str = "expressionOf";
    /// hasItem - links Instance to Item.
    pub const HAS_ITEM: &str = "hasItem";
    /// itemOf - links Item to Instance.
    pub const ITEM_OF: &str = "itemOf";

    // Title properties
    /// title - general title property.
    pub const TITLE: &str = "title";
    /// mainTitle - primary title text.
    pub const MAIN_TITLE: &str = "mainTitle";
    /// subtitle - subordinate title text.
    pub const SUBTITLE: &str = "subtitle";
    /// partName - name of part.
    pub const PART_NAME: &str = "partName";
    /// partNumber - number of part.
    pub const PART_NUMBER: &str = "partNumber";

    // Contribution properties
    /// contribution - links to Contribution.
    pub const CONTRIBUTION: &str = "contribution";
    /// agent - agent of contribution.
    pub const AGENT: &str = "agent";
    /// role - relator role.
    pub const ROLE: &str = "role";

    // Subject properties
    /// subject - general subject property.
    pub const SUBJECT: &str = "subject";

    // Provision activity properties
    /// `provisionActivity` - links to `ProvisionActivity`.
    pub const PROVISION_ACTIVITY: &str = "provisionActivity";
    /// place - place of provision activity.
    pub const PLACE: &str = "place";
    /// date - date of provision activity.
    pub const DATE: &str = "date";
    /// copyrightDate - copyright date.
    pub const COPYRIGHT_DATE: &str = "copyrightDate";

    // Identifier properties
    /// identifiedBy - links to Identifier.
    pub const IDENTIFIED_BY: &str = "identifiedBy";

    // Description properties
    /// responsibilityStatement - statement of responsibility.
    pub const RESPONSIBILITY_STATEMENT: &str = "responsibilityStatement";
    /// editionStatement - edition statement.
    pub const EDITION_STATEMENT: &str = "editionStatement";
    /// extent - physical extent.
    pub const EXTENT: &str = "extent";
    /// dimensions - physical dimensions.
    pub const DIMENSIONS: &str = "dimensions";
    /// classification - links to Classification.
    pub const CLASSIFICATION: &str = "classification";
    /// `classificationPortion` - classification number portion.
    pub const CLASSIFICATION_PORTION: &str = "classificationPortion";
    /// itemPortion - item number/cutter portion.
    pub const ITEM_PORTION: &str = "itemPortion";
    /// note - general note.
    pub const NOTE: &str = "note";
    /// summary - summary of content.
    pub const SUMMARY: &str = "summary";

    // Administrative properties
    /// `adminMetadata` - links to `AdminMetadata`.
    pub const ADMIN_METADATA: &str = "adminMetadata";
    /// creationDate - date record created.
    pub const CREATION_DATE: &str = "creationDate";
    /// changeDate - date record changed.
    pub const CHANGE_DATE: &str = "changeDate";
    /// source - source of record.
    pub const SOURCE: &str = "source";

    // Content/media/carrier
    /// content - content type.
    pub const CONTENT: &str = "content";
    /// media - media type.
    pub const MEDIA: &str = "media";
    /// carrier - carrier type.
    pub const CARRIER: &str = "carrier";
}

/// BFLC extension property local names.
pub mod bflc {
    /// Authorized access point (concatenated name string).
    pub const AAP: &str = "aap";
    /// Primary contribution marker.
    pub const PRIMARY_CONTRIBUTION: &str = "PrimaryContribution";
    /// Encoding level.
    pub const ENCODING_LEVEL: &str = "encodingLevel";
    /// Simple place (transcribed, not parsed).
    pub const SIMPLE_PLACE: &str = "simplePlace";
    /// Simple date (transcribed, not parsed).
    pub const SIMPLE_DATE: &str = "simpleDate";
    /// Simple agent (transcribed, not parsed).
    pub const SIMPLE_AGENT: &str = "simpleAgent";
    /// MARC key for round-trip preservation.
    pub const MARC_KEY: &str = "marcKey";
    /// Series treatment.
    pub const SERIES_TREATMENT: &str = "SeriesTreatment";
    /// Applicable institution.
    pub const APPLICABLE_INSTITUTION: &str = "applicableInstitution";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_uris() {
        assert!(BF.starts_with("http://id.loc.gov/"));
        assert!(BFLC.starts_with("http://id.loc.gov/"));
        assert!(RDF.starts_with("http://www.w3.org/"));
        assert!(RDFS.starts_with("http://www.w3.org/"));
    }

    #[test]
    fn test_namespace_trailing_delimiter() {
        // BF and BFLC use trailing slash
        assert!(BF.ends_with('/'));
        assert!(BFLC.ends_with('/'));
        // MADSRDF uses hash
        assert!(MADSRDF.ends_with('#'));
        // RDF/RDFS use hash
        assert!(RDF.ends_with('#'));
        assert!(RDFS.ends_with('#'));
    }
}
