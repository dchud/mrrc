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
    /// Hub node for expression-level grouping (created when 240 present)
    hub_node: Option<RdfNode>,
    /// Item nodes for specific copies (created when holdings fields present)
    item_nodes: Vec<RdfNode>,
}

impl<'a> MarcToBibframeConverter<'a> {
    fn new(record: &'a Record, config: &'a BibframeConfig) -> Self {
        Self {
            record,
            config,
            graph: RdfGraph::new(),
            work_node: None,
            instance_node: None,
            hub_node: None,
            item_nodes: Vec::new(),
        }
    }

    fn convert(mut self) -> RdfGraph {
        // Create Work and Instance nodes
        self.create_work_node();
        self.create_instance_node();

        // Create Hub node if 240 (uniform title) is present
        self.create_hub_if_needed();

        // Link Work and Instance (possibly through Hub)
        self.link_work_instance();

        // Add Work type from Leader
        self.add_work_type();

        // Process field groups
        self.process_uniform_title(); // 240 uniform title -> Hub
        self.process_titles();
        self.process_creators();
        self.process_contributors();
        self.process_subjects();
        self.process_identifiers();
        self.process_classification();
        self.process_provision_activity();
        self.process_physical_description();
        self.process_notes();

        // Edge case handling (uab.4.4)
        self.process_880_linked_fields();
        self.process_linking_entries();
        self.process_series();
        self.process_format_specific_fields();

        // Process holdings fields to create Items (852, 876-878)
        self.process_holdings();

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
        self.graph
            .add(node, format!("{RDF}type"), RdfNode::bf_class(classes::WORK));
    }

    /// Creates the Instance node with appropriate URI or blank node.
    fn create_instance_node(&mut self) {
        let node = self.generate_entity_uri("instance");
        self.instance_node = Some(node.clone());

        // Add rdf:type - determine from Leader if possible
        let instance_type = self.determine_instance_type();
        self.graph
            .add(node, format!("{RDF}type"), RdfNode::bf_class(instance_type));
    }

    /// Creates a Hub node if field 240 (uniform title) is present.
    /// Hub represents expression-level grouping (e.g., translations, versions).
    fn create_hub_if_needed(&mut self) {
        // Check if 240 field exists
        if self.record.fields.get("240").is_none() {
            return;
        }

        let node = self.generate_entity_uri("hub");
        self.hub_node = Some(node.clone());

        // Add rdf:type bf:Hub
        self.graph
            .add(node, format!("{RDF}type"), RdfNode::bf_class(classes::HUB));
    }

    /// Links Work and Instance with hasInstance/instanceOf.
    /// When Hub is present, links: Work -> hasExpression -> Hub -> hasInstance -> Instance
    fn link_work_instance(&mut self) {
        match (&self.work_node, &self.hub_node, &self.instance_node) {
            // With Hub: Work -> hasExpression -> Hub -> hasInstance -> Instance
            (Some(work), Some(hub), Some(instance)) => {
                // Work -> hasExpression -> Hub
                self.graph.add(
                    work.clone(),
                    format!("{BF}{}", properties::HAS_EXPRESSION),
                    hub.clone(),
                );
                self.graph.add(
                    hub.clone(),
                    format!("{BF}{}", properties::EXPRESSION_OF),
                    work.clone(),
                );
                // Hub -> hasInstance -> Instance
                self.graph.add(
                    hub.clone(),
                    format!("{BF}{}", properties::HAS_INSTANCE),
                    instance.clone(),
                );
                self.graph.add(
                    instance.clone(),
                    format!("{BF}{}", properties::INSTANCE_OF),
                    hub.clone(),
                );
            },
            // Without Hub: Work -> hasInstance -> Instance (direct)
            (Some(work), None, Some(instance)) => {
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
            },
            _ => {},
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

    /// Processes 240 (uniform title) field into Hub title properties.
    /// The 240 field represents a standardized title form for grouping expressions.
    fn process_uniform_title(&mut self) {
        let hub = match &self.hub_node {
            Some(n) => n.clone(),
            None => return, // No Hub = no 240 processing needed
        };

        // Process 240 - Uniform Title
        if let Some(fields) = self.record.fields.get("240") {
            for field in fields {
                let title_node = self.graph.new_blank_node();

                // Add type bf:Title
                self.graph.add(
                    title_node.clone(),
                    format!("{RDF}type"),
                    RdfNode::bf_class(classes::TITLE),
                );

                // Extract 240 subfields
                // $a - Uniform title
                // $d - Date of signing (treaties)
                // $f - Date of work
                // $g - Miscellaneous info
                // $k - Form subheading
                // $l - Language
                // $m - Medium of performance (music)
                // $n - Number of part/section
                // $o - Arranged statement (music)
                // $p - Name of part/section
                // $r - Key (music)
                // $s - Version
                for subfield in &field.subfields {
                    match subfield.code {
                        'a' => {
                            self.graph.add(
                                title_node.clone(),
                                format!("{BF}{}", properties::MAIN_TITLE),
                                RdfNode::literal(&subfield.value),
                            );
                        },
                        'n' => {
                            self.graph.add(
                                title_node.clone(),
                                format!("{BF}{}", properties::PART_NUMBER),
                                RdfNode::literal(&subfield.value),
                            );
                        },
                        'p' => {
                            self.graph.add(
                                title_node.clone(),
                                format!("{BF}{}", properties::PART_NAME),
                                RdfNode::literal(&subfield.value),
                            );
                        },
                        'l' => {
                            // Language of work
                            self.graph.add(
                                hub.clone(),
                                format!("{BF}language"),
                                RdfNode::literal(&subfield.value),
                            );
                        },
                        'f' => {
                            // Date of work
                            self.graph.add(
                                hub.clone(),
                                format!("{BF}{}", properties::DATE),
                                RdfNode::literal(&subfield.value),
                            );
                        },
                        's' => {
                            // Version
                            self.graph.add(
                                hub.clone(),
                                format!("{BF}version"),
                                RdfNode::literal(&subfield.value),
                            );
                        },
                        _ => {},
                    }
                }

                // Link Hub to title
                self.graph.add(
                    hub.clone(),
                    format!("{BF}{}", properties::TITLE),
                    title_node,
                );
            }
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
                },
                'b' => {
                    self.graph.add(
                        title_node.clone(),
                        format!("{BF}{}", properties::SUBTITLE),
                        RdfNode::literal(&subfield.value),
                    );
                },
                'n' => {
                    self.graph.add(
                        title_node.clone(),
                        format!("{BF}{}", properties::PART_NUMBER),
                        RdfNode::literal(&subfield.value),
                    );
                },
                'p' => {
                    self.graph.add(
                        title_node.clone(),
                        format!("{BF}{}", properties::PART_NAME),
                        RdfNode::literal(&subfield.value),
                    );
                },
                _ => {},
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
                _ => {},
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
                },
                'e' => {
                    // Relator term - use as literal if no $4
                    if !field.subfields.iter().any(|s| s.code == '4') {
                        self.graph.add(
                            contribution.clone(),
                            format!("{BF}{}", properties::ROLE),
                            RdfNode::literal(&subfield.value),
                        );
                    }
                },
                _ => {},
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
                },
                _ => {},
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

        // 020 - ISBN (with qualifier handling)
        if let Some(fields) = self.record.fields.get("020") {
            for field in fields {
                self.add_isbn(&instance, field);
            }
        }

        // 022 - ISSN (with linking ISSN handling)
        if let Some(fields) = self.record.fields.get("022") {
            for field in fields {
                self.add_issn(&instance, field);
            }
        }

        // 024 - Other standard identifier
        if let Some(fields) = self.record.fields.get("024") {
            for field in fields {
                self.add_other_identifier(&instance, field);
            }
        }

        // 035 - System control number (with prefix parsing)
        if let Some(fields) = self.record.fields.get("035") {
            for field in fields {
                self.add_system_control_number(&instance, field);
            }
        }
    }

    /// Adds an ISBN identifier with qualifier handling.
    fn add_isbn(&mut self, instance: &RdfNode, field: &Field) {
        // $a contains the ISBN
        if let Some(isbn_subfield) = field.subfields.iter().find(|s| s.code == 'a') {
            let id_node = self.graph.new_blank_node();

            self.graph.add(
                id_node.clone(),
                format!("{RDF}type"),
                RdfNode::bf_class(classes::ISBN),
            );

            self.graph.add(
                id_node.clone(),
                format!("{RDF}value"),
                RdfNode::literal(&isbn_subfield.value),
            );

            // $q contains qualifier (e.g., "hardcover", "pbk.")
            if let Some(qualifier) = field.subfields.iter().find(|s| s.code == 'q') {
                self.graph.add(
                    id_node.clone(),
                    format!("{BF}qualifier"),
                    RdfNode::literal(&qualifier.value),
                );
            }

            // $c contains terms of availability (price, etc.)
            if let Some(terms) = field.subfields.iter().find(|s| s.code == 'c') {
                self.graph.add(
                    id_node.clone(),
                    format!("{BF}acquisitionTerms"),
                    RdfNode::literal(&terms.value),
                );
            }

            self.graph.add(
                instance.clone(),
                format!("{BF}{}", properties::IDENTIFIED_BY),
                id_node,
            );
        }

        // $z contains canceled/invalid ISBN
        for invalid in field.subfields.iter().filter(|s| s.code == 'z') {
            let id_node = self.graph.new_blank_node();

            self.graph.add(
                id_node.clone(),
                format!("{RDF}type"),
                RdfNode::bf_class(classes::ISBN),
            );

            self.graph.add(
                id_node.clone(),
                format!("{RDF}value"),
                RdfNode::literal(&invalid.value),
            );

            self.graph.add(
                id_node.clone(),
                format!("{BF}status"),
                RdfNode::literal("invalid"),
            );

            self.graph.add(
                instance.clone(),
                format!("{BF}{}", properties::IDENTIFIED_BY),
                id_node,
            );
        }
    }

    /// Adds an ISSN identifier with linking ISSN handling.
    fn add_issn(&mut self, instance: &RdfNode, field: &Field) {
        // $a contains the ISSN
        if let Some(issn_subfield) = field.subfields.iter().find(|s| s.code == 'a') {
            let id_node = self.graph.new_blank_node();

            self.graph.add(
                id_node.clone(),
                format!("{RDF}type"),
                RdfNode::bf_class(classes::ISSN),
            );

            self.graph.add(
                id_node.clone(),
                format!("{RDF}value"),
                RdfNode::literal(&issn_subfield.value),
            );

            self.graph.add(
                instance.clone(),
                format!("{BF}{}", properties::IDENTIFIED_BY),
                id_node,
            );
        }

        // $l contains linking ISSN (ISSN-L)
        if let Some(linking) = field.subfields.iter().find(|s| s.code == 'l') {
            let id_node = self.graph.new_blank_node();

            // Use BFLC:IssnL if BFLC is enabled, otherwise generic ISSN with note
            if self.config.include_bflc {
                self.graph.add(
                    id_node.clone(),
                    format!("{RDF}type"),
                    RdfNode::uri(format!("{BFLC}IssnL")),
                );
            } else {
                self.graph.add(
                    id_node.clone(),
                    format!("{RDF}type"),
                    RdfNode::bf_class(classes::ISSN),
                );
                self.graph.add(
                    id_node.clone(),
                    format!("{BF}{}", properties::NOTE),
                    RdfNode::literal("Linking ISSN"),
                );
            }

            self.graph.add(
                id_node.clone(),
                format!("{RDF}value"),
                RdfNode::literal(&linking.value),
            );

            self.graph.add(
                instance.clone(),
                format!("{BF}{}", properties::IDENTIFIED_BY),
                id_node,
            );
        }

        // $y contains incorrect ISSN
        for incorrect in field.subfields.iter().filter(|s| s.code == 'y') {
            let id_node = self.graph.new_blank_node();

            self.graph.add(
                id_node.clone(),
                format!("{RDF}type"),
                RdfNode::bf_class(classes::ISSN),
            );

            self.graph.add(
                id_node.clone(),
                format!("{RDF}value"),
                RdfNode::literal(&incorrect.value),
            );

            self.graph.add(
                id_node.clone(),
                format!("{BF}status"),
                RdfNode::literal("incorrect"),
            );

            self.graph.add(
                instance.clone(),
                format!("{BF}{}", properties::IDENTIFIED_BY),
                id_node,
            );
        }

        // $z contains canceled ISSN
        for canceled in field.subfields.iter().filter(|s| s.code == 'z') {
            let id_node = self.graph.new_blank_node();

            self.graph.add(
                id_node.clone(),
                format!("{RDF}type"),
                RdfNode::bf_class(classes::ISSN),
            );

            self.graph.add(
                id_node.clone(),
                format!("{RDF}value"),
                RdfNode::literal(&canceled.value),
            );

            self.graph.add(
                id_node.clone(),
                format!("{BF}status"),
                RdfNode::literal("canceled"),
            );

            self.graph.add(
                instance.clone(),
                format!("{BF}{}", properties::IDENTIFIED_BY),
                id_node,
            );
        }
    }

    /// Adds other standard identifiers (024 field) with source handling.
    fn add_other_identifier(&mut self, instance: &RdfNode, field: &Field) {
        let id_node = self.graph.new_blank_node();

        // Determine type from first indicator
        let id_type = match field.indicator1 {
            '0' => "Isrc", // International Standard Recording Code
            '1' => "Upc",  // Universal Product Code
            '2' => "Ismn", // International Standard Music Number
            '3' => "Ean",  // International Article Number
            '4' => "Sici", // Serial Item and Contribution Identifier
            '7' => {
                // Source specified in $2
                field
                    .subfields
                    .iter()
                    .find(|s| s.code == '2')
                    .map_or("Identifier", |s| s.value.as_str())
            },
            // '8' and others: Unspecified type
            _ => "Identifier",
        };

        // For well-known types, use bf:class; otherwise use literal type
        match id_type {
            "Isrc" | "Upc" | "Ismn" | "Ean" | "Sici" => {
                self.graph.add(
                    id_node.clone(),
                    format!("{RDF}type"),
                    RdfNode::bf_class(id_type),
                );
            },
            _ => {
                self.graph.add(
                    id_node.clone(),
                    format!("{RDF}type"),
                    RdfNode::bf_class("Identifier"),
                );
                // Add source as separate property
                if field.indicator1 == '7' {
                    if let Some(source) = field.subfields.iter().find(|s| s.code == '2') {
                        self.graph.add(
                            id_node.clone(),
                            format!("{BF}source"),
                            RdfNode::literal(&source.value),
                        );
                    }
                }
            },
        }

        // $a contains the identifier value
        if let Some(value) = field.subfields.iter().find(|s| s.code == 'a') {
            self.graph.add(
                id_node.clone(),
                format!("{RDF}value"),
                RdfNode::literal(&value.value),
            );
        }

        // $c contains terms of availability
        if let Some(terms) = field.subfields.iter().find(|s| s.code == 'c') {
            self.graph.add(
                id_node.clone(),
                format!("{BF}acquisitionTerms"),
                RdfNode::literal(&terms.value),
            );
        }

        // $d contains additional codes
        for code in field.subfields.iter().filter(|s| s.code == 'd') {
            self.graph.add(
                id_node.clone(),
                format!("{BF}qualifier"),
                RdfNode::literal(&code.value),
            );
        }

        // $z contains canceled/invalid identifier
        for invalid in field.subfields.iter().filter(|s| s.code == 'z') {
            let inv_node = self.graph.new_blank_node();
            self.graph.add(
                inv_node.clone(),
                format!("{RDF}type"),
                RdfNode::bf_class("Identifier"),
            );
            self.graph.add(
                inv_node.clone(),
                format!("{RDF}value"),
                RdfNode::literal(&invalid.value),
            );
            self.graph.add(
                inv_node.clone(),
                format!("{BF}status"),
                RdfNode::literal("invalid"),
            );
            self.graph.add(
                instance.clone(),
                format!("{BF}{}", properties::IDENTIFIED_BY),
                inv_node,
            );
        }

        self.graph.add(
            instance.clone(),
            format!("{BF}{}", properties::IDENTIFIED_BY),
            id_node,
        );
    }

    /// Adds system control number (035 field) with prefix parsing.
    fn add_system_control_number(&mut self, instance: &RdfNode, field: &Field) {
        if let Some(value) = field.subfields.iter().find(|s| s.code == 'a') {
            let id_node = self.graph.new_blank_node();

            self.graph.add(
                id_node.clone(),
                format!("{RDF}type"),
                RdfNode::bf_class(classes::LOCAL),
            );

            // Parse prefix in parentheses, e.g., "(OCoLC)12345678"
            let (source, number) = if value.value.starts_with('(') {
                if let Some(close_paren) = value.value.find(')') {
                    let source = &value.value[1..close_paren];
                    let number = &value.value[close_paren + 1..];
                    (Some(source), number)
                } else {
                    (None, value.value.as_str())
                }
            } else {
                (None, value.value.as_str())
            };

            self.graph.add(
                id_node.clone(),
                format!("{RDF}value"),
                RdfNode::literal(number),
            );

            // Add source if prefix was found
            if let Some(src) = source {
                self.graph.add(
                    id_node.clone(),
                    format!("{BF}source"),
                    RdfNode::literal(src),
                );
            }

            self.graph.add(
                instance.clone(),
                format!("{BF}{}", properties::IDENTIFIED_BY),
                id_node,
            );
        }

        // $z contains canceled/invalid control number
        for canceled in field.subfields.iter().filter(|s| s.code == 'z') {
            let id_node = self.graph.new_blank_node();

            self.graph.add(
                id_node.clone(),
                format!("{RDF}type"),
                RdfNode::bf_class(classes::LOCAL),
            );

            self.graph.add(
                id_node.clone(),
                format!("{RDF}value"),
                RdfNode::literal(&canceled.value),
            );

            self.graph.add(
                id_node.clone(),
                format!("{BF}status"),
                RdfNode::literal("canceled"),
            );

            self.graph.add(
                instance.clone(),
                format!("{BF}{}", properties::IDENTIFIED_BY),
                id_node,
            );
        }
    }

    /// Adds a basic identifier to the instance (used for LCCN and other simple cases).
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

    /// Processes classification fields (050, 060, 080, 082).
    /// Classification in BIBFRAME is linked to Work, not Instance.
    fn process_classification(&mut self) {
        let work = match &self.work_node {
            Some(n) => n.clone(),
            None => return,
        };

        // 050 - Library of Congress Classification
        if let Some(fields) = self.record.fields.get("050") {
            for field in fields {
                self.add_classification(&work, field, classes::CLASSIFICATION_LCC);
            }
        }

        // 060 - National Library of Medicine Classification
        if let Some(fields) = self.record.fields.get("060") {
            for field in fields {
                self.add_classification(&work, field, classes::CLASSIFICATION_NLM);
            }
        }

        // 080 - Universal Decimal Classification
        if let Some(fields) = self.record.fields.get("080") {
            for field in fields {
                self.add_classification(&work, field, classes::CLASSIFICATION_UDC);
            }
        }

        // 082 - Dewey Decimal Classification
        if let Some(fields) = self.record.fields.get("082") {
            for field in fields {
                self.add_classification(&work, field, classes::CLASSIFICATION_DDC);
            }
        }

        // 084 - Other Classification (generic)
        if let Some(fields) = self.record.fields.get("084") {
            for field in fields {
                self.add_classification(&work, field, classes::CLASSIFICATION);
            }
        }
    }

    /// Adds a classification node to the Work.
    fn add_classification(&mut self, work: &RdfNode, field: &Field, class_type: &str) {
        let class_node = self.graph.new_blank_node();

        // Add type
        self.graph.add(
            class_node.clone(),
            format!("{RDF}type"),
            RdfNode::bf_class(class_type),
        );

        // $a - Classification number (classificationPortion)
        if let Some(class_num) = field.subfields.iter().find(|s| s.code == 'a') {
            self.graph.add(
                class_node.clone(),
                format!("{BF}{}", properties::CLASSIFICATION_PORTION),
                RdfNode::literal(&class_num.value),
            );
        }

        // $b - Item number/Cutter (itemPortion)
        if let Some(item_num) = field.subfields.iter().find(|s| s.code == 'b') {
            self.graph.add(
                class_node.clone(),
                format!("{BF}{}", properties::ITEM_PORTION),
                RdfNode::literal(&item_num.value),
            );
        }

        // $2 - Source of classification (for 084 Other Classification)
        if let Some(source) = field.subfields.iter().find(|s| s.code == '2') {
            self.graph.add(
                class_node.clone(),
                format!("{BF}{}", properties::SOURCE),
                RdfNode::literal(&source.value),
            );
        }

        // Link classification to Work
        self.graph.add(
            work.clone(),
            format!("{BF}{}", properties::CLASSIFICATION),
            class_node,
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
                    },
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
                },
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
                },
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
                },
                _ => {},
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
                        },
                        'c' => {
                            // Dimensions
                            self.graph.add(
                                instance.clone(),
                                format!("{BF}{}", properties::DIMENSIONS),
                                RdfNode::literal(&subfield.value),
                            );
                        },
                        _ => {},
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

    /// Processes holdings fields (852, 876-878) to create bf:Item entities.
    /// Items represent specific copies of an Instance.
    #[allow(clippy::too_many_lines)]
    fn process_holdings(&mut self) {
        let instance = match &self.instance_node {
            Some(n) => n.clone(),
            None => return,
        };

        // Check if any holdings fields exist
        let has_holdings = self.record.fields.get("852").is_some()
            || self.record.fields.get("876").is_some()
            || self.record.fields.get("877").is_some()
            || self.record.fields.get("878").is_some();

        if !has_holdings {
            return;
        }

        // Process 852 - Location fields (primary source for Items)
        if let Some(fields) = self.record.fields.get("852") {
            for (idx, field) in fields.iter().enumerate() {
                let item_node = self.generate_item_uri(idx);

                // Add rdf:type bf:Item
                self.graph.add(
                    item_node.clone(),
                    format!("{RDF}type"),
                    RdfNode::bf_class(classes::ITEM),
                );

                // $a - Location (institution)
                if let Some(loc) = field.subfields.iter().find(|s| s.code == 'a') {
                    self.graph.add(
                        item_node.clone(),
                        format!("{BF}heldBy"),
                        RdfNode::literal(&loc.value),
                    );
                }

                // $b - Sublocation or collection
                if let Some(subloc) = field.subfields.iter().find(|s| s.code == 'b') {
                    self.graph.add(
                        item_node.clone(),
                        format!("{BF}subLocation"),
                        RdfNode::literal(&subloc.value),
                    );
                }

                // $h, $i, $j, $k, $l, $m - Call number components
                let call_parts: Vec<&str> = field
                    .subfields
                    .iter()
                    .filter(|s| matches!(s.code, 'h' | 'i' | 'j' | 'k' | 'l' | 'm'))
                    .map(|s| s.value.as_str())
                    .collect();
                if !call_parts.is_empty() {
                    self.graph.add(
                        item_node.clone(),
                        format!("{BF}shelfMark"),
                        RdfNode::literal(call_parts.join(" ")),
                    );
                }

                // $p - Barcode
                if let Some(barcode) = field.subfields.iter().find(|s| s.code == 'p') {
                    self.graph.add(
                        item_node.clone(),
                        format!("{BF}itemOf"),
                        RdfNode::literal(&barcode.value),
                    );
                    // Also add as identifier
                    let id_node = self.graph.new_blank_node();
                    self.graph.add(
                        id_node.clone(),
                        format!("{RDF}type"),
                        RdfNode::uri(format!("{BF}Barcode")),
                    );
                    self.graph.add(
                        id_node.clone(),
                        format!("{RDF}value"),
                        RdfNode::literal(&barcode.value),
                    );
                    self.graph.add(
                        item_node.clone(),
                        format!("{BF}{}", properties::IDENTIFIED_BY),
                        id_node,
                    );
                }

                // $x - Nonpublic note
                if let Some(note) = field.subfields.iter().find(|s| s.code == 'x') {
                    self.graph.add(
                        item_node.clone(),
                        format!("{BF}note"),
                        RdfNode::literal(&note.value),
                    );
                }

                // $z - Public note
                if let Some(note) = field.subfields.iter().find(|s| s.code == 'z') {
                    self.graph.add(
                        item_node.clone(),
                        format!("{BF}note"),
                        RdfNode::literal(&note.value),
                    );
                }

                // Link Instance to Item
                self.graph.add(
                    instance.clone(),
                    format!("{BF}{}", properties::HAS_ITEM),
                    item_node.clone(),
                );
                self.graph.add(
                    item_node.clone(),
                    format!("{BF}{}", properties::ITEM_OF),
                    instance.clone(),
                );

                self.item_nodes.push(item_node);
            }
        }

        // Process 876 - Item Information (basic)
        // These often supplement 852 or provide additional item details
        if let Some(fields) = self.record.fields.get("876") {
            for field in fields {
                // Try to find associated item or create new one
                let item_node = if self.item_nodes.is_empty() {
                    let node = self.generate_item_uri(0);
                    self.graph.add(
                        node.clone(),
                        format!("{RDF}type"),
                        RdfNode::bf_class(classes::ITEM),
                    );
                    self.graph.add(
                        instance.clone(),
                        format!("{BF}{}", properties::HAS_ITEM),
                        node.clone(),
                    );
                    self.item_nodes.push(node.clone());
                    node
                } else {
                    self.item_nodes[0].clone()
                };

                // $a - Internal item number
                if let Some(num) = field.subfields.iter().find(|s| s.code == 'a') {
                    let id_node = self.graph.new_blank_node();
                    self.graph.add(
                        id_node.clone(),
                        format!("{RDF}type"),
                        RdfNode::bf_class(classes::LOCAL),
                    );
                    self.graph.add(
                        id_node.clone(),
                        format!("{RDF}value"),
                        RdfNode::literal(&num.value),
                    );
                    self.graph.add(
                        item_node.clone(),
                        format!("{BF}{}", properties::IDENTIFIED_BY),
                        id_node,
                    );
                }

                // $c - Cost
                if let Some(cost) = field.subfields.iter().find(|s| s.code == 'c') {
                    self.graph.add(
                        item_node.clone(),
                        format!("{BF}acquisitionSource"),
                        RdfNode::literal(&cost.value),
                    );
                }

                // $d - Date acquired
                if let Some(date) = field.subfields.iter().find(|s| s.code == 'd') {
                    self.graph.add(
                        item_node.clone(),
                        format!("{BF}acquisitionDate"),
                        RdfNode::literal(&date.value),
                    );
                }

                // $j - Item status
                if let Some(status) = field.subfields.iter().find(|s| s.code == 'j') {
                    self.graph.add(
                        item_node.clone(),
                        format!("{BF}status"),
                        RdfNode::literal(&status.value),
                    );
                }
            }
        }
    }

    /// Generates a URI or blank node for an Item entity.
    fn generate_item_uri(&mut self, seq: usize) -> RdfNode {
        if let Some(ref base) = self.config.base_uri {
            let id = if self.config.use_control_number {
                self.record
                    .control_fields
                    .get("001")
                    .map_or("unknown", String::as_str)
            } else {
                "unknown"
            };
            RdfNode::uri(format!("{base}item/{id}-{seq}"))
        } else {
            self.graph.new_blank_node()
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

    // ========================================================================
    // Edge Case Handling (mrrc-uab.4.4)
    // ========================================================================

    /// Processes 880 linked fields (alternate script representations).
    ///
    /// 880 fields contain the same data as their linked field, but in an alternate
    /// script (e.g., Japanese, Cyrillic). The $6 subfield links to the original
    /// field with format: "TAG-occurrence/script".
    fn process_880_linked_fields(&mut self) {
        let instance = match &self.instance_node {
            Some(n) => n.clone(),
            None => return,
        };
        let work = match &self.work_node {
            Some(n) => n.clone(),
            None => return,
        };

        if let Some(fields) = self.record.fields.get("880") {
            for field in fields {
                // Extract the linked tag from $6 (format: "TAG-occurrence/script")
                let linked_tag = field
                    .subfields
                    .iter()
                    .find(|s| s.code == '6')
                    .map_or("", |s| &s.value[..3.min(s.value.len())]);

                // Determine language tag from script code if present
                let lang_tag = self.extract_language_from_880(field);

                match linked_tag {
                    // Title fields
                    "245" | "246" | "247" => {
                        self.add_880_title(&instance, field, lang_tag.as_deref());
                    },
                    // Edition statement
                    "250" => {
                        if let Some(subfield_a) = field.subfields.iter().find(|s| s.code == 'a') {
                            let node = if let Some(lang) = lang_tag {
                                RdfNode::literal_with_lang(&subfield_a.value, lang)
                            } else {
                                RdfNode::literal(&subfield_a.value)
                            };
                            self.graph.add(
                                instance.clone(),
                                format!("{BF}{}", properties::EDITION_STATEMENT),
                                node,
                            );
                        }
                    },
                    // Publication fields
                    "260" | "264" => {
                        self.add_880_provision(&instance, field, lang_tag.as_deref());
                    },
                    // Series statement
                    "490" => {
                        self.add_880_series(&instance, field, lang_tag.as_deref());
                    },
                    // Note fields (5XX)
                    tag if tag.starts_with('5') => {
                        self.add_880_note(&instance, field, lang_tag.as_deref());
                    },
                    // Subject fields (6XX) - link to Work
                    tag if tag.starts_with('6') => {
                        self.add_880_subject(&work, field, lang_tag.as_deref());
                    },
                    // Added entry fields (7XX)
                    "740" => {
                        self.add_880_related_title(&instance, field, lang_tag.as_deref());
                    },
                    // Linking fields (78X)
                    "780" | "785" | "787" => {
                        self.add_880_linking(&instance, field, linked_tag, lang_tag.as_deref());
                    },
                    _ => {
                        // For unhandled linked fields, just skip with no action
                    },
                }
            }
        }
    }

    /// Extracts language tag from 880 field's $6 subfield.
    fn extract_language_from_880(&self, field: &Field) -> Option<String> {
        field
            .subfields
            .iter()
            .find(|s| s.code == '6')
            .and_then(|s| {
                // Format: "TAG-occurrence/script" or "TAG-occurrence"
                if let Some(slash_pos) = s.value.find('/') {
                    let script = &s.value[slash_pos + 1..];
                    // Map MARC-8 script codes to language tags
                    match script {
                        "(3" | "arab" => Some("ar".to_string()),
                        "(N" | "cyrl" => Some("ru".to_string()), // Cyrillic -> Russian (could be others)
                        "hang" => Some("ko".to_string()),        // Korean Hangul
                        "hani" => Some("zh".to_string()),        // CJK ideographs -> Chinese
                        "jpan" => Some("ja".to_string()),        // Japanese
                        "(2" | "hebr" => Some("he".to_string()),
                        "(S" | "grek" => Some("el".to_string()),
                        // Latin "(B"/"latn" is default, "$1" CJK detects from content, others fallthrough
                        _ => None,
                    }
                    .or_else(|| self.detect_script_from_content(field))
                } else {
                    // Try to detect script from content
                    self.detect_script_from_content(field)
                }
            })
    }

    /// Attempts to detect script from field content.
    #[allow(clippy::unused_self)]
    fn detect_script_from_content(&self, field: &Field) -> Option<String> {
        let text: String = field
            .subfields
            .iter()
            .filter(|s| s.code != '6') // Skip the linking subfield
            .map(|s| &s.value[..])
            .collect();

        // Simple heuristic based on Unicode ranges
        for ch in text.chars() {
            match ch {
                '\u{3040}'..='\u{309F}' | '\u{30A0}'..='\u{30FF}' => return Some("ja".to_string()), // Hiragana/Katakana
                '\u{AC00}'..='\u{D7AF}' => return Some("ko".to_string()), // Hangul
                '\u{4E00}'..='\u{9FFF}' => return Some("zh".to_string()), // CJK (default to Chinese)
                '\u{0400}'..='\u{04FF}' => return Some("ru".to_string()), // Cyrillic
                '\u{0590}'..='\u{05FF}' => return Some("he".to_string()), // Hebrew
                '\u{0600}'..='\u{06FF}' => return Some("ar".to_string()), // Arabic
                '\u{0370}'..='\u{03FF}' => return Some("el".to_string()), // Greek
                _ => {},
            }
        }
        None
    }

    /// Adds an 880 title variant.
    fn add_880_title(&mut self, instance: &RdfNode, field: &Field, lang: Option<&str>) {
        let title_node = self.graph.new_blank_node();

        self.graph.add(
            title_node.clone(),
            format!("{RDF}type"),
            RdfNode::bf_class(classes::TITLE),
        );

        for subfield in &field.subfields {
            if subfield.code == '6' {
                continue; // Skip linking subfield
            }

            let node = if let Some(l) = lang {
                RdfNode::literal_with_lang(&subfield.value, l)
            } else {
                RdfNode::literal(&subfield.value)
            };

            match subfield.code {
                'a' => {
                    self.graph.add(
                        title_node.clone(),
                        format!("{BF}{}", properties::MAIN_TITLE),
                        node,
                    );
                },
                'b' => {
                    self.graph.add(
                        title_node.clone(),
                        format!("{BF}{}", properties::SUBTITLE),
                        node,
                    );
                },
                _ => {},
            }
        }

        self.graph.add(
            instance.clone(),
            format!("{BF}{}", properties::TITLE),
            title_node,
        );
    }

    /// Adds an 880 provision activity (publication info in alternate script).
    fn add_880_provision(&mut self, instance: &RdfNode, field: &Field, lang: Option<&str>) {
        let activity_node = self.graph.new_blank_node();

        self.graph.add(
            activity_node.clone(),
            format!("{RDF}type"),
            RdfNode::bf_class(classes::PUBLICATION),
        );

        for subfield in &field.subfields {
            if subfield.code == '6' {
                continue;
            }

            let node = if let Some(l) = lang {
                RdfNode::literal_with_lang(&subfield.value, l)
            } else {
                RdfNode::literal(&subfield.value)
            };

            match subfield.code {
                'a' => {
                    if self.config.include_bflc {
                        self.graph
                            .add(activity_node.clone(), format!("{BFLC}simplePlace"), node);
                    }
                },
                'b' => {
                    if self.config.include_bflc {
                        self.graph
                            .add(activity_node.clone(), format!("{BFLC}simpleAgent"), node);
                    }
                },
                'c' => {
                    if self.config.include_bflc {
                        self.graph
                            .add(activity_node.clone(), format!("{BFLC}simpleDate"), node);
                    }
                },
                _ => {},
            }
        }

        self.graph.add(
            instance.clone(),
            format!("{BF}{}", properties::PROVISION_ACTIVITY),
            activity_node,
        );
    }

    /// Adds an 880 series statement.
    fn add_880_series(&mut self, instance: &RdfNode, field: &Field, lang: Option<&str>) {
        if let Some(subfield_a) = field.subfields.iter().find(|s| s.code == 'a') {
            let node = if let Some(l) = lang {
                RdfNode::literal_with_lang(&subfield_a.value, l)
            } else {
                RdfNode::literal(&subfield_a.value)
            };
            self.graph
                .add(instance.clone(), format!("{BF}seriesStatement"), node);
        }
    }

    /// Adds an 880 note.
    fn add_880_note(&mut self, instance: &RdfNode, field: &Field, lang: Option<&str>) {
        if let Some(subfield_a) = field.subfields.iter().find(|s| s.code == 'a') {
            let node = if let Some(l) = lang {
                RdfNode::literal_with_lang(&subfield_a.value, l)
            } else {
                RdfNode::literal(&subfield_a.value)
            };
            self.graph
                .add(instance.clone(), format!("{BF}{}", properties::NOTE), node);
        }
    }

    /// Adds an 880 subject.
    fn add_880_subject(&mut self, work: &RdfNode, field: &Field, lang: Option<&str>) {
        let subject_node = self.graph.new_blank_node();

        self.graph.add(
            subject_node.clone(),
            format!("{RDF}type"),
            RdfNode::bf_class(classes::TOPIC),
        );

        // Build label from subfields
        let mut label_parts = Vec::new();
        for subfield in &field.subfields {
            if subfield.code != '6' && subfield.code != '0' && subfield.code != '1' {
                label_parts.push(subfield.value.clone());
            }
        }

        if !label_parts.is_empty() {
            let label = label_parts.join(" ");
            let node = if let Some(l) = lang {
                RdfNode::literal_with_lang(&label, l)
            } else {
                RdfNode::literal(&label)
            };
            self.graph
                .add(subject_node.clone(), format!("{RDFS}label"), node);
        }

        self.graph.add(
            work.clone(),
            format!("{BF}{}", properties::SUBJECT),
            subject_node,
        );
    }

    /// Adds an 880 related title (740 field).
    fn add_880_related_title(&mut self, instance: &RdfNode, field: &Field, lang: Option<&str>) {
        if let Some(subfield_a) = field.subfields.iter().find(|s| s.code == 'a') {
            let title_node = self.graph.new_blank_node();

            self.graph.add(
                title_node.clone(),
                format!("{RDF}type"),
                RdfNode::bf_class(classes::TITLE),
            );

            let node = if let Some(l) = lang {
                RdfNode::literal_with_lang(&subfield_a.value, l)
            } else {
                RdfNode::literal(&subfield_a.value)
            };

            self.graph.add(
                title_node.clone(),
                format!("{BF}{}", properties::MAIN_TITLE),
                node,
            );

            self.graph.add(
                instance.clone(),
                format!("{BF}{}", properties::TITLE),
                title_node,
            );
        }
    }

    /// Adds an 880 linking entry (780/785/787 in alternate script).
    fn add_880_linking(
        &mut self,
        instance: &RdfNode,
        field: &Field,
        linked_tag: &str,
        lang: Option<&str>,
    ) {
        let related_node = self.graph.new_blank_node();

        self.graph.add(
            related_node.clone(),
            format!("{RDF}type"),
            RdfNode::bf_class(classes::INSTANCE),
        );

        // Add title from $t
        if let Some(subfield_t) = field.subfields.iter().find(|s| s.code == 't') {
            let title_node = self.graph.new_blank_node();
            self.graph.add(
                title_node.clone(),
                format!("{RDF}type"),
                RdfNode::bf_class(classes::TITLE),
            );

            let node = if let Some(l) = lang {
                RdfNode::literal_with_lang(&subfield_t.value, l)
            } else {
                RdfNode::literal(&subfield_t.value)
            };

            self.graph.add(
                title_node.clone(),
                format!("{BF}{}", properties::MAIN_TITLE),
                node,
            );
            self.graph.add(
                related_node.clone(),
                format!("{BF}{}", properties::TITLE),
                title_node,
            );
        }

        // Determine relationship type
        let relationship = match linked_tag {
            "780" => "precededBy",
            "785" => "succeededBy",
            _ => "relatedTo",
        };

        self.graph.add(
            instance.clone(),
            format!("{BF}{relationship}"),
            related_node,
        );
    }

    /// Processes 76X-78X linking entry fields.
    ///
    /// These fields describe relationships between bibliographic entities:
    /// - 760: Main series entry
    /// - 762: Subseries entry
    /// - 765: Original language entry
    /// - 767: Translation entry
    /// - 770: Supplement/special issue entry
    /// - 772: Parent record entry
    /// - 773: Host item entry (part of)
    /// - 774: Constituent unit entry
    /// - 775: Other edition entry
    /// - 776: Additional physical form entry
    /// - 777: Issued with entry
    /// - 780: Preceding entry
    /// - 785: Succeeding entry
    /// - 786: Data source entry
    /// - 787: Nonspecific relationship entry
    #[allow(clippy::too_many_lines)]
    fn process_linking_entries(&mut self) {
        let instance = match &self.instance_node {
            Some(n) => n.clone(),
            None => return,
        };
        let work = match &self.work_node {
            Some(n) => n.clone(),
            None => return,
        };

        let linking_tags = [
            ("760", "hasSeries", true),           // Main series
            ("762", "hasSubseries", true),        // Subseries
            ("765", "translationOf", false),      // Original language
            ("767", "hasTranslation", false),     // Translation
            ("770", "supplement", true),          // Supplement
            ("772", "supplementTo", true),        // Parent (supplement to)
            ("773", "partOf", true),              // Host item
            ("774", "hasPart", true),             // Constituent unit
            ("775", "otherEdition", true),        // Other edition
            ("776", "otherPhysicalFormat", true), // Additional physical form
            ("777", "issuedWith", true),          // Issued with
            ("780", "precededBy", true),          // Preceding
            ("785", "succeededBy", true),         // Succeeding
            ("786", "dataSource", false),         // Data source
            ("787", "relatedTo", false),          // Nonspecific
        ];

        for (tag, relationship, is_instance_rel) in linking_tags {
            if let Some(fields) = self.record.fields.get(tag) {
                for field in fields {
                    let related_node = self.graph.new_blank_node();

                    // Determine if linking to Work or Instance
                    let related_type = if is_instance_rel {
                        classes::INSTANCE
                    } else {
                        classes::WORK
                    };

                    self.graph.add(
                        related_node.clone(),
                        format!("{RDF}type"),
                        RdfNode::bf_class(related_type),
                    );

                    // Add title from $t
                    if let Some(title) = field.subfields.iter().find(|s| s.code == 't') {
                        let title_node = self.graph.new_blank_node();
                        self.graph.add(
                            title_node.clone(),
                            format!("{RDF}type"),
                            RdfNode::bf_class(classes::TITLE),
                        );
                        self.graph.add(
                            title_node.clone(),
                            format!("{BF}{}", properties::MAIN_TITLE),
                            RdfNode::literal(&title.value),
                        );
                        self.graph.add(
                            related_node.clone(),
                            format!("{BF}{}", properties::TITLE),
                            title_node,
                        );
                    }

                    // Add creator/contributor from $a
                    if let Some(agent) = field.subfields.iter().find(|s| s.code == 'a') {
                        let agent_node = self.graph.new_blank_node();
                        self.graph.add(
                            agent_node.clone(),
                            format!("{RDFS}label"),
                            RdfNode::literal(&agent.value),
                        );
                        self.graph.add(
                            related_node.clone(),
                            format!("{BF}{}", properties::CONTRIBUTION),
                            agent_node,
                        );
                    }

                    // Add identifiers
                    // $x = ISSN
                    if let Some(issn) = field.subfields.iter().find(|s| s.code == 'x') {
                        let id_node = self.graph.new_blank_node();
                        self.graph.add(
                            id_node.clone(),
                            format!("{RDF}type"),
                            RdfNode::bf_class(classes::ISSN),
                        );
                        self.graph.add(
                            id_node.clone(),
                            format!("{RDF}value"),
                            RdfNode::literal(&issn.value),
                        );
                        self.graph.add(
                            related_node.clone(),
                            format!("{BF}{}", properties::IDENTIFIED_BY),
                            id_node,
                        );
                    }

                    // $z = ISBN
                    if let Some(isbn) = field.subfields.iter().find(|s| s.code == 'z') {
                        let id_node = self.graph.new_blank_node();
                        self.graph.add(
                            id_node.clone(),
                            format!("{RDF}type"),
                            RdfNode::bf_class(classes::ISBN),
                        );
                        self.graph.add(
                            id_node.clone(),
                            format!("{RDF}value"),
                            RdfNode::literal(&isbn.value),
                        );
                        self.graph.add(
                            related_node.clone(),
                            format!("{BF}{}", properties::IDENTIFIED_BY),
                            id_node,
                        );
                    }

                    // $w = Record control number (other system IDs)
                    for ctrl in field.subfields.iter().filter(|s| s.code == 'w') {
                        let id_node = self.graph.new_blank_node();
                        self.graph.add(
                            id_node.clone(),
                            format!("{RDF}type"),
                            RdfNode::bf_class(classes::LOCAL),
                        );
                        self.graph.add(
                            id_node.clone(),
                            format!("{RDF}value"),
                            RdfNode::literal(&ctrl.value),
                        );
                        self.graph.add(
                            related_node.clone(),
                            format!("{BF}{}", properties::IDENTIFIED_BY),
                            id_node,
                        );
                    }

                    // Add relationship note from $i (relationship info)
                    if let Some(rel_info) = field.subfields.iter().find(|s| s.code == 'i') {
                        self.graph.add(
                            related_node.clone(),
                            format!("{BF}{}", properties::NOTE),
                            RdfNode::literal(&rel_info.value),
                        );
                    }

                    // Link from source entity
                    let source = if is_instance_rel { &instance } else { &work };
                    self.graph
                        .add(source.clone(), format!("{BF}{relationship}"), related_node);
                }
            }
        }
    }

    /// Processes series fields (490 and 8XX).
    ///
    /// - 490: Series statement (transcribed)
    /// - 800: Series added entry - Personal name
    /// - 810: Series added entry - Corporate name
    /// - 811: Series added entry - Meeting name
    /// - 830: Series added entry - Uniform title
    #[allow(clippy::too_many_lines)]
    fn process_series(&mut self) {
        let instance = match &self.instance_node {
            Some(n) => n.clone(),
            None => return,
        };
        let work = match &self.work_node {
            Some(n) => n.clone(),
            None => return,
        };

        // 490 - Series Statement (transcribed form)
        if let Some(fields) = self.record.fields.get("490") {
            for field in fields {
                // Check first indicator: 0 = not traced, 1 = traced
                let is_traced = field.indicator1 == '1';

                if let Some(subfield_a) = field.subfields.iter().find(|s| s.code == 'a') {
                    // Always add series statement to Instance
                    self.graph.add(
                        instance.clone(),
                        format!("{BF}seriesStatement"),
                        RdfNode::literal(&subfield_a.value),
                    );

                    // If not traced (490 0_), also add as simple series
                    if !is_traced {
                        let series_node = self.graph.new_blank_node();
                        self.graph.add(
                            series_node.clone(),
                            format!("{RDF}type"),
                            RdfNode::bf_class(classes::WORK),
                        );
                        self.graph.add(
                            series_node.clone(),
                            format!("{RDFS}label"),
                            RdfNode::literal(&subfield_a.value),
                        );
                        self.graph
                            .add(work.clone(), format!("{BF}hasSeries"), series_node);
                    }
                }

                // Add ISSN from $x if present
                if let Some(issn) = field.subfields.iter().find(|s| s.code == 'x') {
                    let id_node = self.graph.new_blank_node();
                    self.graph.add(
                        id_node.clone(),
                        format!("{RDF}type"),
                        RdfNode::bf_class(classes::ISSN),
                    );
                    self.graph.add(
                        id_node.clone(),
                        format!("{RDF}value"),
                        RdfNode::literal(&issn.value),
                    );
                    self.graph
                        .add(instance.clone(), format!("{BF}seriesEnumeration"), id_node);
                }

                // Add volume number from $v
                if let Some(vol) = field.subfields.iter().find(|s| s.code == 'v') {
                    self.graph.add(
                        instance.clone(),
                        format!("{BF}seriesEnumeration"),
                        RdfNode::literal(&vol.value),
                    );
                }
            }
        }

        // 8XX - Series added entries (traced series)
        let series_tags = [
            ("800", classes::PERSON),       // Personal name
            ("810", classes::ORGANIZATION), // Corporate name
            ("811", classes::MEETING),      // Meeting name
            ("830", classes::WORK),         // Uniform title
        ];

        for (tag, agent_type) in series_tags {
            if let Some(fields) = self.record.fields.get(tag) {
                for field in fields {
                    let series_node = self.graph.new_blank_node();

                    // Series is a Work
                    self.graph.add(
                        series_node.clone(),
                        format!("{RDF}type"),
                        RdfNode::bf_class(classes::WORK),
                    );

                    // Build series title from $a and $t
                    let mut title_parts = Vec::new();
                    for subfield in &field.subfields {
                        match subfield.code {
                            'a' | 't' => title_parts.push(subfield.value.clone()),
                            _ => {},
                        }
                    }

                    if !title_parts.is_empty() {
                        let title_node = self.graph.new_blank_node();
                        self.graph.add(
                            title_node.clone(),
                            format!("{RDF}type"),
                            RdfNode::bf_class(classes::TITLE),
                        );
                        self.graph.add(
                            title_node.clone(),
                            format!("{BF}{}", properties::MAIN_TITLE),
                            RdfNode::literal(title_parts.join(". ")),
                        );
                        self.graph.add(
                            series_node.clone(),
                            format!("{BF}{}", properties::TITLE),
                            title_node,
                        );
                    }

                    // For 800/810/811, add the agent (series is "by" this agent)
                    if tag != "830" {
                        let agent_node = self.graph.new_blank_node();
                        self.graph.add(
                            agent_node.clone(),
                            format!("{RDF}type"),
                            RdfNode::bf_class(agent_type),
                        );

                        if let Some(name) = field.subfields.iter().find(|s| s.code == 'a') {
                            self.graph.add(
                                agent_node.clone(),
                                format!("{RDFS}label"),
                                RdfNode::literal(&name.value),
                            );
                        }

                        // Link as contribution
                        let contrib_node = self.graph.new_blank_node();
                        self.graph.add(
                            contrib_node.clone(),
                            format!("{RDF}type"),
                            RdfNode::bf_class(classes::CONTRIBUTION),
                        );
                        self.graph.add(
                            contrib_node.clone(),
                            format!("{BF}{}", properties::AGENT),
                            agent_node,
                        );
                        self.graph.add(
                            series_node.clone(),
                            format!("{BF}{}", properties::CONTRIBUTION),
                            contrib_node,
                        );
                    }

                    // Add volume/numbering from $v
                    if let Some(vol) = field.subfields.iter().find(|s| s.code == 'v') {
                        self.graph.add(
                            instance.clone(),
                            format!("{BF}seriesEnumeration"),
                            RdfNode::literal(&vol.value),
                        );
                    }

                    // Link Work to series
                    self.graph
                        .add(work.clone(), format!("{BF}hasSeries"), series_node);
                }
            }
        }
    }

    /// Processes format-specific fields for music, maps, serials, etc.
    fn process_format_specific_fields(&mut self) {
        let work = match &self.work_node {
            Some(n) => n.clone(),
            None => return,
        };
        let instance = match &self.instance_node {
            Some(n) => n.clone(),
            None => return,
        };

        // Determine format from Leader position 06
        match self.record.leader.record_type {
            'c' | 'd' | 'j' => self.process_music_fields(&work, &instance),
            'e' | 'f' => self.process_cartographic_fields(&work, &instance),
            _ => {},
        }

        // Serials (bibliographic level 's' or 'i')
        if matches!(self.record.leader.bibliographic_level, 's' | 'i') {
            self.process_serial_fields(&instance);
        }
    }

    /// Processes music-specific fields.
    fn process_music_fields(&mut self, work: &RdfNode, instance: &RdfNode) {
        // 382 - Medium of Performance
        if let Some(fields) = self.record.fields.get("382") {
            for field in fields {
                let medium_node = self.graph.new_blank_node();
                self.graph.add(
                    medium_node.clone(),
                    format!("{RDF}type"),
                    RdfNode::uri(format!("{BF}MusicMedium")),
                );

                // $a = Medium of performance
                for subfield in field.subfields.iter().filter(|s| s.code == 'a') {
                    self.graph.add(
                        medium_node.clone(),
                        format!("{RDFS}label"),
                        RdfNode::literal(&subfield.value),
                    );
                }

                // $n = Number of performers
                if let Some(count) = field.subfields.iter().find(|s| s.code == 'n') {
                    self.graph.add(
                        medium_node.clone(),
                        format!("{BF}count"),
                        RdfNode::literal(&count.value),
                    );
                }

                self.graph
                    .add(work.clone(), format!("{BF}musicMedium"), medium_node);
            }
        }

        // 384 - Key
        if let Some(fields) = self.record.fields.get("384") {
            for field in fields {
                if let Some(key) = field.subfields.iter().find(|s| s.code == 'a') {
                    self.graph.add(
                        work.clone(),
                        format!("{BF}musicKey"),
                        RdfNode::literal(&key.value),
                    );
                }
            }
        }

        // 348 - Format of Notated Music
        if let Some(fields) = self.record.fields.get("348") {
            for field in fields {
                if let Some(format) = field.subfields.iter().find(|s| s.code == 'a') {
                    self.graph.add(
                        instance.clone(),
                        format!("{BF}musicFormat"),
                        RdfNode::literal(&format.value),
                    );
                }
            }
        }
    }

    /// Processes cartographic (map) fields.
    fn process_cartographic_fields(&mut self, work: &RdfNode, instance: &RdfNode) {
        // 255 - Cartographic Mathematical Data
        if let Some(fields) = self.record.fields.get("255") {
            for field in fields {
                let carto_node = self.graph.new_blank_node();
                self.graph.add(
                    carto_node.clone(),
                    format!("{RDF}type"),
                    RdfNode::uri(format!("{BF}Cartographic")),
                );

                // $a = Scale statement
                if let Some(scale) = field.subfields.iter().find(|s| s.code == 'a') {
                    self.graph.add(
                        carto_node.clone(),
                        format!("{BF}scale"),
                        RdfNode::literal(&scale.value),
                    );
                }

                // $b = Projection
                if let Some(proj) = field.subfields.iter().find(|s| s.code == 'b') {
                    self.graph.add(
                        carto_node.clone(),
                        format!("{BF}projection"),
                        RdfNode::literal(&proj.value),
                    );
                }

                // $c = Coordinates
                if let Some(coords) = field.subfields.iter().find(|s| s.code == 'c') {
                    self.graph.add(
                        carto_node.clone(),
                        format!("{BF}coordinates"),
                        RdfNode::literal(&coords.value),
                    );
                }

                self.graph.add(
                    work.clone(),
                    format!("{BF}cartographicAttributes"),
                    carto_node,
                );
            }
        }

        // 342 - Geospatial Reference Data
        if let Some(fields) = self.record.fields.get("342") {
            for field in fields {
                if let Some(name) = field.subfields.iter().find(|s| s.code == 'a') {
                    self.graph.add(
                        instance.clone(),
                        format!("{BF}geographicCoverage"),
                        RdfNode::literal(&name.value),
                    );
                }
            }
        }
    }

    /// Processes serial-specific fields.
    fn process_serial_fields(&mut self, instance: &RdfNode) {
        // 310 - Current Publication Frequency
        if let Some(fields) = self.record.fields.get("310") {
            for field in fields {
                if let Some(freq) = field.subfields.iter().find(|s| s.code == 'a') {
                    self.graph.add(
                        instance.clone(),
                        format!("{BF}frequency"),
                        RdfNode::literal(&freq.value),
                    );
                }
            }
        }

        // 321 - Former Publication Frequency
        if let Some(fields) = self.record.fields.get("321") {
            for field in fields {
                if let Some(freq) = field.subfields.iter().find(|s| s.code == 'a') {
                    let freq_node = self.graph.new_blank_node();
                    self.graph.add(
                        freq_node.clone(),
                        format!("{RDFS}label"),
                        RdfNode::literal(&freq.value),
                    );

                    // Add date range if present
                    if let Some(dates) = field.subfields.iter().find(|s| s.code == 'b') {
                        self.graph.add(
                            freq_node.clone(),
                            format!("{BF}{}", properties::DATE),
                            RdfNode::literal(&dates.value),
                        );
                    }

                    self.graph
                        .add(instance.clone(), format!("{BF}frequency"), freq_node);
                }
            }
        }

        // 362 - Dates of Publication
        if let Some(fields) = self.record.fields.get("362") {
            for field in fields {
                if let Some(dates) = field.subfields.iter().find(|s| s.code == 'a') {
                    self.graph.add(
                        instance.clone(),
                        format!("{BF}firstIssue"),
                        RdfNode::literal(&dates.value),
                    );
                }
            }
        }
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

        let serialized = graph
            .serialize(super::super::config::RdfFormat::NTriples)
            .unwrap();
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

        let serialized = graph
            .serialize(super::super::config::RdfFormat::NTriples)
            .unwrap();
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

        let serialized = graph
            .serialize(super::super::config::RdfFormat::NTriples)
            .unwrap();
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

        let serialized = graph
            .serialize(super::super::config::RdfFormat::NTriples)
            .unwrap();
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

        let serialized = graph
            .serialize(super::super::config::RdfFormat::NTriples)
            .unwrap();
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

        let serialized = graph
            .serialize(super::super::config::RdfFormat::NTriples)
            .unwrap();
        assert!(serialized.contains("NotatedMusic"));
    }

    #[test]
    fn test_serial_instance_type() {
        let mut leader = make_test_leader();
        leader.bibliographic_level = 's'; // Serial
        let record = Record::new(leader);

        let config = BibframeConfig::default();
        let graph = convert_marc_to_bibframe(&record, &config);

        let serialized = graph
            .serialize(super::super::config::RdfFormat::NTriples)
            .unwrap();
        assert!(serialized.contains("Serial"));
    }

    #[test]
    fn test_base_uri_generation() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "rec123".to_string());

        let config = BibframeConfig::new().with_base_uri("http://example.org/");
        let graph = convert_marc_to_bibframe(&record, &config);

        let serialized = graph
            .serialize(super::super::config::RdfFormat::NTriples)
            .unwrap();
        assert!(serialized.contains("http://example.org/work/rec123"));
        assert!(serialized.contains("http://example.org/instance/rec123"));
    }

    // ========================================================================
    // Edge Case Tests (mrrc-uab.4.4)
    // ========================================================================

    #[test]
    fn test_880_alternate_script_title() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test880".to_string());

        // Add 245 in Latin script
        let mut field_245 = Field::new("245".to_string(), '0', '0');
        field_245.add_subfield('a', "Test Title".to_string());
        record.add_field(field_245);

        // Add 880 with Japanese alternate
        let mut field_880 = Field::new("880".to_string(), '0', '0');
        field_880.add_subfield('6', "245-00".to_string());
        field_880.add_subfield('a', "東海道五十三次".to_string());
        record.add_field(field_880);

        let config = BibframeConfig::default();
        let graph = convert_marc_to_bibframe(&record, &config);

        let serialized = graph
            .serialize(super::super::config::RdfFormat::NTriples)
            .unwrap();
        // Should have both titles
        assert!(serialized.contains("Test Title"));
        assert!(serialized.contains("東海道五十三次"));
    }

    #[test]
    fn test_linking_entry_780_preceding() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test780".to_string());

        // Add 780 - Preceding entry
        let mut field_780 = Field::new("780".to_string(), '0', '0');
        field_780.add_subfield('t', "Earlier Journal Title".to_string());
        field_780.add_subfield('x', "1234-5678".to_string());
        record.add_field(field_780);

        let config = BibframeConfig::default();
        let graph = convert_marc_to_bibframe(&record, &config);

        let serialized = graph
            .serialize(super::super::config::RdfFormat::NTriples)
            .unwrap();
        assert!(serialized.contains("precededBy"));
        assert!(serialized.contains("Earlier Journal Title"));
        assert!(serialized.contains("1234-5678"));
    }

    #[test]
    fn test_linking_entry_785_succeeding() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test785".to_string());

        // Add 785 - Succeeding entry
        let mut field_785 = Field::new("785".to_string(), '0', '0');
        field_785.add_subfield('t', "Later Journal Title".to_string());
        field_785.add_subfield('w', "(OCoLC)12345678".to_string());
        record.add_field(field_785);

        let config = BibframeConfig::default();
        let graph = convert_marc_to_bibframe(&record, &config);

        let serialized = graph
            .serialize(super::super::config::RdfFormat::NTriples)
            .unwrap();
        assert!(serialized.contains("succeededBy"));
        assert!(serialized.contains("Later Journal Title"));
    }

    #[test]
    fn test_series_490_untraced() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test490".to_string());

        // Add 490 - Series statement (untraced)
        let mut field_490 = Field::new("490".to_string(), '0', ' ');
        field_490.add_subfield('a', "Library science series".to_string());
        field_490.add_subfield('v', "vol. 5".to_string());
        record.add_field(field_490);

        let config = BibframeConfig::default();
        let graph = convert_marc_to_bibframe(&record, &config);

        let serialized = graph
            .serialize(super::super::config::RdfFormat::NTriples)
            .unwrap();
        assert!(serialized.contains("seriesStatement"));
        assert!(serialized.contains("Library science series"));
        assert!(serialized.contains("seriesEnumeration"));
    }

    #[test]
    fn test_series_830_traced() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test830".to_string());

        // Add 830 - Series added entry
        let mut field_830 = Field::new("830".to_string(), ' ', '0');
        field_830.add_subfield('a', "ACM monograph series".to_string());
        field_830.add_subfield('v', "no. 42".to_string());
        record.add_field(field_830);

        let config = BibframeConfig::default();
        let graph = convert_marc_to_bibframe(&record, &config);

        let serialized = graph
            .serialize(super::super::config::RdfFormat::NTriples)
            .unwrap();
        assert!(serialized.contains("hasSeries"));
        assert!(serialized.contains("ACM monograph series"));
    }

    #[test]
    fn test_isbn_with_qualifier() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test020q".to_string());

        // Add 020 with qualifier
        let mut field_020 = Field::new("020".to_string(), ' ', ' ');
        field_020.add_subfield('a', "9780123456789".to_string());
        field_020.add_subfield('q', "hardcover".to_string());
        record.add_field(field_020);

        let config = BibframeConfig::default();
        let graph = convert_marc_to_bibframe(&record, &config);

        let serialized = graph
            .serialize(super::super::config::RdfFormat::NTriples)
            .unwrap();
        assert!(serialized.contains("Isbn"));
        assert!(serialized.contains("9780123456789"));
        assert!(serialized.contains("qualifier"));
        assert!(serialized.contains("hardcover"));
    }

    #[test]
    fn test_isbn_invalid() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test020z".to_string());

        // Add 020 with invalid ISBN
        let mut field_020 = Field::new("020".to_string(), ' ', ' ');
        field_020.add_subfield('z', "9780000000000".to_string());
        record.add_field(field_020);

        let config = BibframeConfig::default();
        let graph = convert_marc_to_bibframe(&record, &config);

        let serialized = graph
            .serialize(super::super::config::RdfFormat::NTriples)
            .unwrap();
        assert!(serialized.contains("9780000000000"));
        assert!(serialized.contains("invalid"));
    }

    #[test]
    fn test_issn_with_linking() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test022l".to_string());

        // Add 022 with linking ISSN
        let mut field_022 = Field::new("022".to_string(), ' ', ' ');
        field_022.add_subfield('a', "1234-5678".to_string());
        field_022.add_subfield('l', "1111-2222".to_string());
        record.add_field(field_022);

        let config = BibframeConfig::default();
        let graph = convert_marc_to_bibframe(&record, &config);

        let serialized = graph
            .serialize(super::super::config::RdfFormat::NTriples)
            .unwrap();
        assert!(serialized.contains("1234-5678"));
        assert!(serialized.contains("1111-2222"));
        // With BFLC enabled, should have IssnL type
        assert!(serialized.contains("IssnL"));
    }

    #[test]
    fn test_035_with_prefix() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test035".to_string());

        // Add 035 with OCLC prefix
        let mut field_035 = Field::new("035".to_string(), ' ', ' ');
        field_035.add_subfield('a', "(OCoLC)12345678".to_string());
        record.add_field(field_035);

        let config = BibframeConfig::default();
        let graph = convert_marc_to_bibframe(&record, &config);

        let serialized = graph
            .serialize(super::super::config::RdfFormat::NTriples)
            .unwrap();
        // Should parse the prefix
        assert!(serialized.contains("12345678"));
        assert!(serialized.contains("OCoLC"));
    }

    #[test]
    fn test_music_format_fields() {
        let mut leader = make_test_leader();
        leader.record_type = 'c'; // Notated music
        let mut record = Record::new(leader);
        record.add_control_field("001".to_string(), "testmusic".to_string());

        // Add 382 - Medium of Performance
        let mut field_382 = Field::new("382".to_string(), ' ', ' ');
        field_382.add_subfield('a', "piano".to_string());
        field_382.add_subfield('n', "1".to_string());
        record.add_field(field_382);

        // Add 384 - Key
        let mut field_384 = Field::new("384".to_string(), ' ', ' ');
        field_384.add_subfield('a', "C major".to_string());
        record.add_field(field_384);

        let config = BibframeConfig::default();
        let graph = convert_marc_to_bibframe(&record, &config);

        let serialized = graph
            .serialize(super::super::config::RdfFormat::NTriples)
            .unwrap();
        assert!(serialized.contains("musicMedium"));
        assert!(serialized.contains("piano"));
        assert!(serialized.contains("musicKey"));
        assert!(serialized.contains("C major"));
    }

    #[test]
    fn test_cartographic_fields() {
        let mut leader = make_test_leader();
        leader.record_type = 'e'; // Cartographic
        let mut record = Record::new(leader);
        record.add_control_field("001".to_string(), "testmap".to_string());

        // Add 255 - Cartographic Mathematical Data
        let mut field_255 = Field::new("255".to_string(), ' ', ' ');
        field_255.add_subfield('a', "Scale 1:24,000".to_string());
        field_255.add_subfield('b', "Mercator proj.".to_string());
        record.add_field(field_255);

        let config = BibframeConfig::default();
        let graph = convert_marc_to_bibframe(&record, &config);

        let serialized = graph
            .serialize(super::super::config::RdfFormat::NTriples)
            .unwrap();
        assert!(serialized.contains("Cartographic"));
        assert!(serialized.contains("scale"));
        assert!(serialized.contains("1:24,000"));
        assert!(serialized.contains("projection"));
    }

    #[test]
    fn test_serial_frequency_fields() {
        let mut leader = make_test_leader();
        leader.bibliographic_level = 's'; // Serial
        let mut record = Record::new(leader);
        record.add_control_field("001".to_string(), "testserial".to_string());

        // Add 310 - Current Publication Frequency
        let mut field_310 = Field::new("310".to_string(), ' ', ' ');
        field_310.add_subfield('a', "Monthly".to_string());
        record.add_field(field_310);

        // Add 362 - Dates of Publication
        let mut field_362 = Field::new("362".to_string(), '0', ' ');
        field_362.add_subfield('a', "Vol. 1, no. 1 (Jan. 1990)-".to_string());
        record.add_field(field_362);

        let config = BibframeConfig::default();
        let graph = convert_marc_to_bibframe(&record, &config);

        let serialized = graph
            .serialize(super::super::config::RdfFormat::NTriples)
            .unwrap();
        assert!(serialized.contains("Serial"));
        assert!(serialized.contains("frequency"));
        assert!(serialized.contains("Monthly"));
        assert!(serialized.contains("firstIssue"));
    }

    #[test]
    fn test_024_with_source() {
        let mut record = Record::new(make_test_leader());
        record.add_control_field("001".to_string(), "test024".to_string());

        // Add 024 with ind1=7 (source in $2)
        let mut field_024 = Field::new("024".to_string(), '7', ' ');
        field_024.add_subfield('a', "10.1000/xyz123".to_string());
        field_024.add_subfield('2', "doi".to_string());
        record.add_field(field_024);

        let config = BibframeConfig::default();
        let graph = convert_marc_to_bibframe(&record, &config);

        let serialized = graph
            .serialize(super::super::config::RdfFormat::NTriples)
            .unwrap();
        assert!(serialized.contains("10.1000/xyz123"));
        assert!(serialized.contains("doi"));
    }
}
