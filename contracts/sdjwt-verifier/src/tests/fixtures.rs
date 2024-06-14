use sylvia::multitest::{App, Proxy};
use avida_test_utils::sdjwt::fixtures::RouteVerificationRequirementsType;
use avida_test_utils::sdjwt::fixtures::{
    FIRST_CALLER_APP_ADDR, FIRST_ROUTE_ID, OWNER_ADDR, VERIFIER_CONTRACT_LABEL, MAX_PRESENTATION_LEN, get_route_verification_requirement
};

use avida_common::types::{
    InputRoutesRequirements, RouteVerificationRequirements,
};

use cw_multi_test::App as MtApp;

use crate::contract::sv::mt::CodeId;
use crate::contract::SdjwtVerifier;
use crate::types::InitRegistration;



/// Is used to instantiate verifier contract with some predefined parameters
pub fn instantiate_verifier_contract<'a>(
    app: &'a App<MtApp>,
    route_verification_requirements_type: RouteVerificationRequirementsType,
) -> (
    Proxy<'a, MtApp, SdjwtVerifier<'a>>,
    RouteVerificationRequirements,
) {
    let fx_route_verification_req =
        get_route_verification_requirement(route_verification_requirements_type);
    let code_id = CodeId::store_code(app);

    // String, // Admin
    // String, // App Addr
    // Vec<(RouteId, RouteVerificationRequirements)>,
    let init_registrations = vec![InitRegistration {
        app_admin: FIRST_CALLER_APP_ADDR.to_string(),
        app_addr: FIRST_CALLER_APP_ADDR.to_string(),
        routes: vec![InputRoutesRequirements {
            route_id: FIRST_ROUTE_ID,
            requirements: fx_route_verification_req.clone(),
        }],
    }];

    (
        code_id
            .instantiate(MAX_PRESENTATION_LEN, init_registrations)
            .with_label(VERIFIER_CONTRACT_LABEL)
            .call(OWNER_ADDR)
            .unwrap(),
        fx_route_verification_req,
    )
}
