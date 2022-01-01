const FT62XX_ADDR: u16 = 0x38;

// #define FT62XX_ADDR 0x38           //!< I2C address
// #define FT62XX_G_FT5201ID 0xA8     //!< FocalTech's panel ID
// #define FT62XX_REG_NUMTOUCHES 0x02 //!< Number of touch points

// #define FT62XX_NUM_X 0x33 //!< Touch X position
// #define FT62XX_NUM_Y 0x34 //!< Touch Y position

// #define FT62XX_REG_MODE 0x00        //!< Device mode, either WORKING or FACTORY
// #define FT62XX_REG_CALIBRATE 0x02   //!< Calibrate mode
// #define FT62XX_REG_WORKMODE 0x00    //!< Work mode
// #define FT62XX_REG_FACTORYMODE 0x40 //!< Factory mode
// #define FT62XX_REG_THRESHHOLD 0x80  //!< Threshold for touch detection
// #define FT62XX_REG_POINTRATE 0x88   //!< Point rate
// #define FT62XX_REG_FIRMVERS 0xA6    //!< Firmware version
// #define FT62XX_REG_CHIPID 0xA3      //!< Chip selecting
// #define FT62XX_REG_VENDID 0xA8      //!< FocalTech's panel ID

// #define FT62XX_VENDID 0x11  //!< FocalTech's panel ID
// #define FT6206_CHIPID 0x06  //!< Chip selecting
// #define FT6236_CHIPID 0x36  //!< Chip selecting
// #define FT6236U_CHIPID 0x64 //!< Chip selecting

struct TS_Point {
    x: u16,
    y: u16,
    z: u16, // Often used for pressure
}

struct FT62XX {
    touched: u8,
    touches: [TS_Point; 10],
}

impl FT62XX {
    pub fn get_point(self, n: u8) -> (u16, u16) {
        (0, 0)
    }

    fn read_register(self, reg: u8) -> u8 {
        0
    }

    fn write_register(self, reg: u8, val: u8) {}

    // Return the number of touches currently active:
    fn touched(self) -> u8 {
        (self.touches.len() & 0xFF) as u8
    }
}
