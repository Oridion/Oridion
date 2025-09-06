use anchor_lang::prelude::*;

#[error_code]
pub enum OridionError {
    #[msg("InvalidTreasure")]
    InvalidTreasure,
    #[msg("Invalid passcode")]
    InvalidPasscode,
    #[msg("Too soon to emergency land")]
    TooSoonToEmLand,
    #[msg("Unauthorized mode")]
    UnauthorizedMode,
    #[msg("Universe is locked")]
    UniverseLocked,
    #[msg("Insufficient fee")]
    InsufficientFee,
    #[msg("Too many pods!")]
    TooManyPods,
    #[msg("Planet name too long")]
    PlanetNameTooLong,
    #[msg("Planet cannot be deleted. Still has funds")]
    PlanetDeleteHasFundsError,
    #[msg("To and from cannot be the same")]
    HopErrorToAndFromAreSame,
    #[msg("Insufficient funds for hop")]
    InsufficientFunds,
    #[msg("Stars IDs must be unique")]
    HopErrorStarsMustBeUnique,
    #[msg("Pod lamports have already landed!")]
    AlreadyLanded,
    #[msg("Planet does not have enough lamports to cover transaction!")]
    PlanetNotEnoughFundsError,
    #[msg("Planet is still locked!")]
    PlanetStillLocked,
    #[msg("Planet must be locked in order to hop")]
    PlanetNotLocked,
    #[msg("Pod not authorized to hop from this planet")]
    NotAuthorizedToHop,
    #[msg("Planet lock has expired!")]
    LockExpired,
    #[msg("Star split calculations do not add up!")]
    StarHopCalculationError,
    #[msg("Pod is in transit. Cannot start if already in transit!")]
    InTransit,
    #[msg("Pod is not in transit. Cannot end if not started!")]
    NotInTransit,
    #[msg("Pod amount must be greater than 0!")]
    InvalidDepositAmount,
    #[msg("Landing timestamp must be in the future!")]
    InvalidLandingTimestamp,
    #[msg("Pod is active!")]
    ExistingPodActive,
    #[msg("Invalid destination address")]
    InvalidDestination,
    #[msg("Invalid delay amount passed")]
    InvalidDelay,
    #[msg("Invalid mode passed")]
    InvalidMode,
    #[msg("Invalid Scatter meta")]
    InvalidScatterMeta,
    #[msg("Scatter split math not adding up!")]
    ScatterSplitMathError,
    #[msg("Scatter total to pod amount mismatch!")]
    ScatterTotalToPodMismatch,
    #[msg("Unusual math error")]
    UnusualMathError,
    #[msg("Insufficient treasury balance for withdrawal")]
    InsufficientTreasuryBalance,
    #[msg("Invalid star meta account passed")]
    InvalidStarMetaPda,
    #[msg("PDA already initialized")]   
    PdaAlreadyInitialized,
    #[msg("Land ticket not found")]
    TicketNotFound,
    #[msg("Land book is full")]
    LandBookFull,
}