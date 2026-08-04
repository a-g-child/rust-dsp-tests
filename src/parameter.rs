

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParameterId{
    Gain,
    Mix,
    Feedback,
    DelayTime,
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
    InvalidTarget{
        id: ParameterId,
        target: ParameterTarget,
    },
    UnknownParameter(ParameterId),
    ChannelOutOfRange{
        channel: usize,
        channel_count: usize,
    },
    OutOfRange {
        id: ParameterId,
        value: f64,
        min: f64,
        max: f64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterScope {
    Global,
    PerChannel,
}

#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub id: ParameterId,
    pub name: &'static str,
    pub min: f64,
    pub max: f64,
    pub default: f64,
    pub scope: ParameterScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParameterTarget {
    Global,
    Channel(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParameterAddress {
    pub id: ParameterId,
    pub target: ParameterTarget,
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

pub fn validate_has_channels(address: ParameterAddress) -> Result<usize, ParameterError>{
    let ParameterTarget::Channel(channel) = address.target else {
        return Err(ParameterError::InvalidTarget {
            id: address.id,
            target: address.target,
        });
    };
    Ok(channel)
}

pub fn validate_channel_exists(channel_count: usize, channel: usize) -> Result<(), ParameterError> {
        if channel >= channel_count {
            return Err(ParameterError::ChannelOutOfRange {
                channel,
                channel_count: channel_count,
            });
        }
        Ok(())
}