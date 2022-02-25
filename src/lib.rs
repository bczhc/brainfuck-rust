pub enum EofBehavior {
    Zero,
    Neg1,
    NoChange,
}

impl EofBehavior {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "zero" => Some(Self::Zero),
            "neg1" => Some(Self::Neg1),
            "nc" => Some(Self::NoChange),
            _ => None,
        }
    }
}
