/// Semantic version of this library (not the CYPHAL specification).
/// API will be backward compatible within the same major version.
pub const CYPHAL_VERSION_MAJOR: u8 = 1;
pub const CYPHAL_VERSION_MINOR: u8 = 1;

/// The version number of the CYPHAL specification implemented by this library.
pub const CYPHAL_SPECIFICATION_VERSION_MAJOR: u8 = 1;
pub const CYPHAL_SPECIFICATION_VERSION_MINOR: u8 = 0;

/// These error codes may be returned from the library API calls whose return type is a signed integer in the negated
/// form (e.g., error code 2 returned as -2). A non-negative return value represents success.
/// API calls whose return type is not a signed integer cannot fail by contract.
/// No other error states may occur in the library.
/// By contract, a well-characterized application with a properly sized memory pool will never encounter errors.
/// The error code 1 is not used because -1 is often used as a generic error code in 3rd-party code.
pub const CYPHAL_ERROR_INVALID_ARGUMENT: u8 = 2;
pub const CYPHAL_ERROR_OUT_OF_MEMORY: u8 = 3;

/// MTU values for the supported protocols.
/// Per the recommendations given in the CYPHAL/CAN Specification, other MTU values should not be used.
pub const CYPHAL_MTU_CAN_CLASSIC: u8 = 8;
pub const CYPHAL_MTU_CAN_FD: u8 = 64;

/// Parameter ranges are inclusive; the lower bound is zero for all. See CYPHAL/CAN Specification for background.
pub const CYPHAL_SUBJECT_ID_MAX: u16 = 8191;
pub const CYPHAL_SERVICE_ID_MAX: u16 = 511;
pub const CYPHAL_NODE_ID_MAX: u8 = 127;
pub const CYPHAL_PRIORITY_MAX: u8 = 7;
pub const CYPHAL_TRANSFER_ID_BIT_LENGTH: u8 = 5;
pub const CYPHAL_TRANSFER_ID_MAX: u8 =  (1 << CYPHAL_TRANSFER_ID_BIT_LENGTH) - 1;

/// This value represents an undefined node-ID: broadcast destination or anonymous source.
/// Library functions treat all values above CYPHAL_NODE_ID_MAX as anonymous.
pub const CYPHAL_NODE_ID_UNSET: u8 = 255;

/// This is the recommended transfer-ID timeout value given in the CYPHAL Specification. The application may choose
/// different values per subscription (i.e., per data specifier) depending on its timing requirements.
pub const CYPHAL_DEFAULT_TRANSFER_ID_TIMEOUT_USEC: usize = 2000000;

pub const CYPHAL_NUM_TRANSFER_KINDS: usize = 3;

pub const CAN_DLC_TO_DLEN: [u8;16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 12, 16, 20, 24, 32, 48, 64];
pub const CAN_DLEN_TO_DLC: [u8; 65] = [
    0,  1,  2,  3,  4,  5,  6,  7,  8,                               // 0-8
    9,  9,  9,  9,                                                   // 9-12
    10, 10, 10, 10,                                                  // 13-16
    11, 11, 11, 11,                                                  // 17-20
    12, 12, 12, 12,                                                  // 21-24
    13, 13, 13, 13, 13, 13, 13, 13,                                  // 25-32
    14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14, 14,  // 33-48
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,  // 49-64
];

pub const BITS_PER_BYTE: u8 = 8;
pub const BYTE_MAX: u16 = 0xff;

pub const CAN_EXT_ID_MASK: u32 = (1 << 29) - 1;

pub const MFT_NON_LAST_FRAME_PAYLOAD_MIN: u8 = 7;

pub const PADDING_BYTE_VALUE: u8 = 0;

pub const OFFSET_PRIORITY:u8 = 26;
pub const OFFSET_SUBJECT_ID: u8 = 8;
pub const OFFSET_SERVICE_ID: u8 = 14;
pub const OFFSET_DST_NODE_ID: u8 = 7;

pub const FLAG_SERVICE_NOT_MESSAGE: u32 =1 << 25;
pub const FLAG_ANONYMOUS_MESSAGE: u32 = 1 << 24;
pub const FLAG_REQUEST_NOT_RESPONSE: u32 = 1 << 24;
pub const FLAG_RESERVED_23: u32 = 1 << 23;
pub const FLAG_RESERVED_07: u32 = 1 << 7;

pub const TAIL_START_OF_TRANSFER: u8 = 128;
pub const TAIL_END_OF_TRANSFER: u8 = 64;
pub const TAIL_TOGGLE: u8 = 32;
pub const TAIL_SIZE_BYTES: u8 = 1;

pub const INITIAL_TOGGLE_STATE: bool = true;

pub const CRC_INITIAL: u16 = 0xFFFF;
pub const CRC_RESIDUE: u16 = 0x0000;
pub const CRC_SIZE_BYTES: u8 = 2;