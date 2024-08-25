/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.7.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export interface InstantiateMsg {
  verifier: string;
  [k: string]: unknown;
}
export type ExecuteMsg = ExecMsg;
export type ExecMsg = {
  register_requirement: {
    msg: RegisterRequirement;
    [k: string]: unknown;
  };
} | {
  give_me_some_drink: {
    msg: GiveMeSomeDrink;
    [k: string]: unknown;
  };
} | {
  give_me_some_food: {
    msg: GiveMeSomeFood;
    [k: string]: unknown;
  };
};
export type RegisterRequirement = {
  drink: {
    requirements: RouteVerificationRequirements;
  };
} | {
  food: {
    requirements: RouteVerificationRequirements;
  };
};
export type Binary = string;
export type TrustRegistry = "cheqd";
export interface RouteVerificationRequirements {
  issuer_source_or_data: IssuerSourceOrData;
  presentation_required: Binary;
}
export interface IssuerSourceOrData {
  data_or_location: Binary;
  source?: TrustRegistry | null;
}
export interface GiveMeSomeDrink {
  kind: string;
  proof: Binary;
}
export interface GiveMeSomeFood {
  kind: string;
  proof: Binary;
}
export type QueryMsg = QueryMsg1;
export type QueryMsg1 = {
  get_verifier_address: {
    [k: string]: unknown;
  };
};
export interface GetVerifierResponse {
  verifier: string;
}