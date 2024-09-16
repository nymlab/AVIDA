// example const req = [["age", { number: [18, "greater_than"] }]];
// see `avida/contracts/sdjwt-verifier/src/types.rs`: `pub type PresentationReq = Vec<(CriterionKey, Criterion)>`
export type PresentationReq = Array<[CriterionKey, Criterion]>;
export type CriterionKey = string;
export type Criterion =
  | { string: string }
  | { number: [number, MathsOperator] }
  | { boolean: boolean }
  | { expires: string }
  | { not_contained_in: Array<number> }
  | { dynamic: Criterion };
export type MathsOperator = "greater_than" | "less_than" | "equal_to";
