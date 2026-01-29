//! BIBFRAME to MARC conversion logic.
//!
//! This module implements the reverse conversion from BIBFRAME 2.0 RDF graphs
//! back to MARC bibliographic records. Note that some data loss is inherent
//! since BIBFRAME is semantically richer than MARC.

use std::collections::HashMap;

use crate::error::Result;
use crate::leader::Leader;
use crate::record::{Field, Record};

use super::namespaces::{classes, properties, BF, RDF, RDFS, RELATORS};
use super::rdf::{RdfGraph, RdfNode};

/// Converts a BIBFRAME RDF graph to a MARC record.
///
/// This is the main entry point for BIBFRAME→MARC conversion.
///
/// # Errors
///
/// Returns an error if the graph cannot be converted (e.g., missing Work entity).
/// Currently the implementation always succeeds, but the Result type is used for
/// API stability to allow future versions to return errors.
#[allow(clippy::unnecessary_wraps)]
pub fn convert_bibframe_to_marc(graph: &RdfGraph) -> Result<Record> {
    let converter = BibframeToMarcConverter::new(graph);
    Ok(converter.convert())
}

/// Information about data that could not be mapped to MARC.
///
/// This struct tracks conversion losses for diagnostic purposes.
/// It will be used in future work to report what data was lost during conversion.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct ConversionLoss {
    /// Properties that had no MARC equivalent.
    pub unmapped_properties: Vec<String>,
    /// Entities that were skipped.
    pub skipped_entities: Vec<String>,
}

/// Internal converter state.
struct BibframeToMarcConverter<'a> {
    graph: &'a RdfGraph,
    /// Index: subject -> list of (predicate, object) pairs
    subject_index: HashMap<String, Vec<(String, RdfNode)>>,
    /// The Work entity node (if found)
    work_node: Option<String>,
    /// The Instance entity node (if found)
    instance_node: Option<String>,
    /// Track unmapped data (for future diagnostic use)
    #[allow(dead_code)]
    loss: ConversionLoss,
}

impl<'a> BibframeToMarcConverter<'a> {
    fn new(graph: &'a RdfGraph) -> Self {
        let mut converter = Self {
            graph,
            subject_index: HashMap::new(),
            work_node: None,
            instance_node: None,
            loss: ConversionLoss::default(),
        };
        converter.build_index();
        converter.find_entities();
        converter
    }

    /// Build an index of triples by subject for efficient lookup.
    fn build_index(&mut self) {
        for triple in self.graph.triples() {
            let subject_key = node_to_key(&triple.subject);
            self.subject_index
                .entry(subject_key)
                .or_default()
                .push((triple.predicate.clone(), triple.object.clone()));
        }
    }

    /// Find Work and Instance entities in the graph.
    fn find_entities(&mut self) {
        let rdf_type = format!("{RDF}type");
        let work_type = format!("{BF}{}", classes::WORK);
        let instance_type = format!("{BF}{}", classes::INSTANCE);

        for triple in self.graph.triples() {
            if triple.predicate == rdf_type {
                if let RdfNode::Uri(ref type_uri) = triple.object {
                    // Check for Work (or Work subtypes like Text, NotatedMusic, etc.)
                    if (type_uri == &work_type || is_work_subtype(type_uri))
                        && self.work_node.is_none()
                    {
                        self.work_node = Some(node_to_key(&triple.subject));
                    }
                    // Check for Instance (or Instance subtypes)
                    if (type_uri == &instance_type || is_instance_subtype(type_uri))
                        && self.instance_node.is_none()
                    {
                        self.instance_node = Some(node_to_key(&triple.subject));
                    }
                }
            }
        }
    }

    fn convert(mut self) -> Record {
        // Create leader based on Work/Instance types
        let leader = self.create_leader();
        let mut record = Record::new(leader);

        // Extract control fields
        self.extract_control_fields(&mut record);

        // Extract data fields
        self.extract_titles(&mut record);
        self.extract_creators(&mut record);
        self.extract_contributors(&mut record);
        self.extract_subjects(&mut record);
        self.extract_identifiers(&mut record);
        self.extract_provision_activity(&mut record);
        self.extract_physical_description(&mut record);
        self.extract_notes(&mut record);

        // Edge case handling (uab.4.4)
        self.extract_series(&mut record);
        self.extract_linking_entries(&mut record);

        record
    }

    /// Creates a Leader based on Work and Instance types.
    fn create_leader(&self) -> Leader {
        let mut record_type = 'a'; // Default: language material
        let mut bib_level = 'm'; // Default: monograph

        // Determine record type from Work type
        if let Some(ref work_key) = self.work_node {
            if let Some(props) = self.subject_index.get(work_key) {
                for (pred, obj) in props {
                    if pred == &format!("{RDF}type") {
                        if let RdfNode::Uri(ref type_uri) = obj {
                            record_type = work_type_to_leader_06(type_uri);
                        }
                    }
                }
            }
        }

        // Determine bibliographic level from Instance type
        if let Some(ref instance_key) = self.instance_node {
            if let Some(props) = self.subject_index.get(instance_key) {
                for (pred, obj) in props {
                    if pred == &format!("{RDF}type") {
                        if let RdfNode::Uri(ref type_uri) = obj {
                            bib_level = instance_type_to_leader_07(type_uri);
                        }
                    }
                }
            }
        }

        Leader {
            record_length: 0, // Will be computed when serializing
            record_status: 'n',
            record_type,
            bibliographic_level: bib_level,
            control_record_type: ' ',
            character_coding: 'a', // UTF-8
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 0, // Will be computed when serializing
            encoding_level: ' ',
            cataloging_form: 'a',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        }
    }

    /// Extracts control fields (001, 008, etc.).
    fn extract_control_fields(&mut self, record: &mut Record) {
        // Try to extract control number from Instance identifiedBy
        if let Some(ref instance_key) = self.instance_node {
            if let Some(control_num) = self.find_control_number(instance_key) {
                record.add_control_field("001".to_string(), control_num);
            }
        }

        // Create minimal 008 field
        // Format: date entered (6) + type of date + dates + country + ...
        let field_008 = self.create_008_field();
        record.add_control_field("008".to_string(), field_008);
    }

    /// Finds control number from identifiers.
    fn find_control_number(&self, instance_key: &str) -> Option<String> {
        let identified_by = format!("{BF}{}", properties::IDENTIFIED_BY);

        if let Some(props) = self.subject_index.get(instance_key) {
            for (pred, obj) in props {
                if pred == &identified_by {
                    let id_key = node_to_key(obj);
                    if let Some(id_props) = self.subject_index.get(&id_key) {
                        // Check if it's an LCCN or Local identifier
                        let mut is_control_id = false;
                        let mut value = None;

                        for (id_pred, id_obj) in id_props {
                            if id_pred == &format!("{RDF}type") {
                                if let RdfNode::Uri(ref type_uri) = id_obj {
                                    if type_uri.contains("Lccn") || type_uri.contains("Local") {
                                        is_control_id = true;
                                    }
                                }
                            }
                            if id_pred == &format!("{RDF}value") {
                                if let RdfNode::Literal { value: ref v, .. } = id_obj {
                                    value = Some(v.clone());
                                }
                            }
                        }

                        if is_control_id {
                            if let Some(v) = value {
                                return Some(v);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Creates a minimal 008 field.
    fn create_008_field(&self) -> String {
        // Minimal 008: 40 characters
        // Positions: date entered (6) + type (1) + dates (8) + country (3) + ... + language (3) + ...
        let mut field = String::with_capacity(40);

        // Date entered (000000 = unknown)
        field.push_str("      ");

        // Type of date/publication status
        field.push('s');

        // Date 1 (extract from provision activity if available)
        let date1 = self
            .extract_publication_date()
            .unwrap_or_else(|| "    ".to_string());
        let date1_truncated = &date1[..date1.len().min(4)];
        for c in date1_truncated.chars() {
            field.push(c);
        }
        // Pad to 4 chars if needed
        for _ in date1_truncated.len()..4 {
            field.push(' ');
        }

        // Date 2
        field.push_str("    ");

        // Place of publication (3 chars)
        field.push_str("xx ");

        // Illustrations, etc. (4 chars for books)
        field.push_str("    ");

        // Target audience
        field.push(' ');

        // Form of item
        field.push(' ');

        // Nature of contents (4 chars)
        field.push_str("    ");

        // Government publication
        field.push(' ');

        // Conference publication
        field.push('0');

        // Festschrift
        field.push('0');

        // Index
        field.push('0');

        // Undefined
        field.push(' ');

        // Literary form
        field.push('0');

        // Biography
        field.push(' ');

        // Language (extract if available)
        field.push_str("eng");

        // Modified record
        field.push(' ');

        // Cataloging source
        field.push(' ');

        field
    }

    /// Extracts publication date from provision activity.
    fn extract_publication_date(&self) -> Option<String> {
        if let Some(ref instance_key) = self.instance_node {
            let prov_activity = format!("{BF}{}", properties::PROVISION_ACTIVITY);

            if let Some(props) = self.subject_index.get(instance_key) {
                for (pred, obj) in props {
                    if pred == &prov_activity {
                        let activity_key = node_to_key(obj);
                        if let Some(activity_props) = self.subject_index.get(&activity_key) {
                            for (act_pred, act_obj) in activity_props {
                                if act_pred == &format!("{BF}{}", properties::DATE) {
                                    if let RdfNode::Literal { value, .. } = act_obj {
                                        // Extract 4-digit year
                                        let year: String = value
                                            .chars()
                                            .filter(char::is_ascii_digit)
                                            .take(4)
                                            .collect();
                                        if year.len() == 4 {
                                            return Some(year);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Extracts titles to 245/246 fields.
    fn extract_titles(&mut self, record: &mut Record) {
        if let Some(ref instance_key) = self.instance_node {
            let title_prop = format!("{BF}{}", properties::TITLE);

            if let Some(props) = self.subject_index.get(instance_key) {
                let mut is_first = true;

                for (pred, obj) in props.clone() {
                    if pred == title_prop {
                        let title_key = node_to_key(&obj);
                        if let Some(title_props) = self.subject_index.get(&title_key) {
                            let tag = if is_first { "245" } else { "246" };
                            let field = self.create_title_field(tag, title_props);
                            record.add_field(field);
                            is_first = false;
                        }
                    }
                }

                // Also extract responsibility statement
                let resp_prop = format!("{BF}{}", properties::RESPONSIBILITY_STATEMENT);
                for (pred, obj) in props.clone() {
                    if pred == resp_prop {
                        if let RdfNode::Literal { value, .. } = obj {
                            // Add to existing 245 if present
                            if let Some(fields) = record.fields.get_mut("245") {
                                if let Some(field) = fields.first_mut() {
                                    field.add_subfield('c', value);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Creates a title field from title properties.
    #[allow(clippy::unused_self)]
    fn create_title_field(&self, tag: &str, props: &[(String, RdfNode)]) -> Field {
        let mut field = Field::new(tag.to_string(), '0', '0');

        for (pred, obj) in props {
            if let RdfNode::Literal { value, .. } = obj {
                if pred.ends_with("mainTitle") {
                    field.add_subfield('a', value.clone());
                } else if pred.ends_with("subtitle") {
                    field.add_subfield('b', value.clone());
                } else if pred.ends_with("partNumber") {
                    field.add_subfield('n', value.clone());
                } else if pred.ends_with("partName") {
                    field.add_subfield('p', value.clone());
                }
            }
        }

        field
    }

    /// Extracts creators to 1XX fields.
    fn extract_creators(&mut self, record: &mut Record) {
        if let Some(ref work_key) = self.work_node {
            let contribution_prop = format!("{BF}{}", properties::CONTRIBUTION);

            if let Some(props) = self.subject_index.get(work_key) {
                for (pred, obj) in props.clone() {
                    if pred == contribution_prop {
                        let contrib_key = node_to_key(&obj);
                        if let Some(contrib_props) = self.subject_index.get(&contrib_key) {
                            // Check if this is a primary contribution
                            let is_primary = contrib_props.iter().any(|(p, o)| {
                                p == &format!("{RDF}type") &&
                                matches!(o, RdfNode::Uri(u) if u.contains("PrimaryContribution"))
                            });

                            if is_primary {
                                if let Some(field) = self.create_agent_field(&contrib_key, "1") {
                                    record.add_field(field);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Extracts contributors to 7XX fields.
    fn extract_contributors(&mut self, record: &mut Record) {
        if let Some(ref work_key) = self.work_node {
            let contribution_prop = format!("{BF}{}", properties::CONTRIBUTION);

            if let Some(props) = self.subject_index.get(work_key) {
                for (pred, obj) in props.clone() {
                    if pred == contribution_prop {
                        let contrib_key = node_to_key(&obj);
                        if let Some(contrib_props) = self.subject_index.get(&contrib_key) {
                            // Check if this is NOT a primary contribution
                            let is_primary = contrib_props.iter().any(|(p, o)| {
                                p == &format!("{RDF}type") &&
                                matches!(o, RdfNode::Uri(u) if u.contains("PrimaryContribution"))
                            });

                            if !is_primary {
                                if let Some(field) = self.create_agent_field(&contrib_key, "7") {
                                    record.add_field(field);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Creates an agent field (1XX or 7XX) from a contribution.
    fn create_agent_field(&self, contrib_key: &str, prefix: &str) -> Option<Field> {
        let contrib_props = self.subject_index.get(contrib_key)?;

        // Find the agent
        let agent_prop = format!("{BF}{}", properties::AGENT);
        let agent_key = contrib_props
            .iter()
            .find(|(p, _)| p == &agent_prop)
            .map(|(_, o)| node_to_key(o))?;

        let agent_props = self.subject_index.get(&agent_key)?;

        // Determine agent type and tag
        let mut agent_type = "Person";
        for (pred, obj) in agent_props {
            if pred == &format!("{RDF}type") {
                if let RdfNode::Uri(type_uri) = obj {
                    if type_uri.contains("Organization") {
                        agent_type = "Organization";
                    } else if type_uri.contains("Meeting") {
                        agent_type = "Meeting";
                    }
                }
            }
        }

        let tag = match (prefix, agent_type) {
            ("1", "Person") => "100",
            ("1", "Organization") => "110",
            ("1", "Meeting") => "111",
            ("7", "Organization") => "710",
            ("7", "Meeting") => "711",
            // Default to 700 (Person) for prefix "7" or any other combination
            _ => "700",
        };

        let mut field = Field::new(tag.to_string(), '1', ' ');

        // Extract agent label
        for (pred, obj) in agent_props {
            if pred == &format!("{RDFS}label") {
                if let RdfNode::Literal { value, .. } = obj {
                    field.add_subfield('a', value.clone());
                }
            }
        }

        // Extract role
        let role_prop = format!("{BF}{}", properties::ROLE);
        for (pred, obj) in contrib_props {
            if pred == &role_prop {
                match obj {
                    RdfNode::Uri(uri) => {
                        // Extract relator code from URI
                        if uri.starts_with(RELATORS) {
                            let code = uri.strip_prefix(RELATORS).unwrap_or("");
                            if !code.is_empty() {
                                field.add_subfield('4', code.to_string());
                            }
                        }
                    },
                    RdfNode::Literal { value, .. } => {
                        field.add_subfield('e', value.clone());
                    },
                    RdfNode::BlankNode(_) => {},
                }
            }
        }

        Some(field)
    }

    /// Extracts subjects to 6XX fields.
    fn extract_subjects(&mut self, record: &mut Record) {
        if let Some(ref work_key) = self.work_node {
            let subject_prop = format!("{BF}{}", properties::SUBJECT);

            if let Some(props) = self.subject_index.get(work_key) {
                for (pred, obj) in props.clone() {
                    if pred == subject_prop {
                        let subject_key = node_to_key(&obj);
                        if let Some(field) = self.create_subject_field(&subject_key) {
                            record.add_field(field);
                        }
                    }
                }
            }
        }
    }

    /// Creates a subject field (6XX) from a subject entity.
    fn create_subject_field(&self, subject_key: &str) -> Option<Field> {
        let subject_props = self.subject_index.get(subject_key)?;

        // Determine subject type and tag
        let mut tag = "650"; // Default: topical
        for (pred, obj) in subject_props {
            if pred == &format!("{RDF}type") {
                if let RdfNode::Uri(type_uri) = obj {
                    tag = subject_type_to_tag(type_uri);
                }
            }
        }

        let mut field = Field::new(tag.to_string(), ' ', '0');

        // Extract label
        for (pred, obj) in subject_props {
            if pred == &format!("{RDFS}label") {
                if let RdfNode::Literal { value, .. } = obj {
                    // Split on "--" for subdivisions
                    let parts: Vec<&str> = value.split("--").collect();
                    if let Some(first) = parts.first() {
                        field.add_subfield('a', first.trim().to_string());
                    }
                    for part in parts.iter().skip(1) {
                        field.add_subfield('x', part.trim().to_string());
                    }
                }
            }
        }

        Some(field)
    }

    /// Extracts identifiers to 0XX fields.
    fn extract_identifiers(&mut self, record: &mut Record) {
        if let Some(ref instance_key) = self.instance_node {
            let identified_by = format!("{BF}{}", properties::IDENTIFIED_BY);

            if let Some(props) = self.subject_index.get(instance_key) {
                for (pred, obj) in props.clone() {
                    if pred == identified_by {
                        let id_key = node_to_key(&obj);
                        if let Some(field) = self.create_identifier_field(&id_key) {
                            record.add_field(field);
                        }
                    }
                }
            }
        }
    }

    /// Creates an identifier field (0XX) from an identifier entity.
    fn create_identifier_field(&self, id_key: &str) -> Option<Field> {
        let id_props = self.subject_index.get(id_key)?;

        // Determine identifier type and tag
        let mut tag = "035"; // Default: system control number
        for (pred, obj) in id_props {
            if pred == &format!("{RDF}type") {
                if let RdfNode::Uri(type_uri) = obj {
                    tag = identifier_type_to_tag(type_uri);
                }
            }
        }

        let mut field = Field::new(tag.to_string(), ' ', ' ');

        // Extract value
        for (pred, obj) in id_props {
            if pred == &format!("{RDF}value") {
                if let RdfNode::Literal { value, .. } = obj {
                    field.add_subfield('a', value.clone());
                }
            }
        }

        // Only return if we have a value
        if field.subfields.is_empty() {
            None
        } else {
            Some(field)
        }
    }

    /// Extracts provision activity to 260/264 fields.
    fn extract_provision_activity(&mut self, record: &mut Record) {
        if let Some(ref instance_key) = self.instance_node {
            let prov_activity = format!("{BF}{}", properties::PROVISION_ACTIVITY);

            if let Some(props) = self.subject_index.get(instance_key) {
                for (pred, obj) in props.clone() {
                    if pred == prov_activity {
                        let activity_key = node_to_key(&obj);
                        if let Some(field) = self.create_provision_field(&activity_key) {
                            record.add_field(field);
                        }
                    }
                }

                // Also check for copyright date
                let copyright_prop = format!("{BF}{}", properties::COPYRIGHT_DATE);
                for (pred, obj) in props.clone() {
                    if pred == copyright_prop {
                        if let RdfNode::Literal { value, .. } = obj {
                            let mut field = Field::new("264".to_string(), ' ', '4');
                            field.add_subfield('c', value);
                            record.add_field(field);
                        }
                    }
                }
            }
        }
    }

    /// Creates a provision activity field (264) from an activity entity.
    #[allow(clippy::cognitive_complexity)]
    fn create_provision_field(&self, activity_key: &str) -> Option<Field> {
        let activity_props = self.subject_index.get(activity_key)?;

        // Determine activity type for indicator
        let mut ind2 = '1'; // Default: publication
        for (pred, obj) in activity_props {
            if pred == &format!("{RDF}type") {
                if let RdfNode::Uri(type_uri) = obj {
                    ind2 = provision_type_to_indicator(type_uri);
                }
            }
        }

        let mut field = Field::new("264".to_string(), ' ', ind2);

        // Extract place, agent, date
        for (pred, obj) in activity_props {
            // Place - try simplePlace first, then nested Place
            if pred.ends_with("simplePlace") {
                if let RdfNode::Literal { value, .. } = obj {
                    field.add_subfield('a', value.clone());
                }
            } else if pred == &format!("{BF}{}", properties::PLACE) {
                let place_key = node_to_key(obj);
                if let Some(place_props) = self.subject_index.get(&place_key) {
                    for (place_pred, place_obj) in place_props {
                        if place_pred == &format!("{RDFS}label") {
                            if let RdfNode::Literal { value, .. } = place_obj {
                                // Only add if we don't already have a place
                                if !field.subfields.iter().any(|s| s.code == 'a') {
                                    field.add_subfield('a', value.clone());
                                }
                            }
                        }
                    }
                }
            }

            // Agent - try simpleAgent first
            if pred.ends_with("simpleAgent") {
                if let RdfNode::Literal { value, .. } = obj {
                    field.add_subfield('b', value.clone());
                }
            } else if pred == &format!("{BF}{}", properties::AGENT) {
                let agent_key = node_to_key(obj);
                if let Some(agent_props) = self.subject_index.get(&agent_key) {
                    for (agent_pred, agent_obj) in agent_props {
                        if agent_pred == &format!("{RDFS}label") {
                            if let RdfNode::Literal { value, .. } = agent_obj {
                                if !field.subfields.iter().any(|s| s.code == 'b') {
                                    field.add_subfield('b', value.clone());
                                }
                            }
                        }
                    }
                }
            }

            // Date - try simpleDate first
            if pred.ends_with("simpleDate") || pred == &format!("{BF}{}", properties::DATE) {
                if let RdfNode::Literal { value, .. } = obj {
                    if !field.subfields.iter().any(|s| s.code == 'c') {
                        field.add_subfield('c', value.clone());
                    }
                }
            }
        }

        // Only return if we have any subfields
        if field.subfields.is_empty() {
            None
        } else {
            Some(field)
        }
    }

    /// Extracts physical description to 300 field.
    fn extract_physical_description(&mut self, record: &mut Record) {
        if let Some(ref instance_key) = self.instance_node {
            let extent_prop = format!("{BF}{}", properties::EXTENT);
            let dimensions_prop = format!("{BF}{}", properties::DIMENSIONS);

            if let Some(props) = self.subject_index.get(instance_key) {
                let mut field = Field::new("300".to_string(), ' ', ' ');

                for (pred, obj) in props {
                    if pred == &extent_prop {
                        if let RdfNode::Literal { value, .. } = obj {
                            field.add_subfield('a', value.clone());
                        }
                    }
                    if pred == &dimensions_prop {
                        if let RdfNode::Literal { value, .. } = obj {
                            field.add_subfield('c', value.clone());
                        }
                    }
                }

                if !field.subfields.is_empty() {
                    record.add_field(field);
                }
            }
        }
    }

    /// Extracts notes to 5XX fields.
    fn extract_notes(&mut self, record: &mut Record) {
        if let Some(ref instance_key) = self.instance_node {
            let note_prop = format!("{BF}{}", properties::NOTE);
            let summary_prop = format!("{BF}{}", properties::SUMMARY);

            if let Some(props) = self.subject_index.get(instance_key) {
                for (pred, obj) in props {
                    if pred == &note_prop {
                        if let RdfNode::Literal { value, .. } = obj {
                            let mut field = Field::new("500".to_string(), ' ', ' ');
                            field.add_subfield('a', value.clone());
                            record.add_field(field);
                        }
                    }
                    if pred == &summary_prop {
                        if let RdfNode::Literal { value, .. } = obj {
                            let mut field = Field::new("520".to_string(), ' ', ' ');
                            field.add_subfield('a', value.clone());
                            record.add_field(field);
                        }
                    }
                }
            }
        }
    }

    // ========================================================================
    // Edge Case Extraction (mrrc-uab.4.4)
    // ========================================================================

    /// Extracts series to 490 and 8XX fields.
    #[allow(clippy::cognitive_complexity)]
    fn extract_series(&mut self, record: &mut Record) {
        if let Some(ref work_key) = self.work_node {
            let has_series_prop = format!("{BF}hasSeries");

            if let Some(props) = self.subject_index.get(work_key) {
                for (pred, obj) in props.clone() {
                    if pred == has_series_prop {
                        let series_key = node_to_key(&obj);
                        if let Some(series_props) = self.subject_index.get(&series_key) {
                            // Extract series title
                            let mut title = String::new();
                            for (series_pred, series_obj) in series_props {
                                // Check for title property
                                if series_pred.ends_with("title") {
                                    let title_key = node_to_key(series_obj);
                                    if let Some(title_props) = self.subject_index.get(&title_key) {
                                        for (t_pred, t_obj) in title_props {
                                            if t_pred.ends_with("mainTitle") {
                                                if let RdfNode::Literal { value, .. } = t_obj {
                                                    title.clone_from(value);
                                                }
                                            }
                                        }
                                    }
                                }
                                // Also check for rdfs:label
                                if series_pred == &format!("{RDFS}label") {
                                    if let RdfNode::Literal { value, .. } = series_obj {
                                        if title.is_empty() {
                                            title.clone_from(value);
                                        }
                                    }
                                }
                            }

                            if !title.is_empty() {
                                // Create 830 field (traced series)
                                let mut field = Field::new("830".to_string(), ' ', '0');
                                field.add_subfield('a', title);
                                record.add_field(field);
                            }
                        }
                    }
                }
            }
        }

        // Also extract series statements from Instance
        if let Some(ref instance_key) = self.instance_node {
            let series_stmt_prop = format!("{BF}seriesStatement");
            let series_enum_prop = format!("{BF}seriesEnumeration");

            if let Some(props) = self.subject_index.get(instance_key) {
                let mut series_statement = None;
                let mut enumeration = None;

                for (pred, obj) in props {
                    if pred == &series_stmt_prop {
                        if let RdfNode::Literal { value, .. } = obj {
                            series_statement = Some(value.clone());
                        }
                    }
                    if pred == &series_enum_prop {
                        if let RdfNode::Literal { value, .. } = obj {
                            enumeration = Some(value.clone());
                        }
                    }
                }

                // Create 490 field if we have a series statement
                if let Some(stmt) = series_statement {
                    let mut field = Field::new("490".to_string(), '0', ' ');
                    field.add_subfield('a', stmt);
                    if let Some(vol) = enumeration {
                        field.add_subfield('v', vol);
                    }
                    record.add_field(field);
                }
            }
        }
    }

    /// Extracts linking entries to 76X-78X fields.
    fn extract_linking_entries(&mut self, record: &mut Record) {
        if let Some(ref instance_key) = self.instance_node {
            // Map of BIBFRAME relationship properties to MARC tags
            let relationship_map = [
                ("precededBy", "780"),
                ("succeededBy", "785"),
                ("partOf", "773"),
                ("hasPart", "774"),
                ("otherPhysicalFormat", "776"),
                ("relatedTo", "787"),
                ("hasSeries", "760"),
                ("supplement", "770"),
                ("supplementTo", "772"),
                ("otherEdition", "775"),
                ("issuedWith", "777"),
            ];

            if let Some(props) = self.subject_index.get(instance_key) {
                for (rel_name, tag) in relationship_map {
                    let rel_prop = format!("{BF}{rel_name}");

                    for (pred, obj) in props.clone() {
                        if pred == rel_prop {
                            let related_key = node_to_key(&obj);
                            if let Some(field) = self.create_linking_field(tag, &related_key) {
                                record.add_field(field);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Creates a linking entry field (76X-78X) from a related entity.
    #[allow(clippy::cognitive_complexity)]
    fn create_linking_field(&self, tag: &str, related_key: &str) -> Option<Field> {
        let related_props = self.subject_index.get(related_key)?;

        let mut field = Field::new(tag.to_string(), '0', ' ');

        // Extract title
        for (pred, obj) in related_props {
            if pred.ends_with("title") {
                let title_key = node_to_key(obj);
                if let Some(title_props) = self.subject_index.get(&title_key) {
                    for (t_pred, t_obj) in title_props {
                        if t_pred.ends_with("mainTitle") {
                            if let RdfNode::Literal { value, .. } = t_obj {
                                field.add_subfield('t', value.clone());
                            }
                        }
                    }
                }
            }
        }

        // Extract identifiers
        let identified_by_prop = format!("{BF}{}", properties::IDENTIFIED_BY);
        for (pred, obj) in related_props {
            if pred == &identified_by_prop {
                let id_key = node_to_key(obj);
                if let Some(id_props) = self.subject_index.get(&id_key) {
                    let mut id_type = "Local";
                    let mut id_value = None;

                    for (id_pred, id_obj) in id_props {
                        if id_pred == &format!("{RDF}type") {
                            if let RdfNode::Uri(type_uri) = id_obj {
                                if type_uri.ends_with("Issn") {
                                    id_type = "Issn";
                                } else if type_uri.ends_with("Isbn") {
                                    id_type = "Isbn";
                                }
                            }
                        }
                        if id_pred == &format!("{RDF}value") {
                            if let RdfNode::Literal { value, .. } = id_obj {
                                id_value = Some(value.clone());
                            }
                        }
                    }

                    if let Some(val) = id_value {
                        match id_type {
                            "Issn" => field.add_subfield('x', val),
                            "Isbn" => field.add_subfield('z', val),
                            _ => field.add_subfield('w', val),
                        }
                    }
                }
            }
        }

        // Only return if we have at least one subfield
        if field.subfields.is_empty() {
            None
        } else {
            Some(field)
        }
    }
}

/// Converts an `RdfNode` to a string key for indexing.
fn node_to_key(node: &RdfNode) -> String {
    match node {
        RdfNode::Uri(uri) => uri.clone(),
        RdfNode::BlankNode(id) => format!("_:{id}"),
        RdfNode::Literal { value, .. } => value.clone(),
    }
}

/// Checks if a type URI is a Work subtype.
fn is_work_subtype(type_uri: &str) -> bool {
    let subtypes = [
        "Text",
        "NotatedMusic",
        "Cartography",
        "MovingImage",
        "StillImage",
        "Audio",
        "MusicAudio",
        "Multimedia",
        "MixedMaterial",
        "Object",
        "Kit",
    ];
    subtypes.iter().any(|t| type_uri.ends_with(t))
}

/// Checks if a type URI is an Instance subtype.
fn is_instance_subtype(type_uri: &str) -> bool {
    let subtypes = ["Serial", "Manuscript", "Electronic", "Print"];
    subtypes.iter().any(|t| type_uri.ends_with(t))
}

/// Maps Work type URI to Leader position 06.
fn work_type_to_leader_06(type_uri: &str) -> char {
    if type_uri.ends_with("Text") {
        'a'
    } else if type_uri.ends_with("NotatedMusic") {
        'c'
    } else if type_uri.ends_with("Cartography") {
        'e'
    } else if type_uri.ends_with("MovingImage") {
        'g'
    } else if type_uri.ends_with("MusicAudio") {
        // Must check MusicAudio before Audio since MusicAudio ends with "Audio"
        'j'
    } else if type_uri.ends_with("Audio") {
        'i'
    } else if type_uri.ends_with("StillImage") {
        'k'
    } else if type_uri.ends_with("Multimedia") {
        'm'
    } else if type_uri.ends_with("Kit") {
        'o'
    } else if type_uri.ends_with("MixedMaterial") {
        'p'
    } else if type_uri.ends_with("Object") {
        'r'
    } else {
        'a' // Default: language material
    }
}

/// Maps Instance type URI to Leader position 07.
fn instance_type_to_leader_07(type_uri: &str) -> char {
    if type_uri.ends_with("Serial") {
        's'
    } else {
        // Default: monograph (including Manuscript which also uses 'm')
        'm'
    }
}

/// Maps subject type URI to MARC tag.
fn subject_type_to_tag(type_uri: &str) -> &'static str {
    if type_uri.ends_with("Person") {
        "600"
    } else if type_uri.ends_with("Organization") {
        "610"
    } else if type_uri.ends_with("Meeting") {
        "611"
    } else if type_uri.ends_with("Work") {
        "630"
    } else if type_uri.ends_with("Topic") {
        "650"
    } else if type_uri.ends_with("Place") {
        "651"
    } else if type_uri.ends_with("GenreForm") {
        "655"
    } else {
        "650" // Default: topical
    }
}

/// Maps identifier type URI to MARC tag.
fn identifier_type_to_tag(type_uri: &str) -> &'static str {
    if type_uri.ends_with("Lccn") {
        "010"
    } else if type_uri.ends_with("Isbn") {
        "020"
    } else if type_uri.ends_with("Issn") {
        "022"
    } else if type_uri.ends_with("Isrc")
        || type_uri.ends_with("Upc")
        || type_uri.ends_with("Ismn")
        || type_uri.ends_with("Ean")
    {
        "024"
    } else {
        "035" // Default: system control number
    }
}

/// Maps provision activity type to MARC 264 indicator 2.
fn provision_type_to_indicator(type_uri: &str) -> char {
    if type_uri.ends_with("Production") {
        '0'
    } else if type_uri.ends_with("Publication") {
        '1'
    } else if type_uri.ends_with("Distribution") {
        '2'
    } else if type_uri.ends_with("Manufacture") {
        '3'
    } else {
        '1' // Default: publication
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bibframe::{marc_to_bibframe, BibframeConfig};
    use crate::leader::Leader;

    fn make_test_leader() -> Leader {
        Leader {
            record_length: 1000,
            record_status: 'n',
            record_type: 'a',
            bibliographic_level: 'm',
            control_record_type: ' ',
            character_coding: 'a',
            indicator_count: 2,
            subfield_code_count: 2,
            data_base_address: 100,
            encoding_level: ' ',
            cataloging_form: 'a',
            multipart_level: ' ',
            reserved: "4500".to_string(),
        }
    }

    #[test]
    fn test_basic_roundtrip() {
        // Create a MARC record
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test123".to_string());

        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test Title".to_string());
        record.add_field(field);

        // Convert to BIBFRAME
        let config = BibframeConfig::default();
        let graph = marc_to_bibframe(&record, &config);

        // Convert back to MARC
        let result = convert_bibframe_to_marc(&graph).unwrap();

        // Check that title is preserved
        assert!(result.fields.contains_key("245"));
        let titles = result.fields.get("245").unwrap();
        assert!(!titles.is_empty());
        assert!(titles[0]
            .subfields
            .iter()
            .any(|s| s.code == 'a' && s.value.contains("Test Title")));
    }

    #[test]
    fn test_creator_roundtrip() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test456".to_string());

        let mut field = Field::new("100".to_string(), '1', ' ');
        field.add_subfield('a', "Smith, John".to_string());
        field.add_subfield('4', "aut".to_string());
        record.add_field(field);

        let config = BibframeConfig::default();
        let graph = marc_to_bibframe(&record, &config);
        let result = convert_bibframe_to_marc(&graph).unwrap();

        // Check that creator is preserved (as 100)
        assert!(result.fields.contains_key("100"));
    }

    #[test]
    fn test_subject_roundtrip() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test789".to_string());

        let mut field = Field::new("650".to_string(), ' ', '0');
        field.add_subfield('a', "Computer science".to_string());
        record.add_field(field);

        let config = BibframeConfig::default();
        let graph = marc_to_bibframe(&record, &config);
        let result = convert_bibframe_to_marc(&graph).unwrap();

        assert!(result.fields.contains_key("650"));
    }

    #[test]
    fn test_identifier_roundtrip() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "testabc".to_string());

        let mut field = Field::new("020".to_string(), ' ', ' ');
        field.add_subfield('a', "9780123456789".to_string());
        record.add_field(field);

        let config = BibframeConfig::default();
        let graph = marc_to_bibframe(&record, &config);
        let result = convert_bibframe_to_marc(&graph).unwrap();

        assert!(result.fields.contains_key("020"));
        let isbns = result.fields.get("020").unwrap();
        assert!(isbns[0]
            .subfields
            .iter()
            .any(|s| s.value.contains("9780123456789")));
    }

    #[test]
    fn test_publication_roundtrip() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "testdef".to_string());

        let mut field = Field::new("264".to_string(), ' ', '1');
        field.add_subfield('a', "New York".to_string());
        field.add_subfield('b', "Publisher".to_string());
        field.add_subfield('c', "2020".to_string());
        record.add_field(field);

        let config = BibframeConfig::default();
        let graph = marc_to_bibframe(&record, &config);
        let result = convert_bibframe_to_marc(&graph).unwrap();

        assert!(result.fields.contains_key("264"));
    }

    #[test]
    fn test_empty_graph() {
        let graph = RdfGraph::new();
        let result = convert_bibframe_to_marc(&graph).unwrap();

        // Should still produce a valid record with minimal data
        assert_eq!(result.leader.record_type, 'a');
    }

    #[test]
    fn test_work_type_preservation() {
        let mut leader = make_test_leader();
        leader.record_type = 'j'; // Musical sound recording
        let record = Record::new(leader);

        let config = BibframeConfig::default();
        let graph = marc_to_bibframe(&record, &config);
        let result = convert_bibframe_to_marc(&graph).unwrap();

        // Should preserve the musical type
        assert_eq!(result.leader.record_type, 'j');
    }

    // ========================================================================
    // Edge Case Round-Trip Tests (mrrc-uab.4.4)
    // ========================================================================

    #[test]
    fn test_series_roundtrip() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "series_test".to_string());

        // Add 830 - Series added entry
        let mut field_830 = Field::new("830".to_string(), ' ', '0');
        field_830.add_subfield('a', "Computer science series".to_string());
        record.add_field(field_830);

        let config = BibframeConfig::default();
        let graph = marc_to_bibframe(&record, &config);
        let result = convert_bibframe_to_marc(&graph).unwrap();

        // Should have series field
        assert!(result.fields.contains_key("830"));
        let series = result.fields.get("830").unwrap();
        assert!(series[0]
            .subfields
            .iter()
            .any(|s| s.value.contains("Computer science")));
    }

    #[test]
    fn test_linking_entry_roundtrip() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "linking_test".to_string());

        // Add 780 - Preceding entry
        let mut field_780 = Field::new("780".to_string(), '0', '0');
        field_780.add_subfield('t', "Previous Title".to_string());
        field_780.add_subfield('x', "1234-5678".to_string());
        record.add_field(field_780);

        let config = BibframeConfig::default();
        let graph = marc_to_bibframe(&record, &config);
        let result = convert_bibframe_to_marc(&graph).unwrap();

        // Should have linking entry
        assert!(result.fields.contains_key("780"));
        let linking = result.fields.get("780").unwrap();
        assert!(linking[0]
            .subfields
            .iter()
            .any(|s| s.code == 't' && s.value.contains("Previous Title")));
        assert!(linking[0]
            .subfields
            .iter()
            .any(|s| s.code == 'x' && s.value.contains("1234-5678")));
    }

    #[test]
    fn test_series_statement_roundtrip() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "series490_test".to_string());

        // Add 490 - Series statement
        let mut field_490 = Field::new("490".to_string(), '0', ' ');
        field_490.add_subfield('a', "Library science series".to_string());
        field_490.add_subfield('v', "vol. 5".to_string());
        record.add_field(field_490);

        let config = BibframeConfig::default();
        let graph = marc_to_bibframe(&record, &config);
        let result = convert_bibframe_to_marc(&graph).unwrap();

        // Should have series statement
        assert!(result.fields.contains_key("490"));
        let series = result.fields.get("490").unwrap();
        assert!(series[0]
            .subfields
            .iter()
            .any(|s| s.value.contains("Library science")));
    }
}
