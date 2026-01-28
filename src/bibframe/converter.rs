//! MARC to BIBFRAME conversion logic.
//!
//! This module implements the core conversion from MARC bibliographic records
//! to BIBFRAME 2.0 RDF graphs following LOC specifications.

use crate::record::{Field, Record};

use super::config::BibframeConfig;
use super::namespaces::{classes, properties, BF, BFLC, RDF, RDFS, RELATORS};
use super::rdf::{RdfGraph, RdfNode};

/// Converts a MARC record to a BIBFRAME RDF graph.
///
/// This is the main entry point for MARC→BIBFRAME conversion.
pub fn convert_marc_to_bibframe(record: &Record, config: &BibframeConfig) -> RdfGraph {
    let converter = MarcToBibframeConverter::new(record, config);
    converter.convert()
}

/// Internal converter state.
struct MarcToBibframeConverter<'a> {
    record: &'a Record,
    config: &'a BibframeConfig,
    graph: RdfGraph,
    work_node: Option<RdfNode>,
    instance_node: Option<RdfNode>,
}

impl<'a> MarcToBibframeConverter<'a> {
    fn new(record: &'a Record, config: &'a BibframeConfig) -> Self {
        Self {
            record,
            config,
            graph: RdfGraph::new(),
            work_node: None,
            instance_node: None,
        }
    }

    fn convert(mut self) -> RdfGraph {
        // Create Work and Instance nodes
        self.create_work_node();
        self.create_instance_node();

        // Link Work and Instance
        self.link_work_instance();

        // Add Work type from Leader
        self.add_work_type();

        // Process field groups
        self.process_titles();
        self.process_creators();
        self.process_contributors();
        self.process_subjects();
        self.process_identifiers();
        self.process_provision_activity();
        self.process_physical_description();
        self.process_notes();

        // Add admin metadata if configured
        if self.config.include_bflc {
            self.add_admin_metadata();
        }

        self.graph
    }

    /// Creates the Work node with appropriate URI or blank node.
    fn create_work_node(&mut self) {
        let node = self.generate_entity_uri("work");
        self.work_node = Some(node.clone());

        // Add rdf:type
        self.graph.add(
            node,
            format!("{RDF}type"),
            RdfNode::bf_class(classes::WORK),
        );
    }

    /// Creates the Instance node with appropriate URI or blank node.
    fn create_instance_node(&mut self) {
        let node = self.generate_entity_uri("instance");
        self.instance_node = Some(node.clone());

        // Add rdf:type - determine from Leader if possible
        let instance_type = self.determine_instance_type();
        self.graph.add(
            node,
            format!("{RDF}type"),
            RdfNode::bf_class(instance_type),
        );
    }

    /// Links Work and Instance with hasInstance/instanceOf.
    fn link_work_instance(&mut self) {
        if let (Some(work), Some(instance)) = (&self.work_node, &self.instance_node) {
            self.graph.add(
                work.clone(),
                format!("{BF}{}", properties::HAS_INSTANCE),
                instance.clone(),
            );
            self.graph.add(
                instance.clone(),
                format!("{BF}{}", properties::INSTANCE_OF),
                work.clone(),
            );
        }
    }

    /// Adds the Work type based on Leader position 06.
    fn add_work_type(&mut self) {
        let work_type = self.determine_work_type();
        if let Some(work) = &self.work_node {
            // Add additional type (Work is already added)
            if work_type != classes::WORK {
                self.graph.add(
                    work.clone(),
                    format!("{RDF}type"),
                    RdfNode::bf_class(work_type),
                );
            }
        }
    }

    /// Determines Work type from Leader position 06.
    fn determine_work_type(&self) -> &'static str {
        match self.record.leader.record_type {
            'a' | 't' => classes::TEXT,
            'c' | 'd' => classes::NOTATED_MUSIC,
            'e' | 'f' => classes::CARTOGRAPHY,
            'g' => classes::MOVING_IMAGE,
            'i' => classes::AUDIO,
            'j' => classes::MUSIC_AUDIO,
            'k' => classes::STILL_IMAGE,
            'm' => classes::MULTIMEDIA,
            'o' => classes::KIT,
            'p' => classes::MIXED_MATERIAL,
            'r' => classes::OBJECT,
            _ => classes::WORK,
        }
    }

    /// Determines Instance type from Leader positions 06 and 07.
    fn determine_instance_type(&self) -> &'static str {
        let leader = &self.record.leader;
        match (leader.record_type, leader.bibliographic_level) {
            ('t' | 'd' | 'f', _) => classes::MANUSCRIPT,
            ('m', _) => classes::ELECTRONIC,
            (_, 's' | 'i') => classes::SERIAL,
            _ => classes::INSTANCE,
        }
    }

    /// Generates a URI or blank node for an entity.
    fn generate_entity_uri(&mut self, entity_type: &str) -> RdfNode {
        if let Some(ref base) = self.config.base_uri {
            let id = if self.config.use_control_number {
                self.record
                    .control_fields
                    .get("001")
                    .map_or("unknown", String::as_str)
            } else {
                "unknown"
            };
            RdfNode::uri(format!("{base}{entity_type}/{id}"))
        } else {
            self.graph.new_blank_node()
        }
    }

    /// Processes 245 and 246 fields into bf:title.
    fn process_titles(&mut self) {
        let instance = match &self.instance_node {
            Some(n) => n.clone(),
            None => return,
        };

        // Process 245 - Main Title
        if let Some(fields) = self.record.fields.get("245") {
            for field in fields {
                self.add_title(&instance, field, true);
            }
        }

        // Process 246 - Variant Titles
        if let Some(fields) = self.record.fields.get("246") {
            for field in fields {
                self.add_title(&instance, field, false);
            }
        }
    }

    /// Adds a title from a MARC field.
    fn add_title(&mut self, instance: &RdfNode, field: &Field, is_main: bool) {
        let title_node = self.graph.new_blank_node();

        // Add type
        self.graph.add(
            title_node.clone(),
            format!("{RDF}type"),
            RdfNode::bf_class(classes::TITLE),
        );

        // Extract title components
        for subfield in &field.subfields {
            match subfield.code {
                'a' => {
                    self.graph.add(
                        title_node.clone(),
                        format!("{BF}{}", properties::MAIN_TITLE),
                        RdfNode::literal(&subfield.value),
                    );
                }
                'b' => {
                    self.graph.add(
                        title_node.clone(),
                        format!("{BF}{}", properties::SUBTITLE),
                        RdfNode::literal(&subfield.value),
                    );
                }
                'n' => {
                    self.graph.add(
                        title_node.clone(),
                        format!("{BF}{}", properties::PART_NUMBER),
                        RdfNode::literal(&subfield.value),
                    );
                }
                'p' => {
                    self.graph.add(
                        title_node.clone(),
                        format!("{BF}{}", properties::PART_NAME),
                        RdfNode::literal(&subfield.value),
                    );
                }
                _ => {}
            }
        }

        // Link to instance
        self.graph.add(
            instance.clone(),
            format!("{BF}{}", properties::TITLE),
            title_node,
        );

        // Add responsibility statement from $c to instance
        if is_main {
            for subfield in &field.subfields {
                if subfield.code == 'c' {
                    self.graph.add(
                        instance.clone(),
                        format!("{BF}{}", properties::RESPONSIBILITY_STATEMENT),
                        RdfNode::literal(&subfield.value),
                    );
                }
            }
        }
    }

    /// Processes 1XX fields (main entry/primary creator).
    fn process_creators(&mut self) {
        let work = match &self.work_node {
            Some(n) => n.clone(),
            None => return,
        };

        // 100 - Personal name
        if let Some(fields) = self.record.fields.get("100") {
            for field in fields {
                self.add_contribution(&work, field, classes::PERSON, true);
            }
        }

        // 110 - Corporate name
        if let Some(fields) = self.record.fields.get("110") {
            for field in fields {
                self.add_contribution(&work, field, classes::ORGANIZATION, true);
            }
        }

        // 111 - Meeting name
        if let Some(fields) = self.record.fields.get("111") {
            for field in fields {
                self.add_contribution(&work, field, classes::MEETING, true);
            }
        }
    }

    /// Processes 7XX fields (added entries/contributors).
    fn process_contributors(&mut self) {
        let work = match &self.work_node {
            Some(n) => n.clone(),
            None => return,
        };

        // 700 - Added personal name
        if let Some(fields) = self.record.fields.get("700") {
            for field in fields {
                self.add_contribution(&work, field, classes::PERSON, false);
            }
        }

        // 710 - Added corporate name
        if let Some(fields) = self.record.fields.get("710") {
            for field in fields {
                self.add_contribution(&work, field, classes::ORGANIZATION, false);
            }
        }

        // 711 - Added meeting name
        if let Some(fields) = self.record.fields.get("711") {
            for field in fields {
                self.add_contribution(&work, field, classes::MEETING, false);
            }
        }
    }

    /// Adds a contribution (agent with role) to the work.
    fn add_contribution(
        &mut self,
        work: &RdfNode,
        field: &Field,
        agent_type: &str,
        is_primary: bool,
    ) {
        // Create contribution node
        let contribution_node = self.graph.new_blank_node();

        // Determine contribution type
        let contrib_type = if is_primary && self.config.include_bflc {
            format!("{BFLC}PrimaryContribution")
        } else {
            format!("{BF}Contribution")
        };

        self.graph.add(
            contribution_node.clone(),
            format!("{RDF}type"),
            RdfNode::uri(&contrib_type),
        );

        // Create agent node
        let agent_node = self.graph.new_blank_node();
        self.graph.add(
            agent_node.clone(),
            format!("{RDF}type"),
            RdfNode::bf_class(agent_type),
        );

        // Extract agent name from $a, $b, $c, $d, $q
        let mut name_parts = Vec::new();
        for subfield in &field.subfields {
            match subfield.code {
                'a' | 'b' | 'c' | 'd' | 'q' => name_parts.push(subfield.value.clone()),
                _ => {}
            }
        }

        if !name_parts.is_empty() {
            let label = name_parts.join(" ").trim().to_string();
            self.graph.add(
                agent_node.clone(),
                format!("{RDFS}label"),
                RdfNode::literal(&label),
            );
        }

        // Link agent to contribution
        self.graph.add(
            contribution_node.clone(),
            format!("{BF}{}", properties::AGENT),
            agent_node,
        );

        // Add role from $4 or $e
        self.add_relator_role(&contribution_node, field);

        // Link contribution to work
        self.graph.add(
            work.clone(),
            format!("{BF}{}", properties::CONTRIBUTION),
            contribution_node,
        );
    }

    /// Adds relator role from $4 (code) or $e (term).
    fn add_relator_role(&mut self, contribution: &RdfNode, field: &Field) {
        for subfield in &field.subfields {
            match subfield.code {
                '4' => {
                    // Relator code - link to id.loc.gov vocabulary
                    let code = subfield.value.trim().to_lowercase();
                    if !code.is_empty() {
                        self.graph.add(
                            contribution.clone(),
                            format!("{BF}{}", properties::ROLE),
                            RdfNode::uri(format!("{RELATORS}{code}")),
                        );
                    }
                }
                'e' => {
                    // Relator term - use as literal if no $4
                    if !field.subfields.iter().any(|s| s.code == '4') {
                        self.graph.add(
                            contribution.clone(),
                            format!("{BF}{}", properties::ROLE),
                            RdfNode::literal(&subfield.value),
                        );
                    }
                }
                _ => {}
            }
        }
    }

    /// Processes 6XX fields (subjects).
    fn process_subjects(&mut self) {
        let work = match &self.work_node {
            Some(n) => n.clone(),
            None => return,
        };

        // 600 - Personal name subject
        if let Some(fields) = self.record.fields.get("600") {
            for field in fields {
                self.add_subject(&work, field, classes::PERSON);
            }
        }

        // 610 - Corporate name subject
        if let Some(fields) = self.record.fields.get("610") {
            for field in fields {
                self.add_subject(&work, field, classes::ORGANIZATION);
            }
        }

        // 611 - Meeting name subject
        if let Some(fields) = self.record.fields.get("611") {
            for field in fields {
                self.add_subject(&work, field, classes::MEETING);
            }
        }

        // 630 - Uniform title subject
        if let Some(fields) = self.record.fields.get("630") {
            for field in fields {
                self.add_subject(&work, field, classes::WORK);
            }
        }

        // 650 - Topical subject
        if let Some(fields) = self.record.fields.get("650") {
            for field in fields {
                self.add_subject(&work, field, classes::TOPIC);
            }
        }

        // 651 - Geographic subject
        if let Some(fields) = self.record.fields.get("651") {
            for field in fields {
                self.add_subject(&work, field, classes::PLACE);
            }
        }

        // 655 - Genre/form
        if let Some(fields) = self.record.fields.get("655") {
            for field in fields {
                self.add_subject(&work, field, classes::GENRE_FORM);
            }
        }
    }

    /// Adds a subject to the work.
    fn add_subject(&mut self, work: &RdfNode, field: &Field, subject_type: &str) {
        let subject_node = self.graph.new_blank_node();

        // Add type
        self.graph.add(
            subject_node.clone(),
            format!("{RDF}type"),
            RdfNode::bf_class(subject_type),
        );

        // Extract label from $a (and subdivisions)
        let mut label_parts = Vec::new();
        for subfield in &field.subfields {
            match subfield.code {
                'a' | 'b' | 'c' | 'd' | 'v' | 'x' | 'y' | 'z' => {
                    label_parts.push(subfield.value.clone());
                }
                _ => {}
            }
        }

        if !label_parts.is_empty() {
            let label = label_parts.join("--");
            self.graph.add(
                subject_node.clone(),
                format!("{RDFS}label"),
                RdfNode::literal(&label),
            );
        }

        // Link to work
        self.graph.add(
            work.clone(),
            format!("{BF}{}", properties::SUBJECT),
            subject_node,
        );
    }

    /// Processes identifier fields (020, 022, 024, 035, etc.).
    fn process_identifiers(&mut self) {
        let instance = match &self.instance_node {
            Some(n) => n.clone(),
            None => return,
        };

        // 010 - LCCN
        if let Some(fields) = self.record.fields.get("010") {
            for field in fields {
                self.add_identifier(&instance, field, classes::LCCN);
            }
        }

        // 020 - ISBN
        if let Some(fields) = self.record.fields.get("020") {
            for field in fields {
                self.add_identifier(&instance, field, classes::ISBN);
            }
        }

        // 022 - ISSN
        if let Some(fields) = self.record.fields.get("022") {
            for field in fields {
                self.add_identifier(&instance, field, classes::ISSN);
            }
        }

        // 024 - Other standard identifier
        if let Some(fields) = self.record.fields.get("024") {
            for field in fields {
                // Type depends on first indicator
                // Note: ind1='7' means source specified in $2, but we use generic Identifier
                let id_type = match field.indicator1 {
                    '0' => "Isrc",
                    '1' => "Upc",
                    '2' => "Ismn",
                    '3' => "Ean",
                    _ => "Identifier",
                };
                self.add_identifier(&instance, field, id_type);
            }
        }

        // 035 - System control number
        if let Some(fields) = self.record.fields.get("035") {
            for field in fields {
                self.add_identifier(&instance, field, classes::LOCAL);
            }
        }
    }

    /// Adds an identifier to the instance.
    fn add_identifier(&mut self, instance: &RdfNode, field: &Field, id_type: &str) {
        let id_node = self.graph.new_blank_node();

        // Add type
        self.graph.add(
            id_node.clone(),
            format!("{RDF}type"),
            RdfNode::bf_class(id_type),
        );

        // Add value from $a
        for subfield in &field.subfields {
            if subfield.code == 'a' {
                self.graph.add(
                    id_node.clone(),
                    format!("{RDF}value"),
                    RdfNode::literal(&subfield.value),
                );
            }
        }

        // Link to instance
        self.graph.add(
            instance.clone(),
            format!("{BF}{}", properties::IDENTIFIED_BY),
            id_node,
        );
    }

    /// Processes 260/264 fields (publication/provision activity).
    fn process_provision_activity(&mut self) {
        let instance = match &self.instance_node {
            Some(n) => n.clone(),
            None => return,
        };

        // 260 - Publication (older format)
        if let Some(fields) = self.record.fields.get("260") {
            for field in fields {
                self.add_provision_activity(&instance, field, classes::PUBLICATION);
            }
        }

        // 264 - Production, Publication, etc.
        if let Some(fields) = self.record.fields.get("264") {
            for field in fields {
                // Type depends on second indicator (ind2='1' or unspecified = Publication)
                let activity_type = match field.indicator2 {
                    '0' => classes::PRODUCTION,
                    '2' => classes::DISTRIBUTION,
                    '3' => classes::MANUFACTURE,
                    '4' => {
                        // Copyright date - handle specially
                        self.add_copyright_date(&instance, field);
                        continue;
                    }
                    _ => classes::PUBLICATION, // Default: ind2='1' or blank
                };
                self.add_provision_activity(&instance, field, activity_type);
            }
        }
    }

    /// Adds a provision activity (publication, distribution, etc.).
    fn add_provision_activity(&mut self, instance: &RdfNode, field: &Field, activity_type: &str) {
        let activity_node = self.graph.new_blank_node();

        // Add type
        self.graph.add(
            activity_node.clone(),
            format!("{RDF}type"),
            RdfNode::bf_class(activity_type),
        );

        // Process subfields
        for subfield in &field.subfields {
            match subfield.code {
                'a' => {
                    // Place
                    let place_node = self.graph.new_blank_node();
                    self.graph.add(
                        place_node.clone(),
                        format!("{RDF}type"),
                        RdfNode::bf_class(classes::PLACE),
                    );
                    self.graph.add(
                        place_node.clone(),
                        format!("{RDFS}label"),
                        RdfNode::literal(&subfield.value),
                    );
                    self.graph.add(
                        activity_node.clone(),
                        format!("{BF}{}", properties::PLACE),
                        place_node,
                    );

                    // Also add simple place if BFLC enabled
                    if self.config.include_bflc {
                        self.graph.add(
                            activity_node.clone(),
                            format!("{BFLC}simplePlace"),
                            RdfNode::literal(&subfield.value),
                        );
                    }
                }
                'b' => {
                    // Agent (publisher/producer/etc.)
                    let agent_node = self.graph.new_blank_node();
                    self.graph.add(
                        agent_node.clone(),
                        format!("{RDFS}label"),
                        RdfNode::literal(&subfield.value),
                    );
                    self.graph.add(
                        activity_node.clone(),
                        format!("{BF}{}", properties::AGENT),
                        agent_node,
                    );

                    // Also add simple agent if BFLC enabled
                    if self.config.include_bflc {
                        self.graph.add(
                            activity_node.clone(),
                            format!("{BFLC}simpleAgent"),
                            RdfNode::literal(&subfield.value),
                        );
                    }
                }
                'c' => {
                    // Date
                    self.graph.add(
                        activity_node.clone(),
                        format!("{BF}{}", properties::DATE),
                        RdfNode::literal(&subfield.value),
                    );

                    // Also add simple date if BFLC enabled
                    if self.config.include_bflc {
                        self.graph.add(
                            activity_node.clone(),
                            format!("{BFLC}simpleDate"),
                            RdfNode::literal(&subfield.value),
                        );
                    }
                }
                _ => {}
            }
        }

        // Link to instance
        self.graph.add(
            instance.clone(),
            format!("{BF}{}", properties::PROVISION_ACTIVITY),
            activity_node,
        );
    }

    /// Adds copyright date from 264 ind2=4.
    fn add_copyright_date(&mut self, instance: &RdfNode, field: &Field) {
        for subfield in &field.subfields {
            if subfield.code == 'c' {
                self.graph.add(
                    instance.clone(),
                    format!("{BF}{}", properties::COPYRIGHT_DATE),
                    RdfNode::literal(&subfield.value),
                );
            }
        }
    }

    /// Processes 300 field (physical description).
    fn process_physical_description(&mut self) {
        let instance = match &self.instance_node {
            Some(n) => n.clone(),
            None => return,
        };

        if let Some(fields) = self.record.fields.get("300") {
            for field in fields {
                for subfield in &field.subfields {
                    match subfield.code {
                        'a' => {
                            // Extent
                            self.graph.add(
                                instance.clone(),
                                format!("{BF}{}", properties::EXTENT),
                                RdfNode::literal(&subfield.value),
                            );
                        }
                        'c' => {
                            // Dimensions
                            self.graph.add(
                                instance.clone(),
                                format!("{BF}{}", properties::DIMENSIONS),
                                RdfNode::literal(&subfield.value),
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Processes 5XX note fields.
    fn process_notes(&mut self) {
        let instance = match &self.instance_node {
            Some(n) => n.clone(),
            None => return,
        };

        // 500 - General note
        if let Some(fields) = self.record.fields.get("500") {
            for field in fields {
                self.add_note(&instance, field, "Note");
            }
        }

        // 520 - Summary
        if let Some(fields) = self.record.fields.get("520") {
            for field in fields {
                for subfield in &field.subfields {
                    if subfield.code == 'a' {
                        self.graph.add(
                            instance.clone(),
                            format!("{BF}{}", properties::SUMMARY),
                            RdfNode::literal(&subfield.value),
                        );
                    }
                }
            }
        }

        // 504 - Bibliography note
        if let Some(fields) = self.record.fields.get("504") {
            for field in fields {
                self.add_note(&instance, field, "Note");
            }
        }
    }

    /// Adds a note to the instance.
    fn add_note(&mut self, instance: &RdfNode, field: &Field, _note_type: &str) {
        for subfield in &field.subfields {
            if subfield.code == 'a' {
                self.graph.add(
                    instance.clone(),
                    format!("{BF}{}", properties::NOTE),
                    RdfNode::literal(&subfield.value),
                );
            }
        }
    }

    /// Adds administrative metadata.
    fn add_admin_metadata(&mut self) {
        let instance = match &self.instance_node {
            Some(n) => n.clone(),
            None => return,
        };

        let admin_node = self.graph.new_blank_node();

        // Add type
        self.graph.add(
            admin_node.clone(),
            format!("{RDF}type"),
            RdfNode::bf_class(classes::ADMIN_METADATA),
        );

        // Add encoding level from Leader/17
        let encoding_level = self.record.leader.encoding_level;
        if encoding_level != ' ' {
            self.graph.add(
                admin_node.clone(),
                format!("{BFLC}encodingLevel"),
                RdfNode::literal(encoding_level.to_string()),
            );
        }

        // Add creation date from 008/00-05 if available
        if let Some(field_008) = self.record.control_fields.get("008") {
            if field_008.len() >= 6 {
                let date_entered = &field_008[0..6];
                self.graph.add(
                    admin_node.clone(),
                    format!("{BF}{}", properties::CREATION_DATE),
                    RdfNode::literal(date_entered),
                );
            }
        }

        // Link to instance
        self.graph.add(
            instance.clone(),
            format!("{BF}{}", properties::ADMIN_METADATA),
            admin_node,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::leader::Leader;
    use crate::record::Field;

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
    fn test_basic_conversion() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test123".to_string());

        let config = BibframeConfig::default();
        let graph = convert_marc_to_bibframe(&record, &config);

        // Should have Work and Instance with types and links
        assert!(graph.len() >= 4);
    }

    #[test]
    fn test_title_conversion() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test123".to_string());

        let mut field = Field::new("245".to_string(), '1', '0');
        field.add_subfield('a', "Test Title :".to_string());
        field.add_subfield('b', "a subtitle /".to_string());
        field.add_subfield('c', "by Author.".to_string());
        record.add_field(field);

        let config = BibframeConfig::default();
        let graph = convert_marc_to_bibframe(&record, &config);

        let serialized = graph.serialize(super::super::config::RdfFormat::NTriples).unwrap();
        assert!(serialized.contains("mainTitle"));
        assert!(serialized.contains("Test Title"));
        assert!(serialized.contains("subtitle"));
        assert!(serialized.contains("responsibilityStatement"));
    }

    #[test]
    fn test_creator_conversion() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test123".to_string());

        let mut field = Field::new("100".to_string(), '1', ' ');
        field.add_subfield('a', "Smith, John,".to_string());
        field.add_subfield('d', "1950-".to_string());
        field.add_subfield('4', "aut".to_string());
        record.add_field(field);

        let config = BibframeConfig::default();
        let graph = convert_marc_to_bibframe(&record, &config);

        let serialized = graph.serialize(super::super::config::RdfFormat::NTriples).unwrap();
        assert!(serialized.contains("Contribution"));
        assert!(serialized.contains("Person"));
        assert!(serialized.contains("Smith, John"));
        assert!(serialized.contains("relators/aut"));
    }

    #[test]
    fn test_subject_conversion() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test123".to_string());

        let mut field = Field::new("650".to_string(), ' ', '0');
        field.add_subfield('a', "Computer science".to_string());
        field.add_subfield('x', "Study and teaching.".to_string());
        record.add_field(field);

        let config = BibframeConfig::default();
        let graph = convert_marc_to_bibframe(&record, &config);

        let serialized = graph.serialize(super::super::config::RdfFormat::NTriples).unwrap();
        assert!(serialized.contains("Topic"));
        assert!(serialized.contains("Computer science"));
    }

    #[test]
    fn test_identifier_conversion() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test123".to_string());

        let mut field = Field::new("020".to_string(), ' ', ' ');
        field.add_subfield('a', "9780123456789".to_string());
        record.add_field(field);

        let config = BibframeConfig::default();
        let graph = convert_marc_to_bibframe(&record, &config);

        let serialized = graph.serialize(super::super::config::RdfFormat::NTriples).unwrap();
        assert!(serialized.contains("Isbn"));
        assert!(serialized.contains("9780123456789"));
    }

    #[test]
    fn test_publication_conversion() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test123".to_string());

        let mut field = Field::new("264".to_string(), ' ', '1');
        field.add_subfield('a', "New York :".to_string());
        field.add_subfield('b', "Publisher,".to_string());
        field.add_subfield('c', "2020.".to_string());
        record.add_field(field);

        let config = BibframeConfig::default();
        let graph = convert_marc_to_bibframe(&record, &config);

        let serialized = graph.serialize(super::super::config::RdfFormat::NTriples).unwrap();
        assert!(serialized.contains("Publication"));
        assert!(serialized.contains("New York"));
        assert!(serialized.contains("Publisher"));
        assert!(serialized.contains("2020"));
    }

    #[test]
    fn test_work_type_determination() {
        // Test music record
        let mut leader = make_test_leader();
        leader.record_type = 'c'; // Notated music
        let record = Record::new(leader);

        let config = BibframeConfig::default();
        let graph = convert_marc_to_bibframe(&record, &config);

        let serialized = graph.serialize(super::super::config::RdfFormat::NTriples).unwrap();
        assert!(serialized.contains("NotatedMusic"));
    }

    #[test]
    fn test_serial_instance_type() {
        let mut leader = make_test_leader();
        leader.bibliographic_level = 's'; // Serial
        let record = Record::new(leader);

        let config = BibframeConfig::default();
        let graph = convert_marc_to_bibframe(&record, &config);

        let serialized = graph.serialize(super::super::config::RdfFormat::NTriples).unwrap();
        assert!(serialized.contains("Serial"));
    }

    #[test]
    fn test_base_uri_generation() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "rec123".to_string());

        let config = BibframeConfig::new().with_base_uri("http://example.org/");
        let graph = convert_marc_to_bibframe(&record, &config);

        let serialized = graph.serialize(super::super::config::RdfFormat::NTriples).unwrap();
        assert!(serialized.contains("http://example.org/work/rec123"));
        assert!(serialized.contains("http://example.org/instance/rec123"));
    }
}
