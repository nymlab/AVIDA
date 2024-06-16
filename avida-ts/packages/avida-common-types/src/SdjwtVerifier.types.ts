/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.7.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export type Binary = string;
export type TrustRegistry = "cheqd";
export interface InstantiateMsg {
  init_registrations: InitRegistration[];
  max_presentation_len: number;
  [k: string]: unknown;
}
export interface InitRegistration {
  app_addr: string;
  app_admin: string;
  routes: InputRoutesRequirements[];
}
export interface InputRoutesRequirements {
  requirements: RouteVerificationRequirements;
  route_id: number;
}
export interface RouteVerificationRequirements {
  presentation_request: Binary;
  verification_source: VerificationSource;
}
export interface VerificationSource {
  data_or_location: Binary;
  source?: TrustRegistry | null;
}
export type ExecuteMsg = AvidaVerifierTraitExecMsg | ExecMsg;
export type AvidaVerifierTraitExecMsg = {
  register: {
    app_addr: string;
    route_criteria: InputRoutesRequirements[];
    [k: string]: unknown;
  };
} | {
  verify: {
    app_addr?: string | null;
    presentation: Binary;
    route_id: number;
    [k: string]: unknown;
  };
} | {
  update: {
    app_addr: string;
    route_criteria?: RouteVerificationRequirements | null;
    route_id: number;
    [k: string]: unknown;
  };
} | {
  deregister: {
    app_addr: string;
    [k: string]: unknown;
  };
};
export type ExecMsg = string;
export type QueryMsg = AvidaVerifierTraitQueryMsg | QueryMsg1;
export type AvidaVerifierTraitQueryMsg = {
  get_routes: {
    app_addr: string;
    [k: string]: unknown;
  };
} | {
  get_route_requirements: {
    app_addr: string;
    route_id: number;
    [k: string]: unknown;
  };
};
export type QueryMsg1 = {
  get_route_verification_key: {
    app_addr: string;
    route_id: number;
    [k: string]: unknown;
  };
};
export type NullableString = string | null;
export type ArrayOfUint64 = number[];