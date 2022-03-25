use crate::types::FieldWithArgs;
use crate::walkers::{CompositeTypeFieldWalker, IndexFieldWalker};
use crate::{
    ast,
    {walkers::ScalarFieldWalker, ParserDatabase},
};

/// Describes any unique criteria in a model. Can either be a primary
/// key, or a unique index.
#[derive(Copy, Clone)]
pub struct UniqueCriteriaWalker<'db> {
    pub(crate) model_id: ast::ModelId,
    pub(crate) fields: &'db [FieldWithArgs],
    pub(crate) db: &'db ParserDatabase,
}

impl<'db> UniqueCriteriaWalker<'db> {
    pub fn fields(self) -> impl ExactSizeIterator<Item = IndexFieldWalker<'db>> + 'db {
        self.fields.iter().map(move |field| match field.field_location {
            crate::types::IndexFieldLocation::InModel(field_id) => {
                let walker = ScalarFieldWalker {
                    model_id: self.model_id,
                    field_id,
                    db: self.db,
                    scalar_field: &self.db.types.scalar_fields[&(self.model_id, field_id)],
                };

                IndexFieldWalker::new(walker)
            }
            crate::types::IndexFieldLocation::InCompositeType(ctid, field_id) => {
                let walker = CompositeTypeFieldWalker {
                    ctid,
                    field_id,
                    field: &self.db.types.composite_type_fields[&(ctid, field_id)],
                    db: self.db,
                };

                IndexFieldWalker::new(walker)
            }
        })
    }

    pub fn is_strict_criteria(self) -> bool {
        !self.has_optional_fields() && !self.has_unsupported_fields()
    }

    pub(crate) fn has_optional_fields(self) -> bool {
        self.fields().any(|field| field.is_optional())
    }

    pub fn has_unsupported_fields(self) -> bool {
        self.fields().any(|field| field.is_unsupported())
    }
}
