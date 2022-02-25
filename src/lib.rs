use std::str::FromStr;

pub enum EofBehavior {
    Zero,
    Neg1,
    NoChange,
}

impl FromStr for EofBehavior {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "zero" => Ok(Self::Zero),
            "neg1" => Ok(Self::Neg1),
            "nc" => Ok(Self::NoChange),
            _ => Err(()),
        }
    }
}
