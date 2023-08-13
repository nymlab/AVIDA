use cosmwasm_std::Event;

pub fn new_event() -> Event {
    Event::new("vectis.AnonCredsVerifier.v1")
}
