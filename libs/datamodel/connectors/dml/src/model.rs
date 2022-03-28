use crate::default_value::DefaultKind;
use crate::field::{Field, FieldType, RelationField, ScalarField};
use crate::scalars::ScalarType;
use crate::traits::{Ignorable, WithDatabaseName, WithName};
use std::fmt;

/// Represents a model in a prisma schema.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Model {
    /// Name of the model.
    pub name: String,
    /// Fields of the model.
    pub fields: Vec<Field>,
    /// Comments associated with this model.
    pub documentation: Option<String>,
    /// The database internal name of this model.
    pub database_name: Option<String>,
    /// Describes Composite Indexes
    pub indices: Vec<IndexDefinition>,
    /// Describes the Primary Keys
    pub primary_key: Option<PrimaryKeyDefinition>,
    /// Indicates if this model is generated.
    pub is_generated: bool,
    /// Indicates if this model has to be commented out.
    pub is_commented_out: bool,
    /// Indicates if this model has to be ignored by the Client.
    pub is_ignored: bool,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IndexAlgorithm {
    BTree,
    Hash,
}

impl Default for IndexAlgorithm {
    fn default() -> Self {
        Self::BTree
    }
}

/// Represents an index defined via `@@index`, `@unique` or `@@unique`.
#[derive(Debug, PartialEq, Clone)]
pub struct IndexDefinition {
    pub name: Option<String>,
    pub db_name: Option<String>,
    pub fields: Vec<IndexField>,
    pub tpe: IndexType,
    pub algorithm: Option<IndexAlgorithm>,
    pub defined_on_field: bool,
}

impl IndexDefinition {
    pub fn is_unique(&self) -> bool {
        matches!(self.tpe, IndexType::Unique)
    }

    pub fn is_fulltext(&self) -> bool {
        matches!(self.tpe, IndexType::Fulltext)
    }
}

/// The field can either be directly in the same model, or in a composite type
/// embedded in the current model.
#[derive(Debug, PartialEq, Clone)]
pub enum IndexFieldLocation {
    InCurrentModel {
        field_name: String,
    },
    InCompositeType {
        composite_type_name: String,
        field_name: String,
        full_path: String,
    },
}

///A field in an index that optionally defines a sort order and length limit.
#[derive(Debug, PartialEq, Clone)]
pub struct IndexField {
    pub location: IndexFieldLocation,
    pub sort_order: Option<SortOrder>,
    pub length: Option<u32>,
}

impl IndexField {
    /// Tests only
    pub fn new_in_model(name: &str) -> Self {
        IndexField {
            location: IndexFieldLocation::InCurrentModel {
                field_name: name.into(),
            },
            sort_order: None,
            length: None,
        }
    }
}

/// Represents a primary key defined via `@@id` or `@id`.
#[derive(Debug, PartialEq, Clone)]
pub struct PrimaryKeyDefinition {
    pub name: Option<String>,
    pub db_name: Option<String>,
    pub fields: Vec<PrimaryKeyField>,
    pub defined_on_field: bool,
}

///A field in a Primary Key that optionally defines a sort order and length limit.
#[derive(Debug, PartialEq, Clone)]
pub struct PrimaryKeyField {
    pub name: String,
    pub sort_order: Option<SortOrder>,
    pub length: Option<u32>,
}

impl PrimaryKeyField {
    /// Tests only
    pub fn new(name: &str) -> Self {
        PrimaryKeyField {
            name: name.to_string(),
            sort_order: None,
            length: None,
        }
    }
}

impl fmt::Display for PrimaryKeyField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if self.sort_order.is_some() || self.length.is_some() {
            write!(f, "(")?;
            if let Some(length) = self.length {
                write!(f, "{}", length)?;
            }
            if let Some(sort) = self.sort_order {
                write!(f, "{}", sort)?;
            }
            write!(f, ")")?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IndexType {
    Unique,
    Normal,
    Fulltext,
}

impl IndexType {
    pub fn is_fulltext(self) -> bool {
        matches!(self, IndexType::Fulltext)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SortOrder::Asc => write!(f, "sort: Asc"),
            SortOrder::Desc => write!(f, "sort: Desc"),
        }
    }
}

/// A unique criteria is a set of fields through which a record can be uniquely identified.
#[derive(Debug)]
pub struct UniqueCriteria<'a> {
    pub fields: Vec<&'a ScalarField>,
}

impl<'a> UniqueCriteria<'a> {
    pub fn new(fields: Vec<&'a ScalarField>) -> UniqueCriteria<'a> {
        UniqueCriteria { fields }
    }
}

impl Model {
    /// Creates a new model with the given name.
    pub fn new(name: String, database_name: Option<String>) -> Model {
        Model {
            name,
            fields: vec![],
            indices: vec![],
            primary_key: None,
            documentation: None,
            database_name,
            is_generated: false,
            is_commented_out: false,
            is_ignored: false,
        }
    }

    /// Adds a field to this model.
    pub fn add_field(&mut self, field: Field) {
        self.fields.push(field)
    }

    /// Gets an iterator over all fields.
    pub fn fields(&self) -> std::slice::Iter<Field> {
        self.fields.iter()
    }

    /// Gets a mutable  iterator over all fields.
    pub fn fields_mut(&mut self) -> impl Iterator<Item = &mut Field> {
        self.fields.iter_mut()
    }

    /// Gets an iterator over all scalar fields.
    pub fn scalar_fields(&self) -> impl Iterator<Item = &ScalarField> {
        self.fields.iter().filter_map(|f| f.as_scalar_field())
    }

    /// Gets an iterator over all relation fields.
    pub fn relation_fields(&self) -> impl Iterator<Item = &RelationField> {
        self.fields.iter().filter_map(|f| f.as_relation_field())
    }

    /// Gets a mutable iterator over all scalar fields.
    pub fn scalar_fields_mut(&mut self) -> impl Iterator<Item = &mut ScalarField> {
        self.fields_mut().filter_map(|fw| match fw {
            Field::RelationField(_) => None,
            Field::CompositeField(_) => None,
            Field::ScalarField(sf) => Some(sf),
        })
    }

    /// Gets a mutable iterator over all relation fields.
    pub fn relation_fields_mut(&mut self) -> impl Iterator<Item = &mut RelationField> {
        self.fields_mut().filter_map(|fw| match fw {
            Field::RelationField(rf) => Some(rf),
            Field::CompositeField(_) => None,
            Field::ScalarField(_) => None,
        })
    }

    /// Finds a field by name.
    pub fn find_field(&self, name: &str) -> Option<&Field> {
        self.fields().find(|f| f.name() == name)
    }

    /// Finds a field by name and returns a mutable reference.
    pub fn find_field_mut(&mut self, name: &str) -> &mut Field {
        self.fields_mut().find(|f| f.name() == name).unwrap()
    }

    /// Finds a scalar field by name.
    pub fn find_scalar_field(&self, name: &str) -> Option<&ScalarField> {
        self.scalar_fields().find(|f| f.name == *name)
    }

    /// Finds a scalar field by name.
    pub fn find_relation_field(&self, name: &str) -> Option<&RelationField> {
        self.relation_fields().find(|f| f.name == *name)
    }

    /// Finds a field by database name.
    pub fn find_scalar_field_db_name(&self, db_name: &str) -> Option<&ScalarField> {
        self.scalar_fields()
            .find(|f| f.database_name.as_deref() == Some(db_name))
    }

    pub fn has_field(&self, name: &str) -> bool {
        self.find_field(name).is_some()
    }

    /// Finds a field by name and returns a mutable reference.
    pub fn find_scalar_field_mut(&mut self, name: &str) -> &mut ScalarField {
        let model_name = &self.name.clone();
        self.scalar_fields_mut().find(|rf| rf.name == *name).expect(&*format!(
            "Could not find scalar field {} on model {}.",
            name, model_name
        ))
    }

    /// Finds a relation field by name and returns a mutable reference.
    #[track_caller]
    pub fn find_relation_field_mut(&mut self, name: &str) -> &mut RelationField {
        let model_name = &self.name.clone();
        self.relation_fields_mut().find(|rf| rf.name == *name).expect(&*format!(
            "Could not find relation field {} on model {}.",
            name, model_name
        ))
    }

    pub fn field_is_indexed(&self, name: &str) -> bool {
        let field = self.find_field(name).unwrap();

        if self.field_is_primary(field.name()) || self.field_is_unique(field.name()) {
            return true;
        }

        let is_first_in_index = self
            .indices
            .iter()
            .any(|index| match &index.fields.first().unwrap().location {
                IndexFieldLocation::InCurrentModel { field_name } => field_name == name,
                IndexFieldLocation::InCompositeType { .. } => false,
            });

        let is_first_in_primary_key = matches!(&self.primary_key, Some(PrimaryKeyDefinition{ fields, ..}) if fields.first().unwrap().name == name);

        is_first_in_index || is_first_in_primary_key
    }

    /// Determines whether there is a singular primary key
    pub fn has_single_id_field(&self) -> bool {
        matches!(&self.primary_key, Some(PrimaryKeyDefinition{fields, ..}) if fields.len() ==1)
    }

    pub fn add_index(&mut self, index: IndexDefinition) {
        self.indices.push(index)
    }

    pub fn has_created_at_and_updated_at(&self) -> bool {
        /// Finds a field by name.
        fn has_field(model: &Model, name: &str) -> bool {
            match model
                .find_scalar_field(name)
                .or_else(|| model.find_scalar_field(name.to_lowercase().as_ref()))
            {
                Some(f) => f.field_type.is_datetime(),
                None => false,
            }
        }

        has_field(self, "createdAt") && has_field(self, "updatedAt")
    }

    pub fn field_is_unique(&self, name: &str) -> bool {
        self.indices.iter().any(|i| {
            let names_match = match &i.fields.first().unwrap().location {
                IndexFieldLocation::InCurrentModel { field_name } => field_name == name,
                IndexFieldLocation::InCompositeType { .. } => false,
            };

            i.is_unique() && i.fields.len() == 1 && names_match
        })
    }

    pub fn field_is_unique_and_defined_on_field(&self, name: &str) -> bool {
        self.indices.iter().any(|i| {
            let names_match = match &i.fields.first().unwrap().location {
                IndexFieldLocation::InCurrentModel { field_name } => field_name == name,
                IndexFieldLocation::InCompositeType { .. } => false,
            };

            i.is_unique() && i.fields.len() == 1 && names_match && i.defined_on_field
        })
    }

    pub fn field_is_primary(&self, field_name: &str) -> bool {
        matches!(&self.primary_key, Some(pk) if pk.fields.len() == 1 && pk.fields.first().unwrap().name == field_name)
    }

    pub fn field_is_primary_and_defined_on_field(&self, field_name: &str) -> bool {
        matches!(&self.primary_key, Some(PrimaryKeyDefinition{ fields, defined_on_field , ..}) if fields.len()  == 1 && fields.first().unwrap().name == field_name && *defined_on_field)
    }

    pub fn field_is_auto_generated_int_id(&self, name: &str) -> bool {
        let field = self.find_scalar_field(name).unwrap();
        let is_autogenerated_id = matches!(field.default_value.as_ref().map(|val| val.kind()), Some(DefaultKind::Expression(_)) if self.field_is_primary(name));
        let is_an_int = matches!(field.field_type, FieldType::Scalar(ScalarType::Int, _, _));

        is_autogenerated_id && is_an_int
    }
}

impl WithName for Model {
    fn name(&self) -> &String {
        &self.name
    }
    fn set_name(&mut self, name: &str) {
        self.name = String::from(name)
    }
}

impl WithDatabaseName for Model {
    fn database_name(&self) -> Option<&str> {
        self.database_name.as_deref()
    }

    fn set_database_name(&mut self, database_name: Option<String>) {
        self.database_name = database_name;
    }
}

impl Ignorable for Model {
    fn is_ignored(&self) -> bool {
        self.is_ignored
    }

    fn ignore(&mut self) {
        self.is_ignored = true;
    }
}
