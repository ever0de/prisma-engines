use expect_test::expect;
use indoc::indoc;

use crate::with_header;

#[test]
fn simple_composite_index() {
    let schema = indoc! {r#"
        type A {
          field String
        }

        model B {
          id Int @id @map("_id")
          a  A

          @@index([a.field])
        }
    "#};

    let dml = with_header(schema, crate::Provider::Mongo, &["mongoDb"]);
    // TODO: proper assertions
    assert!(datamodel::parse_schema(&dml).is_ok());
}

#[test]
fn index_to_a_missing_field_in_a_composite_type() {
    let schema = indoc! {r#"
        type A {
          field String
        }

        model B {
          id Int @id @map("_id")
          a  A

          @@index([a.cat])
        }
    "#};

    let dml = with_header(schema, crate::Provider::Mongo, &["mongoDb"]);
    let error = datamodel::parse_schema(&dml).map(drop).unwrap_err();

    let expected = expect![[r#"
        [1;91merror[0m: [1mError validating model "B": The index definition refers to the unknown fields: cat in type A.[0m
          [1;94m-->[0m  [4mschema.prisma:19[0m
        [1;94m   | [0m
        [1;94m18 | [0m
        [1;94m19 | [0m  @@[1;91mindex([a.cat])[0m
        [1;94m   | [0m
    "#]];

    expected.assert_eq(&error);
}

#[test]
fn index_to_a_missing_composite_field() {
    let schema = indoc! {r#"
        type A {
          field String
        }

        model B {
          id Int @id @map("_id")
          a  A

          @@index([b.field])
          @@unique([b.field])
          @@fulltext([b.field])
        }
    "#};

    let dml = with_header(schema, crate::Provider::Mongo, &["mongoDb"]);
    let error = datamodel::parse_schema(&dml).map(drop).unwrap_err();

    let expected = expect![[r#"
        [1;91merror[0m: [1mError validating model "B": The index definition refers to the unknown fields: b.[0m
          [1;94m-->[0m  [4mschema.prisma:19[0m
        [1;94m   | [0m
        [1;94m18 | [0m
        [1;94m19 | [0m  @@[1;91mindex([b.field])[0m
        [1;94m   | [0m
        [1;91merror[0m: [1mError validating model "B": The unique index definition refers to the unknown fields: b.[0m
          [1;94m-->[0m  [4mschema.prisma:20[0m
        [1;94m   | [0m
        [1;94m19 | [0m  @@index([b.field])
        [1;94m20 | [0m  @@[1;91munique([b.field])[0m
        [1;94m   | [0m
        [1;91merror[0m: [1mError validating model "B": The index definition refers to the unknown fields: b.[0m
          [1;94m-->[0m  [4mschema.prisma:21[0m
        [1;94m   | [0m
        [1;94m20 | [0m  @@unique([b.field])
        [1;94m21 | [0m  @@[1;91mfulltext([b.field])[0m
        [1;94m   | [0m
    "#]];

    expected.assert_eq(&error);
}
