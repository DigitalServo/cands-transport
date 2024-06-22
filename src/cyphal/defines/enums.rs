// Transfer priority level mnemonics per the recommendations given in the Cyphal Specification.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum CyphalPriority {
    Exceptional,
    Immediate,
    Fast,
    High,
    Nominal,
    Low,
    Slow,
    Optional,
    Undefined,
}

impl From<u8> for CyphalPriority {
    fn from(x: u8) -> Self {
        match x {
            0 => CyphalPriority::Exceptional,
            1 => CyphalPriority::Immediate,
            2 => CyphalPriority::Fast,
            3 => CyphalPriority::High,
            4 => CyphalPriority::Nominal,
            5 => CyphalPriority::Low,
            6 => CyphalPriority::Slow,
            7 => CyphalPriority::Optional,
            _ => CyphalPriority::Undefined,
        }
    }
}

// Transfer kinds as defined by the Cyphal Specification.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum CyphalTransferKind {
    Message,
    Response,
    Request,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum CyphalRxPacketType {
    SignleFrame,
    MultiFrameStart,
    MultiFrameEnd,
    MultiFrameInProcess
}