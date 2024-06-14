use cosmwasm_schema::cw_serde;
/// The json key for the disclosed claims
pub type CriterionKey = String;

#[cw_serde]
pub enum Criterion {
    String(String),
    Number(u64, MathsOperator),
    Boolean(bool),
}

#[cw_serde]
pub enum MathsOperator {
    GreaterThan,
    LessThan,
    EqualTo,
}

#[cw_serde]
pub struct PresentationReq(pub Vec<(CriterionKey, Criterion)>);
