#![allow(non_upper_case_globals)]
#![allow(unused)]

pub const HostReg__0: u8 = 0x00;
pub const HostReg__1: u8 = 0x01;
pub const HostReg__2: u8 = 0x02;
pub const HostReg__3: u8 = 0x03;
pub const HostReg__4: u8 = 0x04;
pub const HostReg__5: u8 = 0x05;
pub const HostReg__6: u8 = 0x06;
pub const HostReg__7: u8 = 0x07;
pub const HostReg__8: u8 = 0x08;
pub const HostReg__9: u8 = 0x09;
pub const HostReg__10: u8 = 0x0A;
pub const HostReg__11: u8 = 0x0B;
pub const HostReg__12: u8 = 0x0C;
pub const HostReg__13: u8 = 0x0D;
pub const HostReg__14: u8 = 0x0E;
pub const HostReg__15: u8 = 0x0F;
pub const HostReg__16: u8 = 0x10;
pub const HostReg__17: u8 = 0x11;
pub const HostReg__18: u8 = 0x12;
pub const HostReg__19: u8 = 0x13;
pub const HostReg__20: u8 = 0x14;
pub const HostReg__21: u8 = 0x15;
pub const HostReg__22: u8 = 0x16;
pub const HostReg__23: u8 = 0x17;
pub const HostReg__24: u8 = 0x18;
pub const HostReg__25: u8 = 0x19;
pub const HostReg__26: u8 = 0x1A;
pub const HostReg__27: u8 = 0x1B;
pub const HostReg__28: u8 = 0x1C;
pub const HostReg__29: u8 = 0x1D;
pub const HostReg__30: u8 = 0x1E;
pub const HostReg__31: u8 = 0x1F;

pub trait Register<AddrSize> {
    const REG: AddrSize;

    fn from_byte(b: u8) -> Self;
    fn to_byte(self) -> u8;
    fn def() -> Self;
}

macro_rules! register {
    ($addrty:ty, $name:ident, $reg:expr, $def:expr) => {
        #[doc = concat!("Default value: `", stringify!($def), "`.")]
        pub struct $name(pub u8);

        impl Register<$addrty> for $name {
            const REG: $addrty = $reg;

            fn from_byte(b: u8) -> Self {
                Self(b)
            }

            fn to_byte(self) -> u8 {
                self.0
            }

            #[doc = concat!("Default value: `", stringify!($def), "`.")]
            fn def() -> Self {
                Self($def)
            }
        }
    };
    ($addrty:ty, $name:ident, $reg:expr, $def:expr, $fields:tt) => {
        #[::bitfield_struct::bitfield(u8)]
        #[doc = concat!("Default value: `", stringify!($def), "`.")]
        pub struct $name $fields

        impl Register<$addrty> for $name {
            const REG: $addrty = $reg;

            fn from_byte(b: u8) -> Self {
                Self::from(b)
            }

            fn to_byte(self) -> u8 {
                u8::from(self)
            }

            #[doc = concat!("Default value: `", stringify!($def), "`.")]
            fn def() -> Self {
                Self($def)
            }
        }
    };
}

// ---------------- Register Assignments -------------------------------------

/*--------------------------------------------------------------------------*\
Chip ID / Version
\*--------------------------------------------------------------------------*/
// Chip ID Register
register!(u8, ChipId, HostReg__0, 0);

// Chip Version Register
register!(u8, Version, HostReg__1, 0);

/*--------------------------------------------------------------------------*\
Status Register
\*--------------------------------------------------------------------------*/
// Status 1 Register -- MUST BE HOSTREG__2
register!(u8, Status, HostReg__2, 0, {
    #[bits(2)]
    _p: u8,
    pub data_ready: bool,
    pub command_complete: bool,
    #[bits(4)]
    _p: u8,
});

/*--------------------------------------------------------------------------*\
System Config Register
\*--------------------------------------------------------------------------*/
register!(u8, SystemConfig, HostReg__3, 0, {
    pub reset: bool,
    pub standby: bool,
    pub auto_sleep: bool,
    pub track_disable: bool,
    pub anymeas_enable: bool,
    pub gpio_ctrl_enable: bool,
    pub wakeup_toggle: bool,
    pub force_wakeup: bool,
});

/*--------------------------------------------------------------------------*\
Feed Config Registers
\*--------------------------------------------------------------------------*/
// Feed Config Register1
register!(u8, FeedConfig1, HostReg__4, 0, {
    pub feed_enable: bool,
    pub data_type_relo0_abs1: bool,
    pub filter_disable: bool,
    pub x_axis_disable: bool,
    pub y_axis_disable: bool,
    pub axis_for_z_y0_x1: bool,
    pub x_data_invert: bool,
    pub y_data_invert: bool,
});

// Feed Config Register2
register!(u8, FeedConfig2, HostReg__5, 0, {
    pub intellimouse_mode: bool,
    pub all_tap_disable: bool,
    pub secondary_tap_disable: bool,
    pub scroll_disable: bool,
    pub glide_extend_disable: bool,
    pub palm_before_z_enable: bool,
    pub butns_46_scroll_5_middle: bool,
    pub swap_xy_relative: bool,
});

// Feed Config Register3
register!(u8, FeedConfig3, HostReg__6, 0, {
    pub btns_456_to_123_in_rel: bool,
    pub disable_cross_rate_smoothing: bool,
    pub disable_palm_nerd_meas: bool,
    pub disable_noise_avoidance: bool,
    pub disable_wrap_lockout: bool,
    pub disable_dynamic_emi_adjust: bool,
    pub disable_hw_emi_detect: bool,
    pub disable_sw_emi_detect: bool,
});

/*--------------------------------------------------------------------------*\
Calibration Config
\*--------------------------------------------------------------------------*/
register!(u8, CalConfig, HostReg__7,
          CalConfig::new()
            .with_background_comp_enable(true)
            .with_nerd_comp_enable(true)
            .with_track_error_comp_enable(true)
            .with_tap_comp_enable(true)
            .with_palm_error_comp_enable(true)
            .into(), {
    pub calibrate: bool,
    pub background_comp_enable: bool,
    pub nerd_comp_enable: bool,
    pub track_error_comp_enable: bool,
    pub tap_comp_enable: bool,
    pub palm_error_comp_enable: bool,
    pub calibration_matrix_disable: bool,
    pub force_precalibration_noise_check: bool,
});

/*--------------------------------------------------------------------------*\
PS2 Aux Control Register
\*--------------------------------------------------------------------------*/
pub const HOSTREG__PS2AUX_CTRL: u8 = HostReg__8;
pub const HOSTREG__PS2AUX_CTRL__CMD_PASSTHRU_ENABLE: u8 = 0x01;
pub const HOSTREG__PS2AUX_CTRL__SP_EXTENDED_MODE: u8 = 0x02;
pub const HOSTREG__PS2AUX_CTRL__GS_DISABLE: u8 = 0x04;
pub const HOSTREG__PS2AUX_CTRL__SP_DISABLE: u8 = 0x08;
pub const HOSTREG__PS2AUX_CTRL__GS_COORDINATE_DISABLE: u8 = 0x10;
pub const HOSTREG__PS2AUX_CTRL__SP_COORDINATE_DISABLE: u8 = 0x20;
pub const HOSTREG__PS2AUX_CTRL__DISABLE_AA00_DETECT: u8 = 0x40;
pub const HOSTREG__PS2AUX_CTRL__AUX_PRESENT: u8 = 0x80;
pub const HOSTREG__PR2AUX_CTRL_DEFVAL: u8 = 0x00;

/*--------------------------------------------------------------------------*\
Sample Rate Value
\*--------------------------------------------------------------------------*/
register!(u8, SampleRate, HostReg__9, SampleRate::SPS_100);
impl SampleRate {
    pub const SPS_10: u8 = 0x0A;
    pub const SPS_20: u8 = 0x14;
    pub const SPS_40: u8 = 0x28;
    pub const SPS_60: u8 = 0x3C;
    pub const SPS_80: u8 = 0x50;
    pub const SPS_100: u8 = 0x64;
    pub const SPS_200: u8 = 0xC8;
}

/*--------------------------------------------------------------------------*\
Z Idle Value
\*--------------------------------------------------------------------------*/
register!(u8, ZIdle, HostReg__10, 30);

/*--------------------------------------------------------------------------*\
Z Scaler Value
\*--------------------------------------------------------------------------*/
register!(u8, ZScaler, HostReg__11, 8);

/*--------------------------------------------------------------------------*\
Sleep Interval Value
\*--------------------------------------------------------------------------*/
register!(u8, SleepInterval, HostReg__12, 73);

/*--------------------------------------------------------------------------*\
Sleep Delay Value
\*--------------------------------------------------------------------------*/
register!(u8, SleepDelay, HostReg__13, 39);

/*--------------------------------------------------------------------------*\
Dynamic EMI Bad Channel Count Thresholds
\*--------------------------------------------------------------------------*/
register!(u8, DynamicEmiAdjustThreshold, HostReg__14, 66);

/*--------------------------------------------------------------------------*\
Packet Registers
\*--------------------------------------------------------------------------*/
register!(u8, Packet0, HostReg__18, 0);
register!(u8, Packet1, HostReg__19, 0);
register!(u8, Packet2, HostReg__20, 0);
register!(u8, Packet3, HostReg__21, 0);
register!(u8, Packet4, HostReg__22, 0);
register!(u8, Packet5, HostReg__23, 0);

/*--------------------------------------------------------------------------*\
Port A GPIO Control
\*--------------------------------------------------------------------------*/
register!(u8, PortAGPIOCtrl, HostReg__24, 0xFF);

/*--------------------------------------------------------------------------*\
Port A GPIO Data
\*--------------------------------------------------------------------------*/
register!(u8, PortAGPIOData, HostReg__25, 0);

/*--------------------------------------------------------------------------*\
Port B GPIO Control And Data
\*--------------------------------------------------------------------------*/
register!(u8, PortBGpioCtrl, HostReg__26,
          PortBGpioCtrl::new()
            .with_ctrl_pb0(true)
            .with_ctrl_pb1(true)
            .with_ctrl_pb2(true)
            .into(), {
    pub data_pb0: bool,
    pub data_pb1: bool,
    pub data_pb2: bool,
    pub ctrl_pb0: bool,
    pub ctrl_pb1: bool,
    pub ctrl_pb2: bool,
    pub rsvd_0: bool,
    pub read1_write0: bool,
});

/*--------------------------------------------------------------------------*\
Extended Register Access
\*--------------------------------------------------------------------------*/
register!(u8, AXSValue, HostReg__27, 0);

register!(u8, AXSAddrHigh, HostReg__28, 0);
register!(u8, AXSAddrLow, HostReg__29, 0);

register!(u8, AXSCtrl, HostReg__30, 0, {
    pub read: bool,
    pub write: bool,
    pub inc_addr_read: bool,
    pub inc_addr_write: bool,
    pub rsvd_3: bool,
    pub rsvd_2: bool,
    pub rsvd_1: bool,
    pub rsvd_0: bool,
});

/*--------------------------------------------------------------------------*\
Product ID
\*--------------------------------------------------------------------------*/
register!(u8, ProductId, HostReg__31, 0);

//Some useful values
pub const I2C_ADDRESS_DEFAULT: u8 = 0x2A;
pub const FIRMWARE_ID: u8 = 0x07;
pub const FIRMWARE_VERSION: u8 = 0x9D;

//Anymeas config options
//First setting is HostReg 5.  This sets toggle frequency (EF) and gain.
//Gain is upper two bits (0xC0), frequency is lower 6 bits (0x3F)

register!(u8, AnyMeasAccumBitsElecFreq, HostReg__5, 0, {
    #[bits(5)]
    pub elec_freq: u8,

    #[bits(1)]
    _p: u8,

    #[bits(2)]
    pub accum_bits_select: u8
});

impl AnyMeasAccumBitsElecFreq {
    /// 500,000Hz
    const EF_0: u8 = 0x02;

    /// 444,444Hz
    const EF_1: u8 = 0x03;

    /// 400,000Hz
    const EF_2: u8 = 0x04;

    /// 363,636Hz
    const EF_3: u8 = 0x05;

    /// 333,333Hz
    const EF_4: u8 = 0x06;

    /// 307,692Hz
    const EF_5: u8 = 0x07;

    /// 267,000Hz
    const EF_6: u8 = 0x08;

    /// 235,000Hz
    const EF_7: u8 = 0x09;

    const GAIN_2X: u8 = 0;
    const GAIN_1_6X: u8 = 1;
    const GAIN_1_3X: u8 = 2;
    const GAIN_MIN: u8 = 3;
}

register!(u8, AnyMeasBitLength, HostReg__6, 0, {
    #[bits(2)]
    pub bit_length: u8,

    #[bits(3)]
    _p: u8,

    enable: bool,
    int_flag: bool,
    start_busy: bool,
});
//Next is HostReg 6.  This sets the sample length.  There are four possible settings to bit length.  All other settings are not normally used and should be a 0.
pub const AnyMeas_BitLength: u8 = HostReg__6;
pub const ADCCTRL_BIT_LENGTH: u8 = 0x03  /* Bit 1, 0 */;
pub const ADCCTRL_SAMPLES_32: u8 = 0x00; //Note: this does not work.
pub const ADCCTRL_SAMPLES_128: u8 = 0x01;
pub const ADCCTRL_SAMPLES_256: u8 = 0x02;
pub const ADCCTRL_SAMPLES_512: u8 = 0x03;
pub const ADCCTRL_ENABLE: u8 = 0x20  /* Bit 5 */;
pub const ADCCTRL_INT_FLAG: u8 = 0x40  /* Bit 6 */;
pub const ADCCTRL_START_BUSY: u8 = 0x80  /* Bit 7 */;
//The smaller the sample length the faster the measurement but the lower the SNR.  For high SNR requirements 512 sample length is recommended.  Alternatively, multiple 128 or 256 length measurements could be averaged.

//Next is HostReg 7.  This sets the sense mux.  Pinnacle has 2 sense lines, Sense N and Sense P1.  There is also a Sense P2 but it is not bonded out, it is only internal.
//Signal on Sense N will be inverted from signal on Sense P1.  Other than sign inversion, signal strength should be the same.
pub const AnyMeas_ADC_MuxControl: u8 = HostReg__7;
pub const ADCMUXCTRL_SENSEP1GATE: u8 = 0x01; //Enables Sense P1.  Can be combined with Sense N input or exclusivly Sense P1 alone.
pub const ADCMUXCTRL_SENSEP2GATE: u8 = 0x02; //Not used.
pub const ADCMUXCTRL_SENSENGATE: u8 = 0x04; //Enables Sense N.  Can be combined with Sense P inputs or exclusivly Sense N alone.
pub const ADCMUXCTRL_REF0GATE: u8 = 0x08; //This enables the RefCap0.  This is a capacitor inside the chip that is roughly 0.25pF. It is also controlled with the toggle and polarity bits so those bits must be set properly as well in order to use it.
pub const ADCMUXCTRL_REF1GATE: u8 = 0x10; //This enables the RefCap1.  This is a capacitor inside the chip that is roughly 0.5pF. It is also controlled with the toggle and polarity bits so those bits must be set properly as well in order to use it.
pub const ADCMUXCTRL_OSCMEASEN: u8 = 0x80; //this is a test mode for measuring the internal oscillator.  It is for IC test only.

//Next is HostReg 8.  This contains various ADC config settings that are not likely to be used.
pub const AnyMeas_ADC_Config2: u8 = HostReg__8;
pub const ADCCNFG2_ADC_CLK_SELECT: u8 = 0x01  /* Bit 0 */   ; //If 0 use the standard 8Mhz clock.  If 1 use a divide by 2, 4Mhz clock.  Only used if extra slow toggle frequencies are required.
pub const ADCCNFG2_EMI_FLAG: u8 = 0x02  /* Bit 1 */   ; //EMI flag threshold only used with internal FW.  Not valid in anymeas mode.
pub const ADCCNFG2_EMI_FLAG_THRESHOLD_0: u8 = 0x04  /* Bit 2 */   ; //EMI flag threshold only used with internal FW.  Not valid in anymeas mode.
pub const ADCCNFG2_EMI_FLAG_THRESHOLD_1: u8 = 0x08  /* Bit 3 */   ; //EMI flag threshold only used with internal FW.  Not valid in anymeas mode.
pub const ADCCNFG2_DSX2_EXTEND: u8 = 0x10  /* Bit 4 */   ; //extend one signal on the receive.  Could also be helpful in situations where sensor cap is extremely high.
pub const ADCCNFG2_ETOGGLE_DELAY: u8 = 0x20  /* Bit 5 */   ; //delay a bit before toggling electrodes.  Could be helpful in situations where sensor cap is extremely high.

//Next is HostReg 9.  This sets the aperture length.  Bottom 4 bits set the aperture width
pub const AnyMeas_ADC_AWidth: u8 = HostReg__9;
pub const ADCAWIDTH_AWIDTHMASK: u8 = 0x0F;
pub const ADCAWIDTH_APERTURE_OPEN: u8 = 0x00; //does not work
pub const ADCAWIDTH_APERTURE_125NS: u8 = 0x01; //does not work
pub const ADCAWIDTH_APERTURE_250NS: u8 = 0x02;
pub const ADCAWIDTH_APERTURE_375NS: u8 = 0x03;
pub const ADCAWIDTH_APERTURE_500NS: u8 = 0x04;
pub const ADCAWIDTH_APERTURE_625NS: u8 = 0x05;
pub const ADCAWIDTH_APERTURE_750NS: u8 = 0x06;
pub const ADCAWIDTH_APERTURE_875NS: u8 = 0x07;
pub const ADCAWIDTH_APERTURE_1000NS: u8 = 0x08;
pub const ADCAWIDTH_APERTURE_1125NS: u8 = 0x09;
pub const ADCAWIDTH_APERTURE_1250NS: u8 = 0x0A;
pub const ADCAWIDTH_APERTURE_1375NS: u8 = 0x0B;
pub const ADCAWIDTH_APERTURE_1500NS: u8 = 0x0C;
pub const ADCAWIDTH_APERTURE_1625NS: u8 = 0x0D;
pub const ADCAWIDTH_APERTURE_1750NS: u8 = 0x0E;
pub const ADCAWIDTH_APERTURE_1875NS: u8 = 0x0F;
pub const ADCAWIDTH_AWIDTHPLUSHALF: u8 = 0x10;
pub const ADCAWIDTH_AOPEN: u8 = 0x20;
pub const ADCAWIDTH_W2WAIT: u8 = 0x40;

//next two registers give the high and low bytes to the 16 bit address where Pinnacle will pull the measurement data.  Normally these addresses are within the base 32 registers.
pub const AnyMeas_pADCMeasInfoStart_High_Byte: u8 = HostReg__10;
pub const AnyMeas_pADCMeasInfoStart_Low_Byte: u8 = HostReg__11;

//Next is the measurement index, this sets the measurement state machine to the start and should be a 0 at start.
pub const AnyMeas_MeasIndex: u8 = HostReg__12;
pub const ANYMEASSTATE_RESET_START: u8 = 0x00;
pub const ANYMEASSTATE_START_MEASUREMENT: u8 = 0x01;
pub const ANYMEASSTATE_WAIT_FOR_MEASUREMENT_AND_HOST: u8 = 0x02;

//next is the state itself of the measurement, should always be 0.
pub const AnyMeas_State: u8 = HostReg__13;

//next is the number of measurements.  Use 0x80 to repeat the single measurement or repeat a number of measurements.
//0x40 will turn the ADC off after measurements.  This will result in longer startup time for a subsequent measurement, but lower idle power draw.
pub const AnyMeas_Control_NumMeas: u8 = HostReg__14;
pub const ANYMEAS_CONTROL__NUM_MEAS_MASK: u8 = 0x3F;
pub const ANYMEAS_CONTROL__ADC_POST_MEAS_PWR: u8 = 0x40;
pub const ANYMEAS_CONTROL__REPEAT: u8 = 0x80;

//These are not used
pub const AnyMeas_pADCMeasInfo_High_Byte: u8 = HostReg__15;
pub const AnyMeas_pADCMeasInfo_Low_Byte: u8 = HostReg__16;

//16 bit result of measurement will be found in these two registers.
pub const AnyMeas_Result_High_Byte: u8 = HostReg__17;
pub const AnyMeas_Result_Low_Byte: u8 = HostReg__18;

// ---------------- Extended Register Assignments ----------------------------
/*--------------------------------------------------------------------------*\
ADC Mux Control
\*--------------------------------------------------------------------------*/
pub const EXTREG__ADCMUX_CTRL: u8 = 0x00EB;
pub const EXTREG__ADCMUX_CTRL__SNSP_ENABLE: u8 = 0x01;
pub const EXTREG__ADCMUX_CTRL__SNSN_ENABLE: u8 = 0x04;

/*--------------------------------------------------------------------------*\
Timer Reload Registers
\*--------------------------------------------------------------------------*/
pub const EXTREG__PACKET_TIMER_RELOAD: u16 = 0x019F;
pub const EXTREG__TRACK_TIMER_RELOAD: u16 = 0x019E;
// These two registers should have matching content.
pub const EXTREG__TIMER_RELOAD__300_SPS: u8 = 0x06;
pub const EXTREG__TIMER_RELOAD__200_SPS: u8 = 0x09;
pub const EXTREG__TIMER_RELOAD__100_SPS: u8 = 0x13;

/*--------------------------------------------------------------------------*\
                       Track ADC Config
\*--------------------------------------------------------------------------*/
register!(u16, TrackAdcConfig, 0x0187, 0x4e, {
    #[bits(6)]
    _p: u8,

    #[bits(2)]
    pub attenuate: AdcAttenuation,
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, num_enum::FromPrimitive, num_enum::IntoPrimitive)]
#[repr(u8)]
pub enum AdcAttenuation {
    #[num_enum(default)]
    X1 = 0,
    X2 = 1,
    X3 = 2,
    X4 = 3,
}

impl AdcAttenuation {
    const fn into_bits(self) -> u8 {
        self as _
    }

    const fn from_bits(value: u8) -> Self {
        match value {
            0 => Self::X1,
            1 => Self::X2,
            2 => Self::X3,
            _ => Self::X4,
        }
    }
}

/*--------------------------------------------------------------------------*\
                        Tune Edge Sensitivity
\*--------------------------------------------------------------------------*/
// These registers are not detailed in any publically available documentation
// Names inferred from debug prints in https://github.com/cirque-corp/Cirque_Pinnacle_1CA027/blob/master/Circular_Trackpad

register!(u16, XAxisWideZMin, 0x0149, 0x06);
register!(u16, YAxisWideZMin, 0x0168, 0x05);
