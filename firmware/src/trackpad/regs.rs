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

pub trait Register {
    const REG: u8;

    fn from_byte(b: u8) -> Self;
    fn to_byte(self) -> u8;
    fn def() -> Self;
}

macro_rules! register {
    ($name:ident, $reg:expr, $def:expr) => {
        pub struct $name(pub u8);

        impl Register for $name {
            const REG: u8 = $reg;

            fn from_byte(b: u8) -> Self {
                Self(b)
            }

            fn to_byte(self) -> u8 {
                self.0
            }

            fn def() -> Self {
                Self($def)
            }
        }
    };
    ($name:ident, $reg:expr, $def:expr, $tt:tt) => {
        #[::bitfield_struct::bitfield(u8)]
        pub struct $name $tt

        impl Register for $name {
            const REG: u8 = $reg;

            fn from_byte(b: u8) -> Self {
                Self::from(b)
            }

            fn to_byte(self) -> u8 {
                u8::from(self)
            }

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
register!(ChipId, HostReg__0, 0);

// Chip Version Register
register!(Version, HostReg__1, 0);

/*--------------------------------------------------------------------------*\
Status Register
\*--------------------------------------------------------------------------*/
// Status 1 Register -- MUST BE HOSTREG__2
register!(Status, HostReg__2, 0, {
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
pub const HOSTREG__SYSCONFIG1: u8 = HostReg__3;
pub const HOSTREG__SYSCONFIG1__RESET: u8 = 0x01;
pub const HOSTREG__SYSCONFIG1__STANDBY: u8 = 0x02;
pub const HOSTREG__SYSCONFIG1__AUTO_SLEEP: u8 = 0x04;
pub const HOSTREG__SYSCONFIG1__TRACK_DISABLE: u8 = 0x08;
pub const HOSTREG__SYSCONFIG1__ANYMEAS_ENABLE: u8 = 0x10;
pub const HOSTREG__SYSCONFIG1__GPIO_CTRL_ENABLE: u8 = 0x20;
pub const HOSTREG__SYSCONFIG1__WAKEUP_TOGGLE: u8 = 0x40;
pub const HOSTREG__SYSCONFIG1__FORCE_WAKEUP: u8 = 0x80;
pub const HOSTREG__SYSCONFIG1_DEFVAL: u8 = 0x00;

/*--------------------------------------------------------------------------*\
Feed Config Registers
\*--------------------------------------------------------------------------*/
// Feed Config Register1
register!(FeedConfig, HostReg__4, 0, {
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
pub const HOSTREG__FEEDCONFIG2: u8 = HostReg__5;
pub const HOSTREG__FEEDCONFIG2__INTELLIMOUSE_MODE: u8 = 0x01;
pub const HOSTREG__FEEDCONFIG2__ALL_TAP_DISABLE: u8 = 0x02;
pub const HOSTREG__FEEDCONFIG2__SECONDARY_TAP_DISABLE: u8 = 0x04;
pub const HOSTREG__FEEDCONFIG2__SCROLL_DISABLE: u8 = 0x08;
pub const HOSTREG__FEEDCONFIG2__GLIDE_EXTEND_DISABLE: u8 = 0x10;
pub const HOSTREG__FEEDCONFIG2__PALM_BEFORE_Z_ENABLE: u8 = 0x20;
pub const HOSTREG__FEEDCONFIG2__BUTNS_46_SCROLL_5_MIDDLE: u8 = 0x40;
pub const HOSTREG__FEEDCONFIG2__SWAP_XY_RELATIVE: u8 = 0x80;
pub const HOSTREG__FEEDCONFIG2_DEFVAL: u8 = 0x00;

// Feed Config Register3
pub const HOSTREG__FEEDCONFIG3: u8 = HostReg__6;
pub const HOSTREG__FEEDCONFIG3__BTNS_456_TO_123_IN_REL: u8 = 0x01;
pub const HOSTREG__FEEDCONFIG3__DISABLE_CROSS_RATE_SMOOTHING: u8 = 0x02;
pub const HOSTREG__FEEDCONFIG3__DISABLE_PALM_NERD_MEAS: u8 = 0x04;
pub const HOSTREG__FEEDCONFIG3__DISABLE_NOISE_AVOIDANCE: u8 = 0x08;
pub const HOSTREG__FEEDCONFIG3__DISABLE_WRAP_LOCKOUT: u8 = 0x10;
pub const HOSTREG__FEEDCONFIG3__DISABLE_DYNAMIC_EMI_ADJUST: u8 = 0x20;
pub const HOSTREG__FEEDCONFIG3__DISABLE_HW_EMI_DETECT: u8 = 0x40;
pub const HOSTREG__FEEDCONFIG3__DISABLE_SW_EMI_DETECT: u8 = 0x80;
pub const HOSTREG__FEEDCONFIG3_DEFVAL: u8 = 0x00;

/*--------------------------------------------------------------------------*\
Calibration Config
\*--------------------------------------------------------------------------*/
pub const HOSTREG__CALCONFIG1: u8 = HostReg__7;
pub const HOSTREG__CALCONFIG1__CALIBRATE: u8 = 0x01;
pub const HOSTREG__CALCONFIG1__BACKGROUND_COMP_ENABLE: u8 = 0x02;
pub const HOSTREG__CALCONFIG1__NERD_COMP_ENABLE: u8 = 0x04;
pub const HOSTREG__CALCONFIG1__TRACK_ERROR_COMP_ENABLE: u8 = 0x08;
pub const HOSTREG__CALCONFIG1__TAP_COMP_ENABLE: u8 = 0x10;
pub const HOSTREG__CALCONFIG1__PALM_ERROR_COMP_ENABLE: u8 = 0x20;
pub const HOSTREG__CALCONFIG1__CALIBRATION_MATRIX_DISABLE: u8 = 0x40;
pub const HOSTREG__CALCONFIG1__FORCE_PRECALIBRATION_NOISE_CHECK: u8 = 0x80;
pub const HOSTREG__CALCONFIG1_DEFVAL: u8 = (HOSTREG__CALCONFIG1__BACKGROUND_COMP_ENABLE
    | HOSTREG__CALCONFIG1__NERD_COMP_ENABLE
    | HOSTREG__CALCONFIG1__TRACK_ERROR_COMP_ENABLE
    | HOSTREG__CALCONFIG1__TAP_COMP_ENABLE
    | HOSTREG__CALCONFIG1__PALM_ERROR_COMP_ENABLE);

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
pub const HOSTREG__SAMPLERATE: u8 = HostReg__9;
pub const HOSTREG__SAMPLERATE__10_SPS: u8 = 0x0A;
pub const HOSTREG__SAMPLERATE__20_SPS: u8 = 0x14;
pub const HOSTREG__SAMPLERATE__40_SPS: u8 = 0x28;
pub const HOSTREG__SAMPLERATE__60_SPS: u8 = 0x3C;
pub const HOSTREG__SAMPLERATE__80_SPS: u8 = 0x50;
pub const HOSTREG__SAMPLERATE__100_SPS: u8 = 0x64;
pub const HOSTREG__SAMPLERATE__200_SPS: u8 = 0xC8; // 200sps not supported
                                                   // only for ps2 compatibility
                                                   // rate set to 100sps
pub const HOSTREG__SAMPLERATE_DEFVAL: u8 = HOSTREG__SAMPLERATE__100_SPS;

/*--------------------------------------------------------------------------*\
Z Idle Value
\*--------------------------------------------------------------------------*/
pub const HOSTREG__ZIDLE: u8 = HostReg__10;
pub const HOSTREG__ZIDLE_DEFVAL: u8 = 30; // 0x1E

/*--------------------------------------------------------------------------*\
Z Scaler Value
\*--------------------------------------------------------------------------*/
pub const HOSTREG__ZSCALER: u8 = HostReg__11;
pub const HOSTREG__ZSCALER_DEFVAL: u8 = 8; // 0x08

/*--------------------------------------------------------------------------*\
Sleep Interval Value
\*--------------------------------------------------------------------------*/
pub const HOSTREG__SLEEP_INTERVAL: u8 = HostReg__12;
pub const HOSTREG__SLEEP_INTERVAL_DEFVAL: u8 = 73; // 0x49

/*--------------------------------------------------------------------------*\
Sleep Delay Value
\*--------------------------------------------------------------------------*/
pub const HOSTREG__SLEEP_DELAY: u8 = HostReg__13;
pub const HOSTREG__SLEEP_DELAY_DEFVAL: u8 = 39; // 0x27

/*--------------------------------------------------------------------------*\
Dynamic EMI Bad Channel Count Thresholds
\*--------------------------------------------------------------------------*/
pub const HOSTREG__DYNAMIC_EMI_ADJUST_THRESHOLD: u8 = HostReg__14;
pub const HOSTREG__DYNAMIC_EMI_ADJUST_THRESHOLD_DEFVAL: u8 = 66; // 0x42

/*--------------------------------------------------------------------------*\
Packet Registers
\*--------------------------------------------------------------------------*/
pub const HOSTREG__PACKETBYTE_0: u8 = HostReg__18;
pub const HOSTREG__PACKETBYTE_1: u8 = HostReg__19;
pub const HOSTREG__PACKETBYTE_2: u8 = HostReg__20;
pub const HOSTREG__PACKETBYTE_3: u8 = HostReg__21;
pub const HOSTREG__PACKETBYTE_4: u8 = HostReg__22;
pub const HOSTREG__PACKETBYTE_5: u8 = HostReg__23;

/*--------------------------------------------------------------------------*\
Port A GPIO Control
\*--------------------------------------------------------------------------*/
pub const HOSTREG__PORTA_GPIO_CTRL: u8 = HostReg__24;
pub const HOSTREG__PORTA_GPIO_CTRL_DEFVAL: u8 = 0xFF;

/*--------------------------------------------------------------------------*\
Port A GPIO Data
\*--------------------------------------------------------------------------*/
pub const HOSTREG__PORTA_GPIO_DATA: u8 = HostReg__25;
pub const HOSTREG__PORTA_GPIO_DATA_DEFVAL: u8 = 0x00;

/*--------------------------------------------------------------------------*\
Port B GPIO Control And Data
\*--------------------------------------------------------------------------*/

pub const HOSTREG__PORTB_GPIO_CTRL_DATA: u8 = HostReg__26;
pub const HOSTREG__PORTB_GPIO_DATA__PB0: u8 = 0x01;
pub const HOSTREG__PORTB_GPIO_DATA__PB1: u8 = 0x02;
pub const HOSTREG__PORTB_GPIO_DATA__PB2: u8 = 0x04;
pub const HOSTREG__PORTB_GPIO_CTRL__PB0: u8 = 0x08;
pub const HOSTREG__PORTB_GPIO_CTRL__PB1: u8 = 0x10;
pub const HOSTREG__PORTB_GPIO_CTRL__PB2: u8 = 0x20;
pub const HOSTREG__PORTB_GPIO_RSVD_0: u8 = 0x40;
pub const HOSTREG__PORTB_GPIO_READ1_WRITE0: u8 = 0x80;
pub const HOSTREG__PORTB_GPIO_CTRL_DATA_DEFVAL: u8 =
    (HOSTREG__PORTB_GPIO_CTRL__PB0 | HOSTREG__PORTB_GPIO_CTRL__PB1 | HOSTREG__PORTB_GPIO_CTRL__PB2);

/*--------------------------------------------------------------------------*\
Extended Register Access
\*--------------------------------------------------------------------------*/
pub const HOSTREG__EXT_REG_AXS_VALUE: u8 = HostReg__27;

pub const HOSTREG__EXT_REG_AXS_ADDR_HIGH: u8 = HostReg__28;
pub const HOSTREG__EXT_REG_AXS_ADDR_LOW: u8 = HostReg__29;

pub const HOSTREG__EXT_REG_AXS_CTRL: u8 = HostReg__30;
pub const HOSTREG__EREG_AXS__READ: u8 = 0x01;
pub const HOSTREG__EREG_AXS__WRITE: u8 = 0x02;
pub const HOSTREG__EREG_AXS__INC_ADDR_READ: u8 = 0x04;
pub const HOSTREG__EREG_AXS__INC_ADDR_WRITE: u8 = 0x08;
pub const HOSTREG__EREG_AXS__RSVD_3: u8 = 0x10;
pub const HOSTREG__EREG_AXS__RSVD_2: u8 = 0x20;
pub const HOSTREG__EREG_AXS__RSVD_1: u8 = 0x40;
pub const HOSTREG__EREG_AXS__RSVD_0: u8 = 0x80;

pub const HOSTREG__EXT_REG_AXS_VALUE_DEFVAL: u8 = 0x00;
pub const HOSTREG__EXT_REG_AXS_ADDR_HIGH_DEFVAL: u8 = 0x00;
pub const HOSTREG__EXT_REG_AXS_ADDR_LOW_DEFVAL: u8 = 0x00;
pub const HOSTREG__EXT_REG_AXS_CTRL_DEFVAL: u8 = 0x00;

/*--------------------------------------------------------------------------*\
Product ID
\*--------------------------------------------------------------------------*/
pub const HOSTREG__PRODUCT_ID: u8 = HostReg__31;

//Some useful values
pub const I2C_ADDRESS_DEFAULT: u8 = 0x2A;
pub const FIRMWARE_ID: u8 = 0x07;
pub const FIRMWARE_VERSION: u8 = 0x9D;

//Anymeas config options
//First setting is HostReg 5.  This sets toggle frequency (EF) and gain.
//Gain is upper two bits (0xC0), frequency is lower 6 bits (0x3F)
pub const AnyMeas_AccumBits_ElecFreq: u8 = HostReg__5;
pub const ADCCNFG_ELEC_FREQ: u8 = 0x3F  /* Bit 4, 3, 2, 1, 0 */;
pub const ADCCNFG_EF_0: u8 = 0x02; // 500,000Hz
pub const ADCCNFG_EF_1: u8 = 0x03; // 444,444Hz
pub const ADCCNFG_EF_2: u8 = 0x04; // 400,000Hz
pub const ADCCNFG_EF_3: u8 = 0x05; // 363,636Hz
pub const ADCCNFG_EF_4: u8 = 0x06; // 333,333Hz
pub const ADCCNFG_EF_5: u8 = 0x07; // 307,692Hz
pub const ADCCNFG_EF_6: u8 = 0x09; // 267,000Hz
pub const ADCCNFG_EF_7: u8 = 0x0B; // 235,000Hz
pub const ADCCNFG_ACCUMBITSSELECT: u8 = 0xC0  /* Bit 7, 6 */;
pub const ADCCNFG_ACCBITS_17_14_0: u8 = 0x00; //This is about 2x gain
pub const ADCCNFG_ACCBITS_17_15_1: u8 = 0x40; //This is about 1.6x gain
pub const ADCCNFG_ACCBITS_17_2__80: u8 = 0x80; //This is about 1.3x gain
pub const ADCCNFG_ACCBITS_17_2__C0: u8 = 0xC0; //This is lowest gain
                                               //Note, all frequencies above are based on default 500ns aperture.  If aperture is shorter the frequencies will be faster and if aperture is longer the frequencies will be slower.

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
pub const EXTREG__TRACK_ADCCONFIG: u16 = 0x0187;
// ADC-attenuation settings (held in BIT_7 and BIT_6)
// 1X = most sensitive, 4X = least sensitive
pub const EXTREG__TRACK_ADCCONFIG__ADC_ATTENUATE_MASK: u8 = 0xC0;
pub const EXTREG__TRACK_ADCCONFIG__ADC_ATTENUATE_1X: u8 = 0x00;
pub const EXTREG__TRACK_ADCCONFIG__ADC_ATTENUATE_2X: u8 = 0x40;
pub const EXTREG__TRACK_ADCCONFIG__ADC_ATTENUATE_3X: u8 = 0x80;
pub const EXTREG__TRACK_ADCCONFIG__ADC_ATTENUATE_4X: u8 = 0xC0;
pub const EXTREG__TRACK_ADCCONFIG_DEFVAL: u8 = 0x4E;

/*--------------------------------------------------------------------------*\
                        Tune Edge Sensitivity
\*--------------------------------------------------------------------------*/
// These registers are not detailed in any publically available documentation
// Names inferred from debug prints in https://github.com/cirque-corp/Cirque_Pinnacle_1CA027/blob/master/Circular_Trackpad
pub const EXTREG__XAXIS_WIDEZMIN: u16 = 0x0149;
pub const EXTREG__YAXIS_WIDEZMIN: u16 = 0x0168;
pub const EXTREG__XAXIS_WIDEZMIN_DEFVAL: u16 = 0x06;
pub const EXTREG__YAXIS_WIDEZMIN_DEFVAL: u16 = 0x05;
