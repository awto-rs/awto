pub mod query;

#[derive(MergedObject)]
pub struct Query(pub query::ProductQuery);

// #[derive(MergedObject, Default)]
// pub struct Mutation(query::ProductQuery);
