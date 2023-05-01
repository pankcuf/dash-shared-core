pub enum SequenceGapLimit {
    External,
    Internal,
    Initial,
}

impl SequenceGapLimit {
    pub fn default(&self) -> u32 {
        match self {
            SequenceGapLimit::External => 10,
            SequenceGapLimit::Internal => 5,
            SequenceGapLimit::Initial => 100,
        }
    }

    pub fn unused(&self) -> u32 {
        match self {
            SequenceGapLimit::External => 10,
            SequenceGapLimit::Internal => 5,
            SequenceGapLimit::Initial => 15,
        }
    }

    pub fn dashpay(&self) -> u32 {
        match self {
            SequenceGapLimit::External => 6,
            SequenceGapLimit::Internal => 3,
            SequenceGapLimit::Initial => 10,
        }
    }
}
