const DEF_VGA_GEN_MOR_OUT_IO:        u8  = 0x01;        /* i/o address select       */
const DEF_VGA_GEN_MOR_OUT_ER:        u8  = 0x02;        /* enable ram               */
const DEF_VGA_GEN_MOR_OUT_CLK:       u8  = 0x0C;        /* clock select             */
const DEF_VGA_GEN_MOR_OUT_CLK_25175: u8  = 0x00;        /* 25.175MHz                */
const DEF_VGA_GEN_MOR_OUT_CLK_28322: u8  = 0x04;        /* 28.322                   */
const DEF_VGA_GEN_MOR_OUT_CLK_EXT:   u8  = 0x08;        /* external clock           */
const DEF_VGA_GEN_MOR_OUT_CLK_RSV:   u8  = 0x0C;        /* reserved                 */
const DEF_VGA_GEN_MOR_OUT_PS:        u8  = 0x20;        /* page select              */
const DEF_VGA_GEN_MOR_OUT_PS_UPPER:  u8  = 0x00;        /* upper 64KB page          */
const DEF_VGA_GEN_MOR_OUT_PS_LOWER:  u8  = 0x20;        /* lower 64KB page          */
const DEF_VGA_GEN_MOR_OUT_HSP:       u8  = 0x40;        /* horizontal sync polarity */
const DEF_VGA_GEN_MOR_OUT_VSP:       u8  = 0x80;        /* vertical sync polarity   */
const DEF_VGA_GEN_MOR_OUT_SP:        u8  = 0xC0;        /* sync polarity            */
const DEF_VGA_GEN_MOR_OUT_SP_RSV:    u8  = 0x00;        /* reserved                 */
const DEF_VGA_GEN_MOR_OUT_SP_400L:   u8  = 0x40;        /* 400 lines                */
const DEF_VGA_GEN_MOR_OUT_SP_350L:   u8  = 0x80;        /* 350 lines                */
const DEF_VGA_GEN_MOR_OUT_SP_480L:   u8  = 0xC0;        /* 480 lines                */
const DEF_VGA_GEN_MOR_OUT_RSVD:      u8  = 0x10;        /* reserved bits            */

const DEF_VGA_GEN_ISR1_DE:           u8  = 0x01;        /* display enabel           */
const DEF_VGA_GEN_ISR1_VR:           u8  = 0x08;        /* vertical retrace         */
const DEF_VGA_GEN_ISR1_VF:           u8  = 0x30;        /* video feedback           */

const DEF_PORT_VGA_SEQ_ADDR:         u16 = 0x03C4;      /* Address Register         */
const DEF_PORT_VGA_SEQ_DATA:         u16 = 0x03C5;      /* Data Register            */

const DEF_VGA_SEQ_NUM_REGS:          u8 = 0x05;         /* number of internal registers */

const DEF_VGA_SEQ_ADDR_INDX:         u8 = 0x07;         /* index                    */
const DEF_VGA_SEQ_ADDR_RSVD:         u8 = 0xF8;         /* reserved bits            */

const DEF_VGA_SEQ_RESET_INDX:        u8 = 0x00;         /* register index           */
const DEF_VGA_SEQ_RESET_AR:          u8 = 0x01;         /* asynchronous reset bit   */
const DEF_VGA_SEQ_RESET_SR:          u8 = 0x02;         /* synchronous reset bit    */
const DEF_VGA_SEQ_RESET_ASR:         u8 = 0x03;         /* both reset bit           */
const DEF_VGA_SEQ_RESET_RSVD:        u8 = 0xFC;         /* reserved bits            */

const DEF_VGA_SEQ_CLK_MODE_INDX: u8 = 0x01;             /* register index           */
const DEF_VGA_SEQ_CLK_MODE_89DC: u8 = 0x01;             /* 8/9 dot clocks           */
const DEF_VGA_SEQ_CLK_MODE_SL:   u8 = 0x04;             /* shift load               */
const DEF_VGA_SEQ_CLK_MODE_DC:   u8 = 0x08;             /* dot clock                */
const DEF_VGA_SEQ_CLK_MODE_S4:   u8 = 0x10;             /* shift 4                  */
const DEF_VGA_SEQ_CLK_MODE_SO:   u8 = 0x20;             /* screen off               */
const DEF_VGA_SEQ_CLK_MODE_RSVD: u8 = 0xC2;             /* reserved bits            */

const DEF_VGA_SEQ_MAP_MASK_INDX:     u8 = 0x02;         /* register index           */
const DEF_VGA_SEQ_MAP_MASK_MAP0E:    u8 = 0x01;         /* map0 enable              */
const DEF_VGA_SEQ_MAP_MASK_MAP1E:    u8 = 0x02;         /* map1 enable              */
const DEF_VGA_SEQ_MAP_MASK_MAP2E:    u8 = 0x04;         /* map2 enable              */
const DEF_VGA_SEQ_MAP_MASK_MAP3E:    u8 = 0x08;         /* map3 enable              */
const DEF_VGA_SEQ_MAP_MASK_MAP_MASK: u8 = 0x0F;         /* mask bit                 */
const DEF_VGA_SEQ_MAP_MASK_RSVD:     u8 = 0xF0;         /* reserved bits            */

const DEF_VGA_SEQ_CHAR_MAP_INDX: u8 = 0x03;             /* register index           */
const DEF_VGA_SEQ_CHAR_MAPA:     u8 = 0x2C;             /* mask of Map A            */
const DEF_VGA_SEQ_CHAR_MAPB:     u8 = 0x13;             /* mask of Map B            */
const DEF_VGA_SEQ_CHAR_MAP_RSVD: u8 = 0xC0;             /* reserved bits            */

const DEF_VGA_SEQ_CHAR_MAPA1ST: u8 = 0x00;              /* 1st 8KB of Map2          */
const DEF_VGA_SEQ_CHAR_MAPA2ND: u8 = 0x20;              /* 2nd 8KB of Map2          */
const DEF_VGA_SEQ_CHAR_MAPA3RD: u8 = 0x04;              /* 3rd 8KB of Map2          */
const DEF_VGA_SEQ_CHAR_MAPA4TH: u8 = 0x24;              /* 4th 8KB of Map2          */
const DEF_VGA_SEQ_CHAR_MAPA5TH: u8 = 0x08;              /* 5th 8KB of Map2          */
const DEF_VGA_SEQ_CHAR_MAPA6TH: u8 = 0x28;              /* 6th 8KB of Map2          */
const DEF_VGA_SEQ_CHAR_MAPA7TH: u8 = 0x0C;              /* 7th 8KB of Map2          */
const DEF_VGA_SEQ_CHAR_MAPA8TH: u8 = 0x2C;              /* 8th 8KB of Map2          */

const DEF_VGA_SEQ_CHAR_MAPB1ST: u8 = 0x00;              /* 1st 8KB of Map2          */
const DEF_VGA_SEQ_CHAR_MAPB2ND: u8 = 0x10;              /* 2nd 8KB of Map2          */
const DEF_VGA_SEQ_CHAR_MAPB3RD: u8 = 0x01;              /* 3rd 8KB of Map2          */
const DEF_VGA_SEQ_CHAR_MAPB4TH: u8 = 0x11;              /* 4th 8KB of Map2          */
const DEF_VGA_SEQ_CHAR_MAPB5TH: u8 = 0x02;              /* 5th 8KB of Map2          */
const DEF_VGA_SEQ_CHAR_MAPB6TH: u8 = 0x12;              /* 6th 8KB of Map2          */
const DEF_VGA_SEQ_CHAR_MAPB7TH: u8 = 0x03;              /* 7th 8KB of Map2          */
const DEF_VGA_SEQ_CHAR_MAPB8TH: u8 = 0x13;              /* 8th 8KB of Map2          */

const DEF_VGA_SEQ_MEM_MODE_INDX: u8 = 0x04;             /* register index           */
const DEF_VGA_SEQ_MEM_MODE_EM:   u8 = 0x02;             /* extended memory          */
const DEF_VGA_SEQ_MEM_MODE_OE:   u8 = 0x04;             /* odd or even              */
const DEF_VGA_SEQ_MEM_MODE_CH4:  u8 = 0x08;             /* chain 4                  */
const DEF_VGA_SEQ_MEM_MODE_RSVD: u8 = 0xF1;             /* reserved bits            */

const DEF_VGA_SEQ_HOR_CHAR_CNT_INDX: u8 = 0x04;         /* register index           */

const DEF_PORT_VGA_CRT_ADDR0: u16 = 0x03B4;             /* Address Register(MOR0 = 0) */
const DEF_PORT_VGA_CRT_DATA0: u16 = 0x03B5;             /* Data Register(MOR0 = 0)    */
const DEF_PORT_VGA_CRT_ADDR1: u16 = 0x03D4;             /* Address Register(MOR0 = 1) */
const DEF_PORT_VGA_CRT_DATA1: u16 = 0x03D5;             /* Data Register(MOR0 = 1)    */

const DEF_VGA_CRT_HOR_TOT_INDX: u8 = 0x00;              /* register index           */

const DEF_VGA_CRT_HOR_DISP_EN_INDX: u8 = 0x01;          /* register index            */

const DEF_VGA_CRT_ST_HOR_BL_INDX: u8 = 0x02;            /* register index            */

const DEF_VGA_CRT_END_HOR_BLK_INDX: u8 = 0x03;          /* register index           */
const DEF_VGA_CRT_END_HOR_BLK_EB:   u8 = 0x18;          /* end blanking             */
const DFF_VGA_CRT_END_HOR_DES:      u8 = 0x60;          /* deisplay enable skew     */
const DFF_VGA_CRT_END_HOR_DES_0:    u8 = 0x00;          /* no character clock skew  */
const DFF_VGA_CRT_END_HOR_DES_1:    u8 = 0x20;          /* 1 character clock skew   */
const DFF_VGA_CRT_END_HOR_DES_2:    u8 = 0x40;          /* 2 character clocks skew  */
const DFF_VGA_CRT_END_HOR_DES_3:    u8 = 0x60;          /* 3 character clocks skew  */
const DEF_VGA_CRT_END_HOR_RSVD:     u8 = 0x80;          /* reserved bit. set 1      */

const DEF_VGA_CRT_STA_HOR_RTP_INDX: u8 = 0x04;          /* register index            */

const DEF_VGA_CRT_END_HOR_RTP_INDX: u8 = 0x05;          /* register index           */
const DEF_VGA_CRT_END_HOR_RTP_EHR:  u8 = 0x1F;          /* end horizontal retrace   */
const DEF_VGA_CRT_END_HOR_RTP_HRD:  u8 = 0x60;          /* horizontal retrace delay */
const DEF_VGA_CRT_END_HOR_RTP_EHB5: u8 = 0x80;          /* end horizonta blanking b5*/

const DEF_VGA_CRT_VERT_TOT_INDX: u8 = 0x06;             /* register index            */

const DEF_VGA_CRT_OVERFLOW_INDX:  u8 = 0x07;            /* register index           */
const DEF_VGA_CRT_OVERFLOW_VT8:   u8 = 0x01;            /* bit 8 of vertical total  */
const DEF_VGA_CRT_OVERFLOW_VDEE8: u8 = 0x02;            /* bit 8 of vert disp enable*/
const DEF_VGA_CRT_OVERFLOW_VRS8:  u8 = 0x04;            /* bit 8 of vert retrace sta*/
const DEF_VGA_CRT_OVERFLOW_VBS8:  u8 = 0x08;            /* bit 8 of vert blk start  */
const DEF_VGA_CRT_OVERFLOW_LC8:   u8 = 0x10;            /* bit 8 of line compare    */
const DEF_VGA_CRT_OVERFLOW_VT9:   u8 = 0x20;            /* bit 9 of vertical total  */
const DEF_VGA_CRT_OVERFLOW_VDEE9: u8 = 0x40;            /* bit 9 of vert dis en end */
const DEF_VGA_CRT_OVERFLOW_VRS9:  u8 = 0x80;            /* bit 9 of vert retrace sta*/

const DEF_VGA_CRT_PRE_ROW_SCAN_INDX:   u8 = 0x08;       /* register index           */
const DEF_VGA_CRT_PRE_ROW_SCAN_SRS:    u8 = 0x1F;       /* starting row scan count  */
const DEF_VGA_CRT_PRE_ROW_SCAN_BP:     u8 = 0x60;       /* byte panning             */
const DEF_VGA_CRT_PRE_ROW_SCAN_BP8_00: u8 = 0x00;       /* panning(8-px,shift 00)   */
const DEF_VGA_CRT_PRE_ROW_SCAN_BP8_08: u8 = 0x20;       /* panning(8-px,shift 08)   */
const DEF_VGA_CRT_PRE_ROW_SCAN_BP8_16: u8 = 0x40;       /* panning(8-px,shift 16)   */
const DEF_VGA_CRT_PRE_ROW_SCAN_BP8_24: u8 = 0x60;       /* panning(8-px,shift 24)   */
const DEF_VGA_CRT_PRE_ROW_SCAN_BP9_00: u8 = 0x00;       /* panning(9-px,shift 00)   */
const DEF_VGA_CRT_PRE_ROW_SCAN_BP9_09: u8 = 0x20;       /* panning(9-px,shift 09)   */
const DEF_VGA_CRT_PRE_ROW_SCAN_BP9_18: u8 = 0x40;       /* panning(9-px,shift 18)   */
const DEF_VGA_CRT_PRE_ROW_SCAN_BP9_27: u8 = 0x60;       /* panning(9-px,shift 27)   */
const DEF_VGA_CRT_PRE_ROW_SCAN_RSVD:   u8 = 0x80;       /* reserved bit             */

const DEF_VGA_CRT_MAX_SCAN_LINE_INDX: u8 = 0x09;        /* register index           */
const DEF_VGA_CRT_MAX_SCAN_LINE_MSL:  u8 = 0x1F;        /* maximum scan line        */
const DEF_VGA_CRT_MAX_SCAN_LINE_SVB9: u8 = 0x20;        /* bit 9 of start vert blnk */
const DEF_VGA_CRT_MAX_SCAN_LINE_LC9:  u8 = 0x40;        /* bit 9 of line compare    */
const DEF_VGA_CRT_MAX_SCAN_LINE_LC:   u8 = 0x80;        /* line conversion          */

const DEF_VGA_CRT_CUR_START_INDX: u8 = 0x0A;            /* register index           */
const DEF_VGA_CRT_CUR_START_RSCB: u8 = 0x1F;            /* row scan cursor begins   */
const DEF_VGA_CRT_CUR_START_CO:   u8 = 0x20;            /* cursor off               */
const DEF_VGA_CRT_CUR_START_RSVD: u8 = 0xC0;            /* reserved bits            */

const DEF_VGA_CRT_CUR_END_INDX:  u8 = 0x0B;             /* register index           */
const DEF_VGA_CRT_CUR_END_RSCE:  u8 = 0x1F;             /* row scan cursor ends     */
const DEF_VGA_CRT_CUR_END_CSK:   u8 = 0x60;             /* cursor skew control      */
const DEF_VGA_CRT_CUR_END_CSK_0: u8 = 0x00;             /* cursor skew 0            */
const DEF_VGA_CRT_CUR_END_CSK_1: u8 = 0x20;             /* cursor skew 1            */
const DEF_VGA_CRT_CUR_END_CSK_2: u8 = 0x40;             /* cursor skew 2            */
const DEF_VGA_CRT_CUR_END_CSK_3: u8 = 0x60;             /* cursor skew 3            */
const DEF_VGA_CRT_CUR_END_RSVD:  u8 = 0x80;             /* reserved bit             */

const DEF_VGA_CRT_START_ADDR_HI_INDX: u8 = 0x0C;        /* register index           */

const DEF_VGA_CRT_START_ADDR_LO_INDX: u8 = 0x0D;        /* register index           */

const DEF_VGA_CRT_CUR_LOC_HI_INDX: u8 = 0x0E;           /* register index           */

const DEF_VGA_CRT_CUR_LOC_LO_INDX: u8 = 0x0F;           /* register index           */

const DEF_VGA_CRT_VER_RT_START_INDX: u8 = 0x10;         /* register index           */

const DEF_VGA_CRT_VER_RT_END_INDX:  u8 = 0x11;          /* register index           */
const DEF_VGA_CRT_VER_RT_END_CVI:   u8 = 0x10;          /* clear vertical interrupt */
const DEF_VGA_CRT_VER_RT_END_EVI:   u8 = 0x20;          /* enable vert interrupt    */
const DEF_VGA_CRT_VER_RT_END_S5RC:  u8 = 0x40;          /* select 5 refresh cycle   */
const DEF_VGA_CRT_VER_RT_END_PR0_7: u8 = 0x80;          /* protect registers 0-7    */

const DEF_VGA_CRT_VER_DISP_EN_INDX: u8 = 0x12;          /* register index           */

const DEF_VGA_CRT_OFFSET_INDX: u8 = 0x13;               /* register index           */

const DEF_VGA_CRT_ULINE_LOC_INDX: u8 = 0x14;            /* register index           */
const DEF_VGA_CRT_ULINE_LOC_SU:   u8 = 0x1F;            /* start underline          */
const DEF_VGA_CRT_ULINE_LOC_C4:   u8 = 0x20;            /* count by 4               */
const DEF_VGA_CRT_ULINE_LOC_DW:   u8 = 0x40;            /* doubleword mode          */
const DEF_VGA_CRT_ULINE_LOC_RSVD: u8 = 0x80;            /* reserved bit             */

const DEF_VGA_CRT_START_VER_BLK_INDX: u8 = 0x15;        /* register index            */

const DEF_VGA_CRT_END_VER_BLK_INDX: u8 = 0x16;          /* register index            */

const DEF_VGA_CRT_MODE_CTRL_INDX: u8 = 0x17;            /* register index           */
const DEF_VGA_CRT_MODE_CTRL_CMS0: u8 = 0x01;            /* CMS 0                    */
const DEF_VGA_CRT_MODE_CTRL_SRSC: u8 = 0x02;            /* select row scan counter  */
const DEF_VGA_CRT_MODE_CTRL_HRS:  u8 = 0x04;            /* horizontal retrace select*/
const DEF_VGA_CRT_MODE_CTRL_C2:   u8 = 0x08;            /* count by 2               */
const DEF_VGA_CRT_MODE_CTRL_AW:   u8 = 0x20;            /* address wrap             */
const DEF_VGA_CRT_MODE_CTRL_WBM:  u8 = 0x40;            /* word/byte mode           */
const DEF_VGA_CRT_MODE_CTRL_HR:   u8 = 0x80;            /* hardware reset           */
const DEF_VGA_CRT_MODE_CTRL_RSVD: u8 = 0x10;            /* reserved                 */

const DEF_VGA_CRT_LINE_COMPARE_INDX: u8 = 0x18;         /* register index           */

const DEF_PORT_VGA_GF_ADDR: u16 = 0x03CE;               /* Address Register         */
const DEF_PORT_VGA_GF_DATA: u16 = 0x03CF;               /* Data Register            */

const DEF_VGA_GF_SET_RESET_INDX: u8 = 0x00;             /* register index           */
const DEF_VGA_GF_SET_RESET_SRM0: u8 = 0x01;             /* set/reset Map 0          */
const DEF_VGA_GF_SET_RESET_SRM1: u8 = 0x02;             /* set/reset Map 1          */
const DEF_VGA_GF_SET_RESET_SRM2: u8 = 0x04;             /* set/reset Map 2          */
const DEF_VGA_GF_SET_RESET_SRM3: u8 = 0x08;             /* set/reset Map 3          */
const DEF_VGA_GF_SET_RESET_RSVD: u8 = 0xF0;             /* reserved bits            */

const DEF_VGA_GF_SET_RESET_EN_INDX:  u8 = 0x01;         /* register index           */
const DEF_VGA_GF_SET_RESET_EN_ESRM0: u8 = 0x01;         /* enable set/reset Map 0   */
const DEF_VGA_GF_SET_RESET_EN_ESRM1: u8 = 0x02;         /* enable set/reset Map 1   */
const DEF_VGA_GF_SET_RESET_EN_ESRM2: u8 = 0x04;         /* enable set/reset Map 2   */
const DEF_VGA_GF_SET_RESET_EN_ESRM3: u8 = 0x08;         /* enable set/reset Map 3   */
const DEF_VGA_GF_SET_RESET_EN_RSVD:  u8 = 0xF0;         /* reserved bits            */

const DEF_VGA_GF_COLOR_COMP_INDX: u8 = 0x02;            /* register index           */
const DEF_VGA_GF_COLOR_COMP_CC0:  u8 = 0x01;            /* color compare map0       */
const DEF_VGA_GF_COLOR_COMP_CC1:  u8 = 0x02;            /* color compare map1       */
const DEF_VGA_GF_COLOR_COMP_CC2:  u8 = 0x04;            /* color compare map2       */
const DEF_VGA_GF_COLOR_COMP_CC3:  u8 = 0x08;            /* color compare map3       */
const DEF_VGA_GF_COLOR_COMP_RSVD: u8 = 0xF0;            /* reserved bits            */

const DEF_VGA_GF_DATA_ROTATE_INDX:   u8 = 0x03;         /* register index           */
const DEF_VGA_GF_DATA_ROTATE_RC:     u8 = 0x07;         /* rotate count             */
const DEF_VGA_GF_DATA_ROTATE_FS:     u8 = 0x18;         /* function select          */
const DEF_VGA_GF_DATA_ROTATE_FS_UM:  u8 = 0x00;         /* unmodified               */
const DEF_VGA_GF_DATA_ROTATE_FS_AND: u8 = 0x08;         /* AND function             */
const DEF_VGA_GF_DATA_ROTATE_FS_OR:  u8 = 0x10;         /* OR function              */
const DEF_VGA_GF_DATA_ROTATE_FS_XOR: u8 = 0x18;         /* XOR function             */
const DEF_VGA_GF_DATA_ROTATE_RSVD:   u8 = 0xE0;         /* reserved bits            */

const DEF_VGA_GF_READ_MS_INDX: u8 = 0x04;               /* register index           */
const DEF_VGA_GF_READ_MS_MS:   u8 = 0x03;               /* map select               */
const DEF_VGA_GF_READ_MS_MAP0: u8 = 0x00;               /* Map0                     */
const DEF_VGA_GF_READ_MS_MAP1: u8 = 0x01;               /* Map1                     */
const DEF_VGA_GF_READ_MS_MAP2: u8 = 0x02;               /* Map2                     */
const DEF_VGA_GF_READ_MS_MAP3: u8 = 0x03;               /* Map3                     */
const DEF_VGA_GF_READ_MS_RSVD: u8 = 0xFC;               /* reserved bits            */

const DEF_VGA_GF_MODE_INDX:  u8 = 0x05;                 /* register index           */
const DEF_VGA_GF_MODE_WM:    u8 = 0x03;                 /* write mode               */
const DEF_VGA_GF_MODE_WM_0:  u8 = 0x00;                 /* write mode 0             */
const DEF_VGA_GF_MODE_WM_1:  u8 = 0x01;                 /* write mode 1             */
const DEF_VGA_GF_MODE_WM_2:  u8 = 0x02;                 /* write mode 2             */
const DEF_VGA_GF_MODE_WM_3:  u8 = 0x03;                 /* write mode 3             */
const DEF_VGA_GF_MODE_RM:    u8 = 0x08;                 /* read mode                */
const DEF_VGA_GF_MODE_RM_0:  u8 = 0x00;                 /* read mode 0              */
const DEF_VGA_GF_MODE_RM_1:  u8 = 0x08;                 /* read mode 1              */
const DEF_VGA_GF_MODE_OE:    u8 = 0x10;                 /* odd/even                 */
const DEF_VGA_GF_MODE_SR:    u8 = 0x20;                 /* shift register mode      */
const DEF_VGA_GF_MODE_256CM: u8 = 0x40;                 /* 256-color mode           */
const DEF_VGA_GF_MODE_RSVD:  u8 = 0x84;                 /* reserved bits            */

const DEF_VGA_GF_MISC_INDX:         u8 = 0x06;          /* register index           */
const DEF_VGA_GF_MISC_GM:           u8 = 0x01;          /* graphics mode            */
const DEF_VGA_GF_MISC_GM_TEXT:      u8 = 0x00;          /* select text mode         */
const DEF_VGA_GF_MISC_GM_GRAPHICS:  u8 = 0x01;          /* select graphics mode     */
const DEF_VGA_GF_MISC_OE:           u8 = 0x02;          /* odd/even                 */
const DEF_VGA_GF_MISC_OE_SEQ:       u8 = 0x00;          /* sequencial addressing    */
const DEF_VGA_GF_MISC_OE_OE:        u8 = 0x02;          /* odd/even mode            */
const DEF_VGA_GF_MISC_MM:           u8 = 0x0C;          /* memory map               */
const DEF_VGA_GF_MISC_MM_A0000_128: u8 = 0x00;          /* 0xA0000, 128KB           */
const DEF_VGA_GF_MISC_MM_A0000_64:  u8 = 0x04;          /* 0xA0000, 64KB            */
const DEF_VGA_GF_MISC_MM_B0000_32:  u8 = 0x08;          /* 0xB0000, 32KB            */
const DEF_VGA_GF_MISC_MM_B8000_32:  u8 = 0x0C;          /* 0xB8000, 32KB            */
const DEF_VGA_GF_MISC_RSVD:         u8 = 0xF0;          /* reserved bits            */

const DEF_VGA_GF_COL_DONT_CARE_INDX: u8 = 0x07;         /* register index           */
const DEF_VGA_GF_COL_DONT_CARE_DC:   u8 = 0x0F;         /* don't care               */
const DEF_VGA_GF_COL_DONT_CARE_DC_0: u8 = 0x01;         /* don't care map0          */
const DEF_VGA_GF_COL_DONT_CARE_DC_1: u8 = 0x02;         /* don't care map1          */
const DEF_VGA_GF_COL_DONT_CARE_DC_2: u8 = 0x04;         /* don't care map2          */
const DEF_VGA_GF_COL_DONT_CARE_DC_3: u8 = 0x08;         /* don't care map3          */
const DEF_VGA_GF_COL_DONT_CARE_RSVD: u8 = 0xF0;         /* reserved bits            */

const DEF_VGA_GF_BIT_MASK_INDX: u8 = 0x08;              /* register index           */
const DEF_VGA_GF_BIT_MASK_BIT0: u8 = 0x01;              /* bit0                     */
const DEF_VGA_GF_BIT_MASK_BIT1: u8 = 0x02;              /* bit1                     */
const DEF_VGA_GF_BIT_MASK_BIT2: u8 = 0x04;              /* bit2                     */
const DEF_VGA_GF_BIT_MASK_BIT3: u8 = 0x08;              /* bit3                     */
const DEF_VGA_GF_BIT_MASK_BIT4: u8 = 0x10;              /* bit4                     */
const DEF_VGA_GF_BIT_MASK_BIT5: u8 = 0x20;              /* bit5                     */
const DEF_VGA_GF_BIT_MASK_BIT6: u8 = 0x40;              /* bit6                     */
const DEF_VGA_GF_BIT_MASK_BIT7: u8 = 0x80;              /* bit7                     */

const DEF_PORT_VGA_ATTR_ADDR:  u16 = 0x03C0;            /* Address Register                     */
const DEF_PORT_VGA_ATTR_DAT_W: u16 = 0x03C0;            /* Data Registe(Write)                  */
const DEF_PORT_VGA_ATTR_DAT_R: u16 = 0x03C1;            /* Data Registe(Read)                   */

const DEF_VGA_ATTR_PALETTE_0_INDX: u8 = 0x00;           /* register index           */
const DEF_VGA_ATTR_PALETTE_1_INDX: u8 = 0x01;           /* register index           */
const DEF_VGA_ATTR_PALETTE_2_INDX: u8 = 0x02;           /* register index           */
const DEF_VGA_ATTR_PALETTE_3_INDX: u8 = 0x03;           /* register index           */
const DEF_VGA_ATTR_PALETTE_4_INDX: u8 = 0x04;           /* register index           */
const DEF_VGA_ATTR_PALETTE_5_INDX: u8 = 0x05;           /* register index           */
const DEF_VGA_ATTR_PALETTE_6_INDX: u8 = 0x06;           /* register index           */
const DEF_VGA_ATTR_PALETTE_7_INDX: u8 = 0x07;           /* register index           */
const DEF_VGA_ATTR_PALETTE_8_INDX: u8 = 0x08;           /* register index           */
const DEF_VGA_ATTR_PALETTE_9_INDX: u8 = 0x09;           /* register index           */
const DEF_VGA_ATTR_PALETTE_A_INDX: u8 = 0x0A;           /* register index           */
const DEF_VGA_ATTR_PALETTE_B_INDX: u8 = 0x0B;           /* register index           */
const DEF_VGA_ATTR_PALETTE_C_INDX: u8 = 0x0C;           /* register index           */
const DEF_VGA_ATTR_PALETTE_D_INDX: u8 = 0x0D;           /* register index           */
const DEF_VGA_ATTR_PALETTE_E_INDX: u8 = 0x0E;           /* register index           */
const DEF_VGA_ATTR_PALETTE_F_INDX: u8 = 0x0F;           /* register index           */

const DEF_VGA_ATTR_PALETTE_DATA: u8 = 0x3F;             /* palette data(common)     */
const DEF_VGA_ATTR_PALETTE_RSVD: u8 = 0xC0;             /* reserved bits(common)    */

const DEF_VGA_ATTR_MODE_NINDX: u8 = 0x20 | 0x10;        /* register index set bit*/
const DEF_VGA_ATTR_MODE_INDX:  u8 = 0x10;               /* register index           */
const DEF_VGA_ATTR_MODE_GAM:   u8 = 0x01;               /* graphics/text mode       */
const DEF_VGA_ATTR_MODE_ME:    u8 = 0x02;               /* mono emulation           */
const DEF_VGA_ATTR_MODE_ELGCC: u8 = 0x04;               /* enable line gra char code*/
const DEF_VGA_ATTR_MODE_EBSBI: u8 = 0x08;               /* enable blink/intensity   */
const DEF_VGA_ATTR_MODE_PELPC: u8 = 0x20;               /* pel panning compatibility*/
const DEF_VGA_ATTR_MODE_PELW:  u8 = 0x40;               /* pel width                */
const DEF_VGA_ATTR_MODE_P54S:  u8 = 0x80;               /* P5, P4 select            */
const DEF_VGA_ATTR_MODE_RSVD:  u8 = 0x10;               /* reserved bit             */

const DEF_VGA_ATTR_OS_COLOR_NINDX: u8 = 0x20 | 0x11;    /* register index set bit*/
const DEF_VGA_ATTR_OS_COLOR_INDX:  u8 = 0x11;           /* register index       */

const DEF_VGA_ATTR_C_PLANE_EN_NINDX: u8 = 0x20 | 0x12;  /* register index set bit*/
const DEF_VGA_ATTR_C_PLANE_EN_INDX:  u8 = 0x12;         /* register index           */
const DEF_VGA_ATTR_C_PLANE_EN_ECP:   u8 = 0x0F;         /* enable color plane       */
const DEF_VGA_ATTR_C_PLANE_EN_ECP0:  u8 = 0x01;         /* plane 0                  */
const DEF_VGA_ATTR_C_PLANE_EN_ECP1:  u8 = 0x02;         /* plane 1                  */
const DEF_VGA_ATTR_C_PLANE_EN_ECP2:  u8 = 0x04;         /* plane 3                  */
const DEF_VGA_ATTR_C_PLANE_EN_ECP3:  u8 = 0x08;         /* plane 4                  */
const DEF_VGA_ATTR_C_PLANE_VSM:      u8 = 0x30;         /* Video Status Mux         */
const DEF_VGA_ATTR_C_PLANE_VSM_P2P0: u8 = 0x00;         /* P2=ST1 5, P0=ST1 4       */
const DEF_VGA_ATTR_C_PLANE_VSM_P5P4: u8 = 0x10;         /* P5=ST1 5, P4=ST1 4       */
const DEF_VGA_ATTR_C_PLANE_VSM_P3P1: u8 = 0x20;         /* P3=ST1 5, P1=ST1 4       */
const DEF_VGA_ATTR_C_PLANE_VSM_P7P6: u8 = 0x30;         /* P7=ST1 5, P6=ST1 4       */
const DEF_VGA_ATTR_C_PLANE_EN_RSVD:  u8 = 0xC0;         /* reserved bits            */

const DEF_VGA_ATTR_HOR_PEL_PAN_NINDX: u8 = 0x20 | 0x13; /* register index set bit*/
const DEF_VGA_ATTR_HOR_PEL_PAN_INDX:  u8 = 0x13;        /* register index           */
const DEF_VGA_ATTR_HOR_PEL_PAN_HPP:   u8 = 0x0F;        /* horizontal pel pannning  */
const DEF_VGA_ATTR_HOR_PEL_PAN_HPP_0: u8 = 0x00;        /* pel 0                    */
const DEF_VGA_ATTR_HOR_PEL_PAN_HPP_1: u8 = 0x01;        /* pel 1                    */
const DEF_VGA_ATTR_HOR_PEL_PAN_HPP_2: u8 = 0x02;        /* pel 2                    */
const DEF_VGA_ATTR_HOR_PEL_PAN_HPP_3: u8 = 0x03;        /* pel 3                    */
const DEF_VGA_ATTR_HOR_PEL_PAN_HPP_4: u8 = 0x04;        /* pel 4                    */
const DEF_VGA_ATTR_HOR_PEL_PAN_HPP_5: u8 = 0x05;        /* pel 5                    */
const DEF_VGA_ATTR_HOR_PEL_PAN_HPP_6: u8 = 0x06;        /* pel 6                    */
const DEF_VGA_ATTR_HOR_PEL_PAN_HPP_7: u8 = 0x07;        /* pel 7                    */
const DEF_VGA_ATTR_HOR_PEL_PAN_HPP_8: u8 = 0x08;        /* pel 8                    */
const DEF_VGA_ATTR_HOR_PEL_PAN_RSVD:  u8 = 0xF0;        /* reserved bits            */

const DEF_VGA_ATTR_COLOR_SEL_NINDX: u8 = 0x20 | 0x14;   /* register index set bit*/
const DEF_VGA_ATTR_COLOR_SEL_INDX:  u8 = 0x14;          /* register index           */
const DEF_VGA_ATTR_COLOR_SEL_SC4:   u8 = 0x01;          /* S_color4                 */
const DEF_VGA_ATTR_COLOR_SEL_SC5:   u8 = 0x02;          /* S_color5                 */
const DEF_VGA_ATTR_COLOR_SEL_SC45:  u8 = 0x03;          /* S_color4, 5              */
const DEF_VGA_ATTR_COLOR_SEL_SC6:   u8 = 0x04;          /* S_color6                 */
const DEF_VGA_ATTR_COLOR_SEL_SC7:   u8 = 0x08;          /* S_color7                 */
const DEF_VGA_ATTR_COLOR_SEL_SC67:  u8 = 0x0C;          /* S_color6, 7              */
const DEF_VGA_ATTR_COLOR_SEL_RSVD:  u8 = 0xF0;          /* reserved bits            */

impl Vga {
    fn write_mor(misc: u8) {
        let o = inb(VGA_MISC_READ);
        outb(VGA_MISC_WRITE, misc | (o & VGA_MISC_RESERVED));
    }

    fn write_seq_address(index: u8) -> u8 {
        let o = inb(DEF_PORT_VGA_SEQ_ADDR);
        outb(DEF_PORT_VGA_SEQ_ADDR, index | (o & DEF_VGA_SEQ_ADDR_RSVD));
        o
    }

    fn write_seq_data(data: u8) -> u8 {
        let o = inb(DEF_PORT_VGA_SEQ_DATA);
        outb(DEF_PORT_VGA_SEQ_DATA, data);
        o
    }

    fn read_seq(index: u8) -> u8 {
        VgaRegister::write_seq_address(index);
        inb(DEF_PORT_VGA_SEQ_DATA)
    }

    fn write_seq(index: u8, data: u8) {
        let old_data = VgaRegister::read_seq();
        let mask = match index {
            DEF_VGA_SEQ_RESET_INDX => DEF_VGA_SEQ_RESET_RSVD,
            DEF_VGA_SEQ_CLK_MODE_INDX => DEF_VGA_SEQ_CLK_MODE_RSVD,
            DEF_VGA_SEQ_MAP_MASK_INDX => DEF_VGA_SEQ_MAP_MASK_RSVD,
            DEF_VGA_SEQ_CHAR_MAP_INDX => DEF_VGA_SEQ_CHAR_MAP_RSVD,
            DEF_VGA_SEQ_MEM_MODE_INDX => DEF_VGA_SEQ_MEM_MODE_RSVD,
            _ => 0x00
        };

        let data = (old_data & mask) | (data & !mask);
        VgaRegister::write_seq_address(index);
        VgaRegister::write_seq_data();

        old_data
    }

    fn read_crtc(index: u8) -> u8 {
        /* ------------------------------------------------------------------------ */
        /* color emulation                                                          */
        /* ------------------------------------------------------------------------ */
        outb(DEF_PORT_VGA_CRT_ADDR1, index);
        inb(DEF_PORT_VGA_CRT_DATA1)
    }

    fn write_crtc(index: u8, data: u8) -> u8 {
        /* ------------------------------------------------------------------------ */
        /* read register and preserve reserved bits, then write                     */
        /* ------------------------------------------------------------------------ */
        let old_data = VgaRegister::read_crtc(index);
        let mask = match(index) {
            DEF_VGA_CRT_END_HOR_BLK_INDX  => DEF_VGA_CRT_END_HOR_RSVD,
            DEF_VGA_CRT_PRE_ROW_SCAN_INDX => DEF_VGA_CRT_PRE_ROW_SCAN_RSVD,
            DEF_VGA_CRT_CUR_START_INDX    => DEF_VGA_CRT_CUR_START_RSVD,
            DEF_VGA_CRT_CUR_END_INDX      => DEF_VGA_CRT_CUR_END_RSVD,
            DEF_VGA_CRT_ULINE_LOC_INDX    => DEF_VGA_CRT_ULINE_LOC_RSVD,
            DEF_VGA_CRT_MODE_CTRL_INDX    => DEF_VGA_CRT_MODE_CTRL_RSVD,
            _ => 0x00
        };

        let new_data = if index == DEF_VGA_CRT_END_HOR_BLK_INDX {
            data | DEF_VGA_CRT_END_HOR_RSVD
        } else {
            (old_data & mask) | (data & !mask)
        };

        /* ------------------------------------------------------------------------ */
        /* color emulation                                                          */
        /* ------------------------------------------------------------------------ */
        outb(DEF_PORT_VGA_CRT_ADDR1, index);
        outb(DEF_PORT_VGA_CRT_DATA1, new_data);

        old_data
    }

    fn lock_crtc() {
        let data = VgaRegister::read_crtc(DEF_VGA_CRT_VER_RT_END_INDX);

        /* ------------------------------------------------------------------------ */
        /* set lock bit                                                             */
        /* ------------------------------------------------------------------------ */
        VgaRegister::write_crtc(DEF_VGA_CRT_VER_RT_END_INDX, data | DEF_VGA_CRT_VER_RT_END_PR0_7);
    }

    fn unlock_crtc() {
        let data = VgaRegister::read_crtc(DEF_VGA_CRT_VER_RT_END_INDX);
        /* ------------------------------------------------------------------------ */
        /* clear lock bit                                                           */
        /* ------------------------------------------------------------------------ */
        VgaRegister::write_crtc(DEF_VGA_CRT_VER_RT_END_INDX, data & !DEF_VGA_CRT_VER_RT_END_PR0_7);
    }

    fn vga_screen_on() {
        let clock_mode = VgaRegister::read_seq(DEF_VGA_SEQ_CLK_MODE_INDX);

        /* ------------------------------------------------------------------------ */
        /* clear screen off bit                                                     */
        /* ------------------------------------------------------------------------ */
        VgaRegister::write_seq(DEF_VGA_SEQ_CLK_MODE_INDX, clock_mode & !DEF_VGA_SEQ_CLK_MODE_SO);
    }

    fn vga_screen_off() {
        let clock_mode = VgaRegister::read_seq(DEF_VGA_SEQ_CLK_MODE_INDX);

        /* ------------------------------------------------------------------------ */
        /* set screen off bit                                                       */
        /* ------------------------------------------------------------------------ */
        VgaRegister::write_seq(DEF_VGA_SEQ_CLK_MODE_INDX, clock_mode | DEF_VGA_SEQ_CLK_MODE_SO);
    }

    fn write_gfc_address(index: u8) -> u8 {
        /* ------------------------------------------------------------------------ */
        /* read register and preserve reserved bits, then write                     */
        /* ------------------------------------------------------------------------ */
        let old_index = inb(DEF_PORT_VGA_GF_ADDR);
        outb( DEF_PORT_VGA_GF_ADDR, index | (old_index & DEF_VGA_GF_ADDR_RSVD));

        old_index
    }

    fn write_gfc_data(index: u8, data: u8) -> u8 {
        /* ------------------------------------------------------------------------ */
        /* read register and preserve reserved bits, then write                     */
        /* ------------------------------------------------------------------------ */
        let old_data = inb(DEF_PORT_VGA_GF_DATA);
        outb(DEF_PORT_VGA_GF_DATA, data);
        old_data
    }

    fn set_vga_write_mode(mode: u8) {
        /* ------------------------------------------------------------------------ */
        /* read gf mode register                                                    */
        /* ------------------------------------------------------------------------ */
        let read_data = read_gfc(DEF_VGA_GF_MODE_INDX);
        /* ------------------------------------------------------------------------ */
        /* clear write mode bit                                                     */
        /* ------------------------------------------------------------------------ */
        let mut write_data = read_data & !DEF_VGA_GF_MODE_WM;
        /* ------------------------------------------------------------------------ */
        /* set write mode bit                                                       */
        /* ------------------------------------------------------------------------ */
        write_data = write_data | ( mode & DEF_VGA_GF_MODE_WM );

        write_gfc(DEF_VGA_GF_MODE_INDX, write_data);
    }

    fn set_vga_read_mode(mode: u8) {
        /* ------------------------------------------------------------------------ */
        /* read gf mode register                                                    */
        /* ------------------------------------------------------------------------ */
        let read_data = read_gfc(DEF_VGA_GF_MODE_INDX);
        /* ------------------------------------------------------------------------ */
        /* clear read mode bit                                                      */
        /* ------------------------------------------------------------------------ */
        let mut write_data = read_data & !DEF_VGA_GF_MODE_RM;
        /* ------------------------------------------------------------------------ */
        /* set read mode bit                                                        */
        /* ------------------------------------------------------------------------ */
        write_data = write_data | ((mode << 3) & DEF_VGA_GF_MODE_RM);

        write_gfc(DEF_VGA_GF_MODE_INDX, write_data);
    }

    pub fn load(&self) {
        /* ------------------------------------------------------------------------ */
        /* set color emulation params to misellaneous register                      */
        /* ------------------------------------------------------------------------ */
        VgaRegister::write_mor(self.misc | DEF_VGA_GEN_MOR_OUT_IO);

        /* ------------------------------------------------------------------------ */
        /* synchronous reset                                                        */
        /* ------------------------------------------------------------------------ */
        VgaRegister::write_seq(DEF_VGA_SEQ_RESET_INDX, DEF_VGA_SEQ_RESET_AR);
        /* ------------------------------------------------------------------------ */
        /* set clock mode and screen off                                            */
        /* ------------------------------------------------------------------------ */
        VgaRegister::write_seq(DEF_VGA_SEQ_CLK_MODE_INDX,
                  self.seq[DEF_VGA_SEQ_CLK_MODE_INDX] |
                  DEF_VGA_SEQ_CLK_MODE_SO);

        /* ------------------------------------------------------------------------ */
        /* set sequencer params                                                     */
        /* ------------------------------------------------------------------------ */
        for i in DEF_VGA_SEQ_MAP_MASK_INDX..DEF_VGA_SEQ_NUM_REGS {
            VgaRegister::write_seq(i as u8, self.seq[i]);
        }

        /* ------------------------------------------------------------------------ */
        /* release reset state                                                      */
        /* ------------------------------------------------------------------------ */
        VgaRegister::write_seq(DEF_VGA_SEQ_RESET_INDX,
                  self.seq[0] |
                  DEF_VGA_SEQ_RESET_AR | DEF_VGA_SEQ_RESET_SR);

        /* ------------------------------------------------------------------------ */
        /* unlock crt controller                                                    */
        /* ------------------------------------------------------------------------ */
        VgaRegister::unlock_crtc();

        /* ------------------------------------------------------------------------ */
        /* set crt params                                                           */
        /* ------------------------------------------------------------------------ */
        for i in DEF_VGA_CRT_HOR_TOT_INDX..DEF_VGA_CRT_NUM_REGS {
            VgaRegister::write_crtc(i as u8, self.crt[i]);
        }

        /* ------------------------------------------------------------------------ */
        /* lock crt controller                                                      */
        /* ------------------------------------------------------------------------ */
        VgaRegister::lock_crtc();

        /* ------------------------------------------------------------------------ */
        /* set graphics params                                                      */
        /* ------------------------------------------------------------------------ */
        for i in DEF_VGA_GF_SET_RESET_INDX..DEF_VGA_GF_NUM_REGS {
            VgaRegister::write_gfc(i as u8, self.gf[i]);
        }

        /* ------------------------------------------------------------------------ */
        /* set attribute params                                                     */
        /* ------------------------------------------------------------------------ */
        for i in DEF_VGA_ATTR_PALETTE_0_INDX..DEF_VGA_ATTR_NUM_REGS {
            if i >= DEF_VGA_ATTR_MODE_INDX {
                VgaRegister::write_attrc(i as u8 |
                            DEF_VGA_ATTR_ADDR_IPAS, self.attr[i]);
            } else {
                VgaRegister::write_attrc(i as u8, self.attr[i]);
            }
        }

        inb(DEF_PORT_VGA_ISR1_R1);
        VgaRegister::write_attrc(DEF_VGA_ATTR_ADDR_IPAS);    // enable video

        /* ------------------------------------------------------------------------ */
        /* wait 50ms for screen to be stable                                        */
        /* ------------------------------------------------------------------------ */
        msdelay(50);

        /* ------------------------------------------------------------------------ */
        /* screen on                                                                */
        /* ------------------------------------------------------------------------ */
        VgaRegister::vga_screen_on();
    }
}

