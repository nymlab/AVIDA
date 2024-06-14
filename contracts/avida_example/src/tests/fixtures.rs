use avida_common::types::RouteVerificationRequirements;
use avida_sdjwt_verifier::types::{Criterion, MathsOperator, PresentationReq};
use avida_test_utils::sdjwt::fixtures::{
    claims, make_presentation, make_route_verification_requirements, PresentationVerificationType,
    RouteVerificationRequirementsType,
};

pub fn create_presentation() -> String {
    let claims = claims("Alice", 30, true, 2021);
    make_presentation(claims, PresentationVerificationType::Success)
}

pub fn setup_requirement() -> RouteVerificationRequirements {
    // Add only 1 criterion - age greater than 18
    let presentation_req: PresentationReq = vec![(
        "age".to_string(),
        Criterion::Number(18, MathsOperator::GreaterThan),
    )];
    make_route_verification_requirements(
        presentation_req,
        RouteVerificationRequirementsType::Supported,
    )
}
