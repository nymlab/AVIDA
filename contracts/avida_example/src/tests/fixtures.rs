use avida_common::types::RouteVerificationRequirements;
use avida_sdjwt_verifier::types::{Criterion, MathsOperator, PresentationReq, CW_EXPIRATION};
use avida_test_utils::sdjwt::fixtures::{
    claims, get_default_block_info, make_presentation, make_route_verification_requirements,
    PresentationVerificationType, RouteVerificationRequirementsType,
};

pub fn create_presentation() -> String {
    let claims = claims("Alice", 30, true, 2021, None);
    make_presentation(claims, PresentationVerificationType::Success)
}

pub fn create_presentation_with_exp(expired: bool) -> String {
    let exp = if expired {
        cw_utils::Expiration::AtTime(get_default_block_info().time.minus_hours(1))
    } else {
        cw_utils::Expiration::AtTime(get_default_block_info().time.plus_hours(1))
    };
    let claims = claims("Alice", 30, true, 2021, Some(exp));
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

pub fn setup_requirement_with_expiration() -> RouteVerificationRequirements {
    // Add only 1 criterion - age greater than 18
    let presentation_req: PresentationReq = vec![
        (
            "age".to_string(),
            Criterion::Number(18, MathsOperator::GreaterThan),
        ),
        (CW_EXPIRATION.to_string(), Criterion::Expires(true)),
    ];

    make_route_verification_requirements(
        presentation_req,
        RouteVerificationRequirementsType::Supported,
    )
}
