

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParameterId{
    Gain,
    Mix,
    MixLeft,
    MixRight,
    Feedback,
    FeedbackLeft,
    FeedbackRight,
    DelayTime,
    DelayTimeLeft,
    DelayTimeRight,
    Threshold,
    Ratio,
    Attack,
    Release,
    Decay,
    Size,
    Spread,
    Drive,
    Ceiling,
}

#[derive(Debug, PartialEq)]
pub enum ParameterError {
    UnknownParameter(ParameterId),
    OutOfRange {
        id: ParameterId,
        value: f64,
        min: f64,
        max: f64,
    },
}

#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub id: ParameterId,
    pub name: &'static str,
    pub min: f64,
    pub max: f64,
    pub default: f64,
}

pub fn validate_parameter(id: ParameterId,value: f64, parameter: &ParameterInfo) -> Result<(), ParameterError>{
    if !(parameter.min..=parameter.max).contains(&value) {
                    return Err(
                        ParameterError::OutOfRange {
                            id,
                            value,
                            min: parameter.min,
                            max: parameter.max,
                        },
                    );
                }
    Ok(())
}