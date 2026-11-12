// Clean-room GTA: San Andreas SCM opcode arity & symbol table derived from gta-reversed.
// Positive `arity` = exact number of typed `eScriptParameterType` arguments;
// `-1` = vararg argument list terminated by `0x00` (`SCRIPT_PARAM_END_OF_ARGUMENTS`);
// `-2` = switch jump table (`u8 count + (i32 value, i32 offset)` pairs).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpcodeShape {
    Fixed(u8),
    VarArgTerminated,
    SwitchTable,
}

#[derive(Debug, Clone, Copy)]
pub struct OpcodeMeta {
    pub id: u16,
    pub name: &'static str,
    pub shape: OpcodeShape,
}

const SA_OPCODE_TABLE: &[OpcodeMeta] = &[
    OpcodeMeta {
        id: 0x0000,
        name: "NOP",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x0001,
        name: "WAIT",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0002,
        name: "GOTO",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0003,
        name: "SHAKE_CAM",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0004,
        name: "SET_VAR_INT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0005,
        name: "SET_VAR_FLOAT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0006,
        name: "SET_LVAR_INT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0007,
        name: "SET_LVAR_FLOAT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0008,
        name: "ADD_VAL_TO_INT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0009,
        name: "ADD_VAL_TO_FLOAT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x000A,
        name: "ADD_VAL_TO_INT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x000B,
        name: "ADD_VAL_TO_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x000C,
        name: "SUB_VAL_FROM_INT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x000D,
        name: "SUB_VAL_FROM_FLOAT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x000E,
        name: "SUB_VAL_FROM_INT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x000F,
        name: "SUB_VAL_FROM_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0010,
        name: "MULT_INT_VAR_BY_VAL",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0011,
        name: "MULT_FLOAT_VAR_BY_VAL",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0012,
        name: "MULT_INT_LVAR_BY_VAL",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0013,
        name: "MULT_FLOAT_LVAR_BY_VAL",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0014,
        name: "DIV_INT_VAR_BY_VAL",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0015,
        name: "DIV_FLOAT_VAR_BY_VAL",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0016,
        name: "DIV_INT_LVAR_BY_VAL",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0017,
        name: "DIV_FLOAT_LVAR_BY_VAL",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0018,
        name: "IS_INT_VAR_GREATER_THAN_NUMBER",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0019,
        name: "IS_INT_LVAR_GREATER_THAN_NUMBER",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x001A,
        name: "IS_NUMBER_GREATER_THAN_INT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x001B,
        name: "IS_NUMBER_GREATER_THAN_INT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x001C,
        name: "IS_INT_VAR_GREATER_THAN_INT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x001D,
        name: "IS_INT_LVAR_GREATER_THAN_INT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x001E,
        name: "IS_INT_VAR_GREATER_THAN_INT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x001F,
        name: "IS_INT_LVAR_GREATER_THAN_INT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0020,
        name: "IS_FLOAT_VAR_GREATER_THAN_NUMBER",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0021,
        name: "IS_FLOAT_LVAR_GREATER_THAN_NUMBER",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0022,
        name: "IS_NUMBER_GREATER_THAN_FLOAT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0023,
        name: "IS_NUMBER_GREATER_THAN_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0024,
        name: "IS_FLOAT_VAR_GREATER_THAN_FLOAT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0025,
        name: "IS_FLOAT_LVAR_GREATER_THAN_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0026,
        name: "IS_FLOAT_VAR_GREATER_THAN_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0027,
        name: "IS_FLOAT_LVAR_GREATER_THAN_FLOAT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0028,
        name: "IS_INT_VAR_GREATER_OR_EQUAL_TO_NUMBER",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0029,
        name: "IS_INT_LVAR_GREATER_OR_EQUAL_TO_NUMBER",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x002A,
        name: "IS_NUMBER_GREATER_OR_EQUAL_TO_INT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x002B,
        name: "IS_NUMBER_GREATER_OR_EQUAL_TO_INT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x002C,
        name: "IS_INT_VAR_GREATER_OR_EQUAL_TO_INT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x002D,
        name: "IS_INT_LVAR_GREATER_OR_EQUAL_TO_INT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x002E,
        name: "IS_INT_VAR_GREATER_OR_EQUAL_TO_INT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x002F,
        name: "IS_INT_LVAR_GREATER_OR_EQUAL_TO_INT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0030,
        name: "IS_FLOAT_VAR_GREATER_OR_EQUAL_TO_NUMBER",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0031,
        name: "IS_FLOAT_LVAR_GREATER_OR_EQUAL_TO_NUMBER",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0032,
        name: "IS_NUMBER_GREATER_OR_EQUAL_TO_FLOAT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0033,
        name: "IS_NUMBER_GREATER_OR_EQUAL_TO_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0034,
        name: "IS_FLOAT_VAR_GREATER_OR_EQUAL_TO_FLOAT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0035,
        name: "IS_FLOAT_LVAR_GREATER_OR_EQUAL_TO_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0036,
        name: "IS_FLOAT_VAR_GREATER_OR_EQUAL_TO_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0037,
        name: "IS_FLOAT_LVAR_GREATER_OR_EQUAL_TO_FLOAT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0038,
        name: "IS_INT_VAR_EQUAL_TO_NUMBER",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0039,
        name: "IS_INT_LVAR_EQUAL_TO_NUMBER",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x003A,
        name: "IS_INT_VAR_EQUAL_TO_INT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x003B,
        name: "IS_INT_LVAR_EQUAL_TO_INT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x003C,
        name: "IS_INT_VAR_EQUAL_TO_INT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0042,
        name: "IS_FLOAT_VAR_EQUAL_TO_NUMBER",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0043,
        name: "IS_FLOAT_LVAR_EQUAL_TO_NUMBER",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0044,
        name: "IS_FLOAT_VAR_EQUAL_TO_FLOAT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0045,
        name: "IS_FLOAT_LVAR_EQUAL_TO_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0046,
        name: "IS_FLOAT_VAR_EQUAL_TO_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x004D,
        name: "GOTO_IF_FALSE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x004E,
        name: "TERMINATE_THIS_SCRIPT",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x004F,
        name: "START_NEW_SCRIPT",
        shape: OpcodeShape::VarArgTerminated,
    },
    OpcodeMeta {
        id: 0x0050,
        name: "GOSUB",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0051,
        name: "RETURN",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x0053,
        name: "CREATE_PLAYER",
        shape: OpcodeShape::Fixed(5),
    },
    OpcodeMeta {
        id: 0x0058,
        name: "ADD_INT_VAR_TO_INT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0059,
        name: "ADD_FLOAT_VAR_TO_FLOAT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x005A,
        name: "ADD_INT_LVAR_TO_INT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x005B,
        name: "ADD_FLOAT_LVAR_TO_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x005C,
        name: "ADD_INT_VAR_TO_INT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x005D,
        name: "ADD_FLOAT_VAR_TO_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x005E,
        name: "ADD_INT_LVAR_TO_INT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x005F,
        name: "ADD_FLOAT_LVAR_TO_FLOAT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0060,
        name: "SUB_INT_VAR_FROM_INT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0061,
        name: "SUB_FLOAT_VAR_FROM_FLOAT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0062,
        name: "SUB_INT_LVAR_FROM_INT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0063,
        name: "SUB_FLOAT_LVAR_FROM_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0064,
        name: "SUB_INT_VAR_FROM_INT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0065,
        name: "SUB_FLOAT_VAR_FROM_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0066,
        name: "SUB_INT_LVAR_FROM_INT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0067,
        name: "SUB_FLOAT_LVAR_FROM_FLOAT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0068,
        name: "MULT_INT_VAR_BY_INT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0069,
        name: "MULT_FLOAT_VAR_BY_FLOAT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x006A,
        name: "MULT_INT_LVAR_BY_INT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x006B,
        name: "MULT_FLOAT_LVAR_BY_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x006C,
        name: "MULT_INT_VAR_BY_INT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x006D,
        name: "MULT_FLOAT_VAR_BY_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x006E,
        name: "MULT_INT_LVAR_BY_INT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x006F,
        name: "MULT_FLOAT_LVAR_BY_FLOAT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0070,
        name: "DIV_INT_VAR_BY_INT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0071,
        name: "DIV_FLOAT_VAR_BY_FLOAT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0072,
        name: "DIV_INT_LVAR_BY_INT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0073,
        name: "DIV_FLOAT_LVAR_BY_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0074,
        name: "DIV_INT_VAR_BY_INT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0075,
        name: "DIV_FLOAT_VAR_BY_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0076,
        name: "DIV_INT_LVAR_BY_INT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0077,
        name: "DIV_FLOAT_LVAR_BY_FLOAT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0078,
        name: "ADD_TIMED_VAL_TO_FLOAT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0079,
        name: "ADD_TIMED_VAL_TO_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x007A,
        name: "ADD_TIMED_FLOAT_VAR_TO_FLOAT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x007B,
        name: "ADD_TIMED_FLOAT_LVAR_TO_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x007C,
        name: "ADD_TIMED_FLOAT_VAR_TO_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x007D,
        name: "ADD_TIMED_FLOAT_LVAR_TO_FLOAT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x007E,
        name: "SUB_TIMED_VAL_FROM_FLOAT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x007F,
        name: "SUB_TIMED_VAL_FROM_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0080,
        name: "SUB_TIMED_FLOAT_VAR_FROM_FLOAT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0081,
        name: "SUB_TIMED_FLOAT_LVAR_FROM_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0082,
        name: "SUB_TIMED_FLOAT_VAR_FROM_FLOAT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0083,
        name: "SUB_TIMED_FLOAT_LVAR_FROM_FLOAT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0084,
        name: "SET_VAR_INT_TO_VAR_INT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0085,
        name: "SET_LVAR_INT_TO_LVAR_INT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0086,
        name: "SET_VAR_FLOAT_TO_VAR_FLOAT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0087,
        name: "SET_LVAR_FLOAT_TO_LVAR_FLOAT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0088,
        name: "SET_VAR_FLOAT_TO_LVAR_FLOAT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0089,
        name: "SET_LVAR_FLOAT_TO_VAR_FLOAT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x008A,
        name: "SET_VAR_INT_TO_LVAR_INT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x008B,
        name: "SET_LVAR_INT_TO_VAR_INT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x008C,
        name: "CSET_VAR_INT_TO_VAR_FLOAT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x008D,
        name: "CSET_VAR_FLOAT_TO_VAR_INT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x008E,
        name: "CSET_LVAR_INT_TO_VAR_FLOAT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x008F,
        name: "CSET_LVAR_FLOAT_TO_VAR_INT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0090,
        name: "CSET_VAR_INT_TO_LVAR_FLOAT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0091,
        name: "CSET_VAR_FLOAT_TO_LVAR_INT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0092,
        name: "CSET_LVAR_INT_TO_LVAR_FLOAT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0093,
        name: "CSET_LVAR_FLOAT_TO_LVAR_INT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0094,
        name: "ABS_VAR_INT",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0095,
        name: "ABS_LVAR_INT",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0096,
        name: "ABS_VAR_FLOAT",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0097,
        name: "ABS_LVAR_FLOAT",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0099,
        name: "GENERATE_RANDOM_INT",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x00A0,
        name: "GET_CHAR_COORDINATES",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x00A1,
        name: "SET_CHAR_COORDINATES",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x00A3,
        name: "IS_CHAR_IN_AREA_2D",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x00A4,
        name: "IS_CHAR_IN_AREA_3D",
        shape: OpcodeShape::Fixed(8),
    },
    OpcodeMeta {
        id: 0x00AE,
        name: "SET_CAR_DRIVING_STYLE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x00BA,
        name: "PRINT_BIG",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x00BB,
        name: "PRINT",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x00BC,
        name: "PRINT_NOW",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x00BE,
        name: "CLEAR_PRINTS",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x00C0,
        name: "SET_TIME_OF_DAY",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x00C1,
        name: "GET_MINUTES_TO_TIME_OF_DAY",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x00C2,
        name: "IS_POINT_ON_SCREEN",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x00C3,
        name: "DEBUG_ON",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x00C4,
        name: "DEBUG_OFF",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x00C5,
        name: "RETURN_TRUE",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x00C6,
        name: "RETURN_FALSE",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x00D6,
        name: "ANDOR",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x00D7,
        name: "LAUNCH_MISSION",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x00D9,
        name: "STORE_CAR_CHAR_IS_IN",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x00DB,
        name: "IS_CHAR_IN_CAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x00DD,
        name: "IS_CHAR_IN_MODEL",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x00DF,
        name: "IS_CHAR_IN_ANY_CAR",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x00E1,
        name: "IS_BUTTON_PRESSED",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x00EC,
        name: "LOCATE_CHAR_ANY_MEANS_2D",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x00ED,
        name: "LOCATE_CHAR_ON_FOOT_2D",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x00EE,
        name: "LOCATE_CHAR_IN_CAR_2D",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x00EF,
        name: "LOCATE_STOPPED_CHAR_ANY_MEANS_2D",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x00F0,
        name: "LOCATE_STOPPED_CHAR_ON_FOOT_2D",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x00F1,
        name: "LOCATE_STOPPED_CHAR_IN_CAR_2D",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x00F2,
        name: "LOCATE_CHAR_ANY_MEANS_CHAR_2D",
        shape: OpcodeShape::Fixed(5),
    },
    OpcodeMeta {
        id: 0x00F3,
        name: "LOCATE_CHAR_ON_FOOT_CHAR_2D",
        shape: OpcodeShape::Fixed(5),
    },
    OpcodeMeta {
        id: 0x00F4,
        name: "LOCATE_CHAR_IN_CAR_CHAR_2D",
        shape: OpcodeShape::Fixed(5),
    },
    OpcodeMeta {
        id: 0x00FE,
        name: "LOCATE_CHAR_ANY_MEANS_3D",
        shape: OpcodeShape::Fixed(8),
    },
    OpcodeMeta {
        id: 0x00FF,
        name: "LOCATE_CHAR_ON_FOOT_3D",
        shape: OpcodeShape::Fixed(8),
    },
    OpcodeMeta {
        id: 0x0100,
        name: "LOCATE_CHAR_IN_CAR_3D",
        shape: OpcodeShape::Fixed(8),
    },
    OpcodeMeta {
        id: 0x0101,
        name: "LOCATE_STOPPED_CHAR_ANY_MEANS_3D",
        shape: OpcodeShape::Fixed(8),
    },
    OpcodeMeta {
        id: 0x0102,
        name: "LOCATE_STOPPED_CHAR_ON_FOOT_3D",
        shape: OpcodeShape::Fixed(8),
    },
    OpcodeMeta {
        id: 0x0103,
        name: "LOCATE_STOPPED_CHAR_IN_CAR_3D",
        shape: OpcodeShape::Fixed(8),
    },
    OpcodeMeta {
        id: 0x0104,
        name: "LOCATE_CHAR_ANY_MEANS_CHAR_3D",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x0105,
        name: "LOCATE_CHAR_ON_FOOT_CHAR_3D",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x0106,
        name: "LOCATE_CHAR_IN_CAR_CHAR_3D",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x0107,
        name: "CREATE_OBJECT",
        shape: OpcodeShape::Fixed(5),
    },
    OpcodeMeta {
        id: 0x0108,
        name: "DELETE_OBJECT",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0109,
        name: "ADD_SCORE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x010A,
        name: "IS_SCORE_GREATER",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x010B,
        name: "STORE_SCORE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x010D,
        name: "ALTER_WANTED_LEVEL",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x010E,
        name: "ALTER_WANTED_LEVEL_NO_DROP",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x010F,
        name: "IS_WANTED_LEVEL_GREATER",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0110,
        name: "CLEAR_WANTED_LEVEL",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0111,
        name: "SET_DEATHARREST_STATE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0117,
        name: "IS_PLAYER_DEAD",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0118,
        name: "IS_CHAR_DEAD",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0122,
        name: "IS_PLAYER_PRESSING_HORN",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0129,
        name: "CREATE_CHAR_INSIDE_CAR",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x014B,
        name: "CREATE_CAR_GENERATOR",
        shape: OpcodeShape::Fixed(13),
    },
    OpcodeMeta {
        id: 0x014C,
        name: "SWITCH_CAR_GENERATOR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x014D,
        name: "ADD_PAGER_MESSAGE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0154,
        name: "IS_CHAR_IN_ZONE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0158,
        name: "POINT_CAMERA_AT_CAR",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x0159,
        name: "POINT_CAMERA_AT_CHAR",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x015A,
        name: "RESTORE_CAMERA",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x015B,
        name: "SHAKE_PAD",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x015F,
        name: "SET_FIXED_CAMERA_POSITION",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x0160,
        name: "POINT_CAMERA_AT_POINT",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x0164,
        name: "REMOVE_BLIP",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0169,
        name: "SET_FADING_COLOUR",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x016A,
        name: "DO_FADE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x016F,
        name: "DRAW_SHADOW",
        shape: OpcodeShape::Fixed(10),
    },
    OpcodeMeta {
        id: 0x0172,
        name: "GET_CHAR_HEADING",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0173,
        name: "SET_CHAR_HEADING",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0176,
        name: "GET_OBJECT_HEADING",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0177,
        name: "SET_OBJECT_HEADING",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0179,
        name: "IS_CHAR_TOUCHING_OBJECT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x017B,
        name: "SET_CHAR_AMMO",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x0180,
        name: "DECLARE_MISSION_FLAG",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0184,
        name: "IS_CHAR_HEALTH_GREATER",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0189,
        name: "ADD_BLIP_FOR_CONTACT_POINT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x018A,
        name: "ADD_BLIP_FOR_COORD",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x01B4,
        name: "SET_PLAYER_CONTROL",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x01B9,
        name: "SET_CURRENT_CHAR_WEAPON",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x01BB,
        name: "GET_OBJECT_COORDINATES",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x01BC,
        name: "SET_OBJECT_COORDINATES",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x01C2,
        name: "MARK_CHAR_AS_NO_LONGER_NEEDED",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x01C4,
        name: "MARK_OBJECT_AS_NO_LONGER_NEEDED",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x01C7,
        name: "DONT_REMOVE_OBJECT",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x01E3,
        name: "PRINT_WITH_NUMBER_BIG",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x01E4,
        name: "PRINT_WITH_NUMBER",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x01E5,
        name: "PRINT_WITH_NUMBER_NOW",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x01E7,
        name: "SWITCH_ROADS_ON",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x01E8,
        name: "SWITCH_ROADS_OFF",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x01F0,
        name: "SET_MAX_WANTED_LEVEL",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x01F3,
        name: "IS_CAR_IN_AIR_PROPER",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x01F5,
        name: "GET_PLAYER_CHAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x01F7,
        name: "SET_POLICE_IGNORE_PLAYER",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0202,
        name: "LOCATE_CHAR_ANY_MEANS_CAR_2D",
        shape: OpcodeShape::Fixed(5),
    },
    OpcodeMeta {
        id: 0x0203,
        name: "LOCATE_CHAR_ON_FOOT_CAR_2D",
        shape: OpcodeShape::Fixed(5),
    },
    OpcodeMeta {
        id: 0x0204,
        name: "LOCATE_CHAR_IN_CAR_CAR_2D",
        shape: OpcodeShape::Fixed(5),
    },
    OpcodeMeta {
        id: 0x0205,
        name: "LOCATE_CHAR_ANY_MEANS_CAR_3D",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x0206,
        name: "LOCATE_CHAR_ON_FOOT_CAR_3D",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x0207,
        name: "LOCATE_CHAR_IN_CAR_CAR_3D",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x0208,
        name: "GENERATE_RANDOM_FLOAT_IN_RANGE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0209,
        name: "GENERATE_RANDOM_INT_IN_RANGE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0213,
        name: "CREATE_PICKUP",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x0217,
        name: "PRINT_BIG_Q",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x0221,
        name: "APPLY_BRAKES_TO_PLAYERS_CAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0223,
        name: "SET_CHAR_HEALTH",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0226,
        name: "GET_CHAR_HEALTH",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x022A,
        name: "SWITCH_PED_ROADS_ON",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x022B,
        name: "SWITCH_PED_ROADS_OFF",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x023B,
        name: "IS_CHAR_TOUCHING_OBJECT_ON_FOOT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0241,
        name: "IS_PLAYER_IN_REMOTE_MODE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0247,
        name: "REQUEST_MODEL",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0248,
        name: "HAS_MODEL_LOADED",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0249,
        name: "MARK_MODEL_AS_NO_LONGER_NEEDED",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0250,
        name: "DRAW_LIGHT",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x0253,
        name: "STORE_CLOCK",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x0254,
        name: "RESTORE_CLOCK",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x0297,
        name: "RESET_NUM_OF_MODELS_KILLED_BY_PLAYER",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0298,
        name: "GET_NUM_OF_MODELS_KILLED_BY_PLAYER",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x02A0,
        name: "IS_CHAR_STOPPED",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x02A3,
        name: "SWITCH_WIDESCREEN",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x02A7,
        name: "ADD_SPRITE_BLIP_FOR_CONTACT_POINT",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x02A8,
        name: "ADD_SPRITE_BLIP_FOR_COORD",
        shape: OpcodeShape::Fixed(5),
    },
    OpcodeMeta {
        id: 0x02A9,
        name: "SET_CHAR_ONLY_DAMAGED_BY_PLAYER",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x02AB,
        name: "SET_CHAR_PROOFS",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x02AC,
        name: "SET_CAR_PROOFS",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x02CB,
        name: "IS_CHAR_ON_SCREEN",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x02CE,
        name: "GET_GROUND_Z_FOR_3D_COORD",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x02CF,
        name: "START_SCRIPT_FIRE",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x02D6,
        name: "IS_CHAR_SHOOTING_IN_AREA",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x02D8,
        name: "IS_CURRENT_CHAR_WEAPON",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x02E0,
        name: "IS_CHAR_SHOOTING",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x02E2,
        name: "SET_CHAR_ACCURACY",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x02E3,
        name: "GET_CAR_SPEED",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x02EB,
        name: "RESTORE_CAMERA_JUMPCUT",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x02F2,
        name: "IS_CHAR_MODEL",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x02F6,
        name: "SIN",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x02F7,
        name: "COS",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x02FC,
        name: "PRINT_WITH_2_NUMBERS",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x02FD,
        name: "PRINT_WITH_2_NUMBERS_NOW",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x02FF,
        name: "PRINT_WITH_3_NUMBERS",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x0300,
        name: "PRINT_WITH_3_NUMBERS_NOW",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x0302,
        name: "PRINT_WITH_4_NUMBERS",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x0303,
        name: "PRINT_WITH_4_NUMBERS_NOW",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x0305,
        name: "PRINT_WITH_5_NUMBERS",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x0306,
        name: "PRINT_WITH_5_NUMBERS_NOW",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x0308,
        name: "PRINT_WITH_6_NUMBERS",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x0309,
        name: "PRINT_WITH_6_NUMBERS_NOW",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x030C,
        name: "PLAYER_MADE_PROGRESS",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x030D,
        name: "SET_PROGRESS_TOTAL",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0317,
        name: "REGISTER_MISSION_GIVEN",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x031A,
        name: "REMOVE_ALL_SCRIPT_FIRES",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x031B,
        name: "IS_FIRST_CAR_COLOUR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x031C,
        name: "IS_SECOND_CAR_COLOUR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x031D,
        name: "HAS_CHAR_BEEN_DAMAGED_BY_WEAPON",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0321,
        name: "EXPLODE_CHAR_HEAD",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0326,
        name: "START_CHAR_FIRE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x032A,
        name: "SET_CAMERA_ZOOM",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x032B,
        name: "CREATE_PICKUP_WITH_AMMO",
        shape: OpcodeShape::Fixed(7),
    },
    OpcodeMeta {
        id: 0x0330,
        name: "SET_PLAYER_NEVER_GETS_TIRED",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0331,
        name: "SET_PLAYER_FAST_RELOAD",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0332,
        name: "SET_CHAR_BLEEDING",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0337,
        name: "SET_CHAR_VISIBLE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x033E,
        name: "DISPLAY_TEXT",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x033F,
        name: "SET_TEXT_SCALE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0340,
        name: "SET_TEXT_COLOUR",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x0341,
        name: "SET_TEXT_JUSTIFY",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0342,
        name: "SET_TEXT_CENTRE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0343,
        name: "SET_TEXT_WRAPX",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0344,
        name: "SET_TEXT_CENTRE_SIZE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0345,
        name: "SET_TEXT_BACKGROUND",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0348,
        name: "SET_TEXT_PROPORTIONAL",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0349,
        name: "SET_TEXT_FONT",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x034F,
        name: "REMOVE_CHAR_ELEGANTLY",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0350,
        name: "SET_CHAR_STAY_IN_SAME_PLACE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0356,
        name: "IS_EXPLOSION_IN_AREA",
        shape: OpcodeShape::Fixed(7),
    },
    OpcodeMeta {
        id: 0x0362,
        name: "WARP_CHAR_FROM_CAR_TO_COORD",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x0363,
        name: "SET_VISIBILITY_OF_CLOSEST_OBJECT_OF_TYPE",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x0364,
        name: "HAS_CHAR_SPOTTED_CHAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x036A,
        name: "WARP_CHAR_INTO_CAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x036D,
        name: "PRINT_WITH_2_NUMBERS_BIG",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x036E,
        name: "PRINT_WITH_3_NUMBERS_BIG",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x036F,
        name: "PRINT_WITH_4_NUMBERS_BIG",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x0370,
        name: "PRINT_WITH_5_NUMBERS_BIG",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x0371,
        name: "PRINT_WITH_6_NUMBERS_BIG",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x0373,
        name: "SET_CAMERA_BEHIND_PLAYER",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x0381,
        name: "SET_OBJECT_VELOCITY",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x0382,
        name: "SET_OBJECT_COLLISION",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x038B,
        name: "LOAD_ALL_MODELS_NOW",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x038D,
        name: "DRAW_SPRITE",
        shape: OpcodeShape::Fixed(9),
    },
    OpcodeMeta {
        id: 0x038E,
        name: "DRAW_RECT",
        shape: OpcodeShape::Fixed(8),
    },
    OpcodeMeta {
        id: 0x038F,
        name: "LOAD_SPRITE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0390,
        name: "LOAD_TEXTURE_DICTIONARY",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0391,
        name: "REMOVE_TEXTURE_DICTIONARY",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x0393,
        name: "SET_CHAR_ANIM_SPEED",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x039E,
        name: "SET_CHAR_CANT_BE_DRAGGED_OUT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x03A3,
        name: "IS_CHAR_MALE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x03A4,
        name: "SCRIPT_NAME",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x03A6,
        name: "FIND_DRUG_PLANE_COORDINATES",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x03B7,
        name: "SWITCH_WORLD_PROCESSING",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x03BF,
        name: "SET_EVERYONE_IGNORE_PLAYER",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x03C8,
        name: "SET_CAMERA_IN_FRONT_OF_PLAYER",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x03CA,
        name: "DOES_OBJECT_EXIST",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x03CB,
        name: "LOAD_SCENE",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x03CC,
        name: "ADD_STUCK_CAR_CHECK",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x03CD,
        name: "REMOVE_STUCK_CAR_CHECK",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x03CE,
        name: "IS_CAR_STUCK",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x03CF,
        name: "LOAD_MISSION_AUDIO",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x03D1,
        name: "PLAY_MISSION_AUDIO",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x03D5,
        name: "CLEAR_THIS_PRINT",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x03D6,
        name: "CLEAR_THIS_BIG_PRINT",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x03DE,
        name: "SET_PED_DENSITY_MULTIPLIER",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x03E0,
        name: "SET_TEXT_DRAW_BEFORE_FADE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x03E3,
        name: "SET_SPRITES_DRAW_BEFORE_FADE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x03E4,
        name: "SET_TEXT_RIGHT_JUSTIFY",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x03E5,
        name: "PRINT_HELP",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x03E6,
        name: "CLEAR_HELP",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x03E7,
        name: "FLASH_HUD_OBJECT",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x03EB,
        name: "CLEAR_SMALL_PRINTS",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x03EE,
        name: "CAN_PLAYER_START_MISSION",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x03EF,
        name: "MAKE_PLAYER_SAFE_FOR_CUTSCENE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x03F0,
        name: "USE_TEXT_COMMANDS",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x03FE,
        name: "SET_CHAR_MONEY",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0417,
        name: "LOAD_AND_LAUNCH_MISSION_INTERNAL",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x042C,
        name: "SET_TOTAL_NUMBER_OF_MISSIONS",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0430,
        name: "WARP_CHAR_INTO_CAR_AS_PASSENGER",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x0433,
        name: "SET_CHAR_IS_CHRIS_CRIMINAL",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0445,
        name: "ARE_ANY_CAR_CHEATS_ACTIVATED",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x0446,
        name: "SET_CHAR_SUFFERS_CRITICAL_HITS",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0448,
        name: "IS_CHAR_SITTING_IN_CAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0449,
        name: "IS_CHAR_SITTING_IN_ANY_CAR",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x044B,
        name: "IS_CHAR_ON_FOOT",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0454,
        name: "GET_DEBUG_CAMERA_COORDINATES",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x0457,
        name: "IS_PLAYER_TARGETTING_CHAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0458,
        name: "IS_PLAYER_TARGETTING_OBJECT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0459,
        name: "TERMINATE_ALL_SCRIPTS_WITH_THIS_NAME",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x045A,
        name: "DISPLAY_TEXT_WITH_NUMBER",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x045B,
        name: "DISPLAY_TEXT_WITH_2_NUMBERS",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x045C,
        name: "FAIL_CURRENT_MISSION",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x0463,
        name: "GET_DEBUG_CAMERA_POINT_AT",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x0464,
        name: "ATTACH_CHAR_TO_CAR",
        shape: OpcodeShape::Fixed(8),
    },
    OpcodeMeta {
        id: 0x0465,
        name: "DETACH_CHAR_FROM_CAR",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0467,
        name: "CLEAR_CHAR_LAST_WEAPON_DAMAGE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x046E,
        name: "GIVE_REMOTE_CONTROLLED_MODEL_TO_PLAYER",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x0470,
        name: "GET_CURRENT_CHAR_WEAPON",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0471,
        name: "LOCATE_CHAR_ANY_MEANS_OBJECT_2D",
        shape: OpcodeShape::Fixed(5),
    },
    OpcodeMeta {
        id: 0x0472,
        name: "LOCATE_CHAR_ON_FOOT_OBJECT_2D",
        shape: OpcodeShape::Fixed(5),
    },
    OpcodeMeta {
        id: 0x0474,
        name: "LOCATE_CHAR_ANY_MEANS_OBJECT_3D",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x0475,
        name: "LOCATE_CHAR_ON_FOOT_OBJECT_3D",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x047A,
        name: "IS_CHAR_ON_ANY_BIKE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0480,
        name: "CAN_CHAR_SEE_DEAD_CHAR",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0488,
        name: "IS_MODEL_AVAILABLE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0489,
        name: "SHUT_CHAR_UP",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x048F,
        name: "REMOVE_ALL_CHAR_WEAPONS",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0491,
        name: "HAS_CHAR_GOT_WEAPON",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x04A5,
        name: "GET_DEAD_CHAR_PICKUP_COORDS",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x04A7,
        name: "IS_CHAR_IN_ANY_BOAT",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x04A9,
        name: "IS_CHAR_IN_ANY_HELI",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x04AB,
        name: "IS_CHAR_IN_ANY_PLANE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x04AD,
        name: "IS_CHAR_IN_WATER",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x04AE,
        name: "SET_VAR_INT_TO_CONSTANT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x04AF,
        name: "SET_LVAR_INT_TO_CONSTANT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x04B0,
        name: "IS_INT_VAR_GREATER_THAN_CONSTANT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x04B1,
        name: "IS_INT_LVAR_GREATER_THAN_CONSTANT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x04B2,
        name: "IS_CONSTANT_GREATER_THAN_INT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x04B3,
        name: "IS_CONSTANT_GREATER_THAN_INT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x04B4,
        name: "IS_INT_VAR_GREATER_OR_EQUAL_TO_CONSTANT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x04B5,
        name: "IS_INT_LVAR_GREATER_OR_EQUAL_TO_CONSTANT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x04B6,
        name: "IS_CONSTANT_GREATER_OR_EQUAL_TO_INT_VAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x04B7,
        name: "IS_CONSTANT_GREATER_OR_EQUAL_TO_INT_LVAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x04B8,
        name: "GET_CHAR_WEAPON_IN_SLOT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x04C4,
        name: "GET_OFFSET_FROM_CHAR_IN_WORLD_COORDS",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x04C5,
        name: "HAS_CHAR_BEEN_PHOTOGRAPHED",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x04C8,
        name: "IS_CHAR_IN_FLYING_VEHICLE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x04CE,
        name: "ADD_SHORT_RANGE_SPRITE_BLIP_FOR_COORD",
        shape: OpcodeShape::Fixed(5),
    },
    OpcodeMeta {
        id: 0x04D1,
        name: "CLEAR_HELI_ORIENTATION",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x04D7,
        name: "FREEZE_CHAR_POSITION",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x04D8,
        name: "SET_CHAR_DROWNS_IN_WATER",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x04DB,
        name: "REMOVE_RC_BUGGY",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x04DD,
        name: "GET_CHAR_ARMOUR",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x04DE,
        name: "SET_CHAR_ARMOUR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x04E3,
        name: "SET_PLAYER_MOOD",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x04E4,
        name: "REQUEST_COLLISION",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x04ED,
        name: "REQUEST_ANIMATION",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x04EE,
        name: "HAS_ANIMATION_LOADED",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x04F0,
        name: "IS_CHAR_WAITING_FOR_WORLD_COLLISION",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x04F4,
        name: "ATTACH_CHAR_TO_OBJECT",
        shape: OpcodeShape::Fixed(8),
    },
    OpcodeMeta {
        id: 0x0500,
        name: "IS_PLAYER_WEARING",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x0501,
        name: "SET_PLAYER_CAN_DO_DRIVE_BY",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0509,
        name: "GET_DISTANCE_BETWEEN_COORDS_2D",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x050A,
        name: "GET_DISTANCE_BETWEEN_COORDS_3D",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x050B,
        name: "POP_CAR_BOOT_USING_PHYSICS",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0512,
        name: "PRINT_HELP_FOREVER",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0513,
        name: "PRINT_HELP_FOREVER_WITH_NUMBER",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x051A,
        name: "HAS_CHAR_BEEN_DAMAGED_BY_CHAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x051B,
        name: "HAS_CHAR_BEEN_DAMAGED_BY_CAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0526,
        name: "SET_CHAR_STAY_IN_CAR_WHEN_JACKED",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x052C,
        name: "SET_PLAYER_DRUNKENNESS",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0547,
        name: "IS_CHAR_TOUCHING_VEHICLE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x054A,
        name: "SET_CHAR_CAN_BE_SHOT_IN_VEHICLE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x054C,
        name: "LOAD_MISSION_TEXT",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x054E,
        name: "CLEAR_CHAR_LAST_DAMAGE_ENTITY",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x055D,
        name: "MAKE_PLAYER_FIRE_PROOF",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x055E,
        name: "INCREASE_PLAYER_MAX_HEALTH",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x055F,
        name: "INCREASE_PLAYER_MAX_ARMOUR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0563,
        name: "ENSURE_PLAYER_HAS_DRIVE_BY_WEAPON",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0568,
        name: "SET_CHAR_NEVER_TARGETTED",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x056C,
        name: "IS_CHAR_IN_ANY_POLICE_VEHICLE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x056D,
        name: "DOES_CHAR_EXIST",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0570,
        name: "ADD_SHORT_RANGE_SPRITE_BLIP_FOR_CONTACT_POINT",
        shape: OpcodeShape::Fixed(5),
    },
    OpcodeMeta {
        id: 0x0572,
        name: "SET_ALL_TAXIS_HAVE_NITRO",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0575,
        name: "FREEZE_CHAR_POSITION_AND_DONT_LOAD_COLLISION",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x057E,
        name: "SET_PLAYER_IS_IN_STADIUM",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0581,
        name: "DISPLAY_RADAR",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0582,
        name: "REGISTER_BEST_POSITION",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0583,
        name: "IS_PLAYER_IN_INFO_ZONE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0588,
        name: "SET_LOAD_COLLISION_FOR_CHAR_FLAG",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x058A,
        name: "ADD_BIG_GUN_FLASH",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x0597,
        name: "IS_CHAR_DUCKING",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x059F,
        name: "GET_OBJECT_VELOCITY",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x05A0,
        name: "IS_DEBUG_CAMERA_ON",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x05A9,
        name: "SET_VAR_TEXT_LABEL",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x05AA,
        name: "SET_LVAR_TEXT_LABEL",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x05AD,
        name: "IS_VAR_TEXT_LABEL_EQUAL_TO_TEXT_LABEL",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x05AE,
        name: "IS_LVAR_TEXT_LABEL_EQUAL_TO_TEXT_LABEL",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x05E2,
        name: "TASK_KILL_CHAR_ON_FOOT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x05F6,
        name: "IS_CHAR_IN_ANGLED_AREA_2D",
        shape: OpcodeShape::Fixed(7),
    },
    OpcodeMeta {
        id: 0x05F8,
        name: "IS_CHAR_IN_ANGLED_AREA_IN_CAR_2D",
        shape: OpcodeShape::Fixed(7),
    },
    OpcodeMeta {
        id: 0x05FC,
        name: "IS_CHAR_IN_ANGLED_AREA_3D",
        shape: OpcodeShape::Fixed(9),
    },
    OpcodeMeta {
        id: 0x05FD,
        name: "IS_CHAR_IN_ANGLED_AREA_ON_FOOT_3D",
        shape: OpcodeShape::Fixed(9),
    },
    OpcodeMeta {
        id: 0x05FE,
        name: "IS_CHAR_IN_ANGLED_AREA_IN_CAR_3D",
        shape: OpcodeShape::Fixed(9),
    },
    OpcodeMeta {
        id: 0x0602,
        name: "IS_CHAR_IN_TAXI",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x060A,
        name: "LOAD_CHAR_DECISION_MAKER",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x060B,
        name: "SET_CHAR_DECISION_MAKER",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x060D,
        name: "SET_TEXT_DROPSHADOW",
        shape: OpcodeShape::Fixed(5),
    },
    OpcodeMeta {
        id: 0x0611,
        name: "IS_CHAR_PLAYING_ANIM",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0612,
        name: "SET_CHAR_ANIM_PLAYING_FLAG",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x0613,
        name: "GET_CHAR_ANIM_CURRENT_TIME",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0614,
        name: "SET_CHAR_ANIM_CURRENT_TIME",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x0619,
        name: "SET_CHAR_COLLISION",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x061A,
        name: "GET_CHAR_ANIM_TOTAL_TIME",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0623,
        name: "INCREMENT_INT_STAT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0624,
        name: "INCREMENT_FLOAT_STAT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0625,
        name: "DECREMENT_INT_STAT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0626,
        name: "DECREMENT_FLOAT_STAT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0627,
        name: "REGISTER_INT_STAT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0628,
        name: "REGISTER_FLOAT_STAT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0629,
        name: "SET_INT_STAT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x062A,
        name: "SET_FLOAT_STAT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0634,
        name: "TASK_KILL_CHAR_ON_FOOT_WHILE_DUCKING",
        shape: OpcodeShape::Fixed(5),
    },
    OpcodeMeta {
        id: 0x0639,
        name: "TASK_TURN_CHAR_TO_FACE_CHAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0642,
        name: "IS_CHAR_AT_SCRIPTED_ATTRACTOR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0643,
        name: "SET_SEQUENCE_TO_REPEAT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x064D,
        name: "PAUSE_FX_SYSTEM",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x0652,
        name: "GET_INT_STAT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0653,
        name: "GET_FLOAT_STAT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0664,
        name: "WRITE_DEBUG_WITH_FLOAT",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0665,
        name: "GET_CHAR_MODEL",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0679,
        name: "ATTACH_CAMERA_TO_VEHICLE",
        shape: OpcodeShape::Fixed(9),
    },
    OpcodeMeta {
        id: 0x067A,
        name: "ATTACH_CAMERA_TO_VEHICLE_LOOK_AT_VEHICLE",
        shape: OpcodeShape::Fixed(7),
    },
    OpcodeMeta {
        id: 0x067B,
        name: "ATTACH_CAMERA_TO_VEHICLE_LOOK_AT_CHAR",
        shape: OpcodeShape::Fixed(7),
    },
    OpcodeMeta {
        id: 0x067C,
        name: "ATTACH_CAMERA_TO_CHAR",
        shape: OpcodeShape::Fixed(9),
    },
    OpcodeMeta {
        id: 0x067E,
        name: "ATTACH_CAMERA_TO_CHAR_LOOK_AT_CHAR",
        shape: OpcodeShape::Fixed(7),
    },
    OpcodeMeta {
        id: 0x068C,
        name: "IS_PLAYER_TARGETTING_ANYTHING",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x068D,
        name: "GET_ACTIVE_CAMERA_COORDINATES",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x068E,
        name: "GET_ACTIVE_CAMERA_POINT_AT",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x06A7,
        name: "ATTACH_CHAR_TO_BIKE",
        shape: OpcodeShape::Fixed(9),
    },
    OpcodeMeta {
        id: 0x06AC,
        name: "GET_CHAR_SPEED",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x06AF,
        name: "DISABLE_PLAYER_SPRINT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x06B2,
        name: "DELETE_SEARCHLIGHT",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x06B7,
        name: "IS_CHAR_IN_SEARCHLIGHT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x06C8,
        name: "SET_LA_RIOTS",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x06C9,
        name: "REMOVE_CHAR_FROM_GROUP",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x06CF,
        name: "DISPLAY_TIMER_BARS",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x06D1,
        name: "SET_VAR_TEXT_LABEL16",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x06D2,
        name: "SET_LVAR_TEXT_LABEL16",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x06DF,
        name: "DELETE_PLAYER",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x06E0,
        name: "SET_TWO_PLAYER_CAMERA_MODE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x06F1,
        name: "LIMIT_TWO_PLAYER_DISTANCE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x06F2,
        name: "RELEASE_TWO_PLAYER_DISTANCE",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x06F3,
        name: "SET_PLAYER_PLAYER_TARGETTING",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x06FA,
        name: "SET_PLAYERS_CAN_BE_IN_SEPARATE_CARS",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x070D,
        name: "BUILD_PLAYER_MODEL",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0717,
        name: "START_SETTING_UP_CONVERSATION",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0719,
        name: "FINISH_SETTING_UP_CONVERSATION",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x071A,
        name: "IS_CONVERSATION_AT_NODE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x072F,
        name: "ADD_STUCK_CAR_CHECK_WITH_WARP",
        shape: OpcodeShape::Fixed(7),
    },
    OpcodeMeta {
        id: 0x0746,
        name: "SET_RELATIONSHIP",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x074B,
        name: "DRAW_SPRITE_WITH_ROTATION",
        shape: OpcodeShape::Fixed(10),
    },
    OpcodeMeta {
        id: 0x0767,
        name: "SET_ZONE_POPULATION_TYPE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x076A,
        name: "SET_ZONE_DEALER_STRENGTH",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x076B,
        name: "GET_ZONE_DEALER_STRENGTH",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x076C,
        name: "SET_ZONE_GANG_STRENGTH",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x076D,
        name: "GET_ZONE_GANG_STRENGTH",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0776,
        name: "REQUEST_IPL",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x077A,
        name: "SET_CHAR_RELATIONSHIP",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x0784,
        name: "GIVE_PLAYER_CLOTHES",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x0792,
        name: "CLEAR_CHAR_TASKS_IMMEDIATELY",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x079D,
        name: "PLAYER_ENTERED_DOCK_CRANE",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x079E,
        name: "PLAYER_ENTERED_BUILDINGSITE_CRANE",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x079F,
        name: "PLAYER_LEFT_CRANE",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x07AF,
        name: "GET_PLAYER_GROUP",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x07B4,
        name: "SET_PLAYER_GROUP_RECRUITMENT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x07CC,
        name: "SET_PLAYER_ENTER_CAR_BUTTON",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x07D3,
        name: "REGISTER_SCRIPT_BRAIN_FOR_CODE_USE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x07EE,
        name: "SET_CAR_ALWAYS_CREATE_SKIDS",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x07F1,
        name: "IS_PLAYER_PERFORMING_WHEELIE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x07F2,
        name: "IS_PLAYER_PERFORMING_STOPPIE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x07F9,
        name: "PLAYER_ENTERED_QUARRY_CRANE",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x07FA,
        name: "PLAYER_ENTERED_LAS_VEGAS_CRANE",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x07FB,
        name: "SWITCH_ENTRY_EXIT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x07FC,
        name: "DISPLAY_TEXT_WITH_FLOAT",
        shape: OpcodeShape::Fixed(5),
    },
    OpcodeMeta {
        id: 0x0801,
        name: "GET_CAMERA_FOV",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0811,
        name: "GET_CAR_CHAR_IS_USING",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0816,
        name: "SET_CHAR_KINDA_STAY_IN_SAME_PLACE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0817,
        name: "TASK_FOLLOW_PATROL_ROUTE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x081C,
        name: "SET_TEXT_EDGE",
        shape: OpcodeShape::Fixed(5),
    },
    OpcodeMeta {
        id: 0x0822,
        name: "SET_FIRST_PERSON_IN_CAR_CAMERA_MODE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x082A,
        name: "SET_PLAYER_DUCK_BUTTON",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0834,
        name: "DO_CAMERA_BUMP",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0836,
        name: "SET_OBJECT_ANIM_SPEED",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x0837,
        name: "IS_OBJECT_PLAYING_ANIM",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0839,
        name: "GET_OBJECT_ANIM_CURRENT_TIME",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x083A,
        name: "SET_OBJECT_ANIM_CURRENT_TIME",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x083C,
        name: "SET_CHAR_VELOCITY",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x083D,
        name: "GET_CHAR_VELOCITY",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x083E,
        name: "SET_CHAR_ROTATION",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x0844,
        name: "IS_VAR_TEXT_LABEL_EMPTY",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0845,
        name: "IS_LVAR_TEXT_LABEL_EMPTY",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0846,
        name: "IS_VAR_TEXT_LABEL16_EMPTY",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0847,
        name: "IS_LVAR_TEXT_LABEL16_EMPTY",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0856,
        name: "SET_CHAR_ALLOWED_TO_DUCK",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0860,
        name: "SET_CHAR_AREA_VISIBLE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0871,
        name: "SWITCH_START",
        shape: OpcodeShape::SwitchTable,
    },
    OpcodeMeta {
        id: 0x0872,
        name: "SWITCH_CONTINUED",
        shape: OpcodeShape::SwitchTable,
    },
    OpcodeMeta {
        id: 0x0874,
        name: "SET_ZONE_POPULATION_RACE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0879,
        name: "SET_GANG_WARS_ACTIVE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x087E,
        name: "SET_CHAR_DROPS_WEAPONS_WHEN_DEAD",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x087F,
        name: "SET_CHAR_NEVER_LEAVES_GROUP",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0881,
        name: "SET_PLAYER_FIRE_BUTTON",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0883,
        name: "ATTACH_FX_SYSTEM_TO_CHAR_BONE",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x0884,
        name: "REGISTER_ATTRACTOR_SCRIPT_BRAIN_FOR_CODE_USE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0889,
        name: "GET_DEAD_CHAR_COORDINATES",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x089B,
        name: "IS_PLAYER_IN_POSITION_FOR_CONVERSATION",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x089C,
        name: "ENABLE_CONVERSATION",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x08A2,
        name: "PLANE_ATTACK_PLAYER_USING_DOG_FIGHT",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x08B3,
        name: "SET_ZONE_FOR_GANG_WARS_TRAINING",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x08C6,
        name: "SET_CHAR_CAN_BE_KNOCKED_OFF_BIKE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x08CA,
        name: "INIT_ZONE_POPULATION_SETTINGS",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x08D2,
        name: "SET_OBJECT_SCALE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x08D3,
        name: "GET_CURRENT_POPULATION_ZONE_TYPE",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x08E6,
        name: "SET_PLANE_UNDERCARRIAGE_UP",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x08E8,
        name: "ATTACH_ANIMS_TO_MODEL",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x08ED,
        name: "CLEAR_CONVERSATION_FOR_CHAR",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x08F9,
        name: "IS_VAR_TEXT_LABEL16_EQUAL_TO_TEXT_LABEL",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x08FA,
        name: "IS_LVAR_TEXT_LABEL16_EQUAL_TO_TEXT_LABEL",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0901,
        name: "SET_PLAYER_JUMP_BUTTON",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x090C,
        name: "SET_SPECIFIC_ZONE_TO_TRIGGER_GANG_WAR",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x090D,
        name: "CLEAR_SPECIFIC_ZONES_TO_TRIGGER_GANG_WAR",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x0913,
        name: "START_NEW_STREAMED_SCRIPT",
        shape: OpcodeShape::VarArgTerminated,
    },
    OpcodeMeta {
        id: 0x0914,
        name: "REGISTER_STREAMED_SCRIPT_INTERNAL",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x091A,
        name: "GET_LATEST_CONSOLE_COMMAND",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x091D,
        name: "SWITCH_ROADS_BACK_TO_ORIGINAL",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x091E,
        name: "SWITCH_PED_ROADS_BACK_TO_ORIGINAL",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x0920,
        name: "CAMERA_SET_VECTOR_TRACK",
        shape: OpcodeShape::Fixed(8),
    },
    OpcodeMeta {
        id: 0x0922,
        name: "CAMERA_SET_LERP_FOV",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x0925,
        name: "CAMERA_RESET_NEW_SCRIPTABLES",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x0928,
        name: "ALLOCATE_STREAMED_SCRIPT_TO_RANDOM_PED",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x0929,
        name: "ALLOCATE_STREAMED_SCRIPT_TO_OBJECT",
        shape: OpcodeShape::Fixed(5),
    },
    OpcodeMeta {
        id: 0x092A,
        name: "SET_PLAYER_CAN_BE_DAMAGED",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x092B,
        name: "GET_GROUP_MEMBER",
        shape: OpcodeShape::Fixed(5),
    },
    OpcodeMeta {
        id: 0x092F,
        name: "CAMERA_PERSIST_TRACK",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0930,
        name: "CAMERA_PERSIST_POS",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0931,
        name: "CAMERA_PERSIST_FOV",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0933,
        name: "CAMERA_IS_VECTOR_MOVE_RUNNING",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x0934,
        name: "CAMERA_IS_VECTOR_TRACK_RUNNING",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x0936,
        name: "CAMERA_SET_VECTOR_MOVE",
        shape: OpcodeShape::Fixed(8),
    },
    OpcodeMeta {
        id: 0x0937,
        name: "DRAW_WINDOW",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x093D,
        name: "SET_CINEMA_CAMERA",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0942,
        name: "HAS_PLAYER_BOUGHT_ITEM",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0944,
        name: "SET_CAMERA_IN_FRONT_OF_CHAR",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0945,
        name: "GET_PLAYER_MAX_ARMOUR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x094E,
        name: "DISABLE_CHAR_SPEECH",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x094F,
        name: "ENABLE_CHAR_SPEECH",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x095E,
        name: "CONTROL_CAR_DOOR",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x095F,
        name: "GET_DOOR_ANGLE_RATIO",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0960,
        name: "SET_PLAYER_DISPLAY_VITAL_STATS_BUTTON",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0997,
        name: "SET_MISSION_RESPECT_TOTAL",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0998,
        name: "AWARD_PLAYER_MISSION_RESPECT",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x099C,
        name: "CAMERA_SET_SHAKE_SIMULATION_SIMPLE",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x09A4,
        name: "SET_UP_CONVERSATION_NODE_WITH_SPEECH",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x09AA,
        name: "SET_UP_CONVERSATION_END_NODE_WITH_SPEECH",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x09AD,
        name: "SET_PLAYER_IN_CAR_CAMERA_MODE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x09B5,
        name: "SET_CHAR_SIGNAL_AFTER_KILL",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x09B6,
        name: "SET_CHAR_WANTED_BY_POLICE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x09B7,
        name: "SET_ZONE_NO_COPS",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x09BA,
        name: "DISPLAY_ZONE_NAMES",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x09BC,
        name: "SET_CHAR_COORDINATES_DONT_WARP_GANG_NO_OFFSET",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x09C1,
        name: "ADD_NEXT_MESSAGE_TO_PREVIOUS_BRIEFS",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x09C5,
        name: "IS_CHAR_USING_MAP_ATTRACTOR",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x09C9,
        name: "REMOVE_CHAR_FROM_CAR_MAINTAIN_POSITION",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x09CC,
        name: "DOES_OBJECT_HAVE_THIS_MODEL",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x09D0,
        name: "IS_VEHICLE_ON_ALL_WHEELS",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x09D5,
        name: "SET_CHAR_SAY_CONTEXT_IMPORTANT",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x09D6,
        name: "SET_CHAR_SAY_SCRIPT",
        shape: OpcodeShape::Fixed(5),
    },
    OpcodeMeta {
        id: 0x09DE,
        name: "IS_CHAR_GETTING_IN_TO_A_CAR",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x09E3,
        name: "FIND_TRAIN_DIRECTION",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x09E8,
        name: "GET_CHAR_AREA_VISIBLE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x09EC,
        name: "ALLOW_FIXED_CAMERA_COLLISION",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x09ED,
        name: "HAS_CHAR_SPOTTED_CHAR_IN_FRONT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x09EF,
        name: "SET_VEHICLE_CAMERA_TWEAK",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x09F0,
        name: "RESET_VEHICLE_CAMERA_TWEAK",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x09F1,
        name: "REPORT_MISSION_AUDIO_EVENT_AT_CHAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x09F4,
        name: "IGNORE_HEIGHT_DIFFERENCE_FOLLOWING_NODES",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x09F5,
        name: "SHUT_ALL_CHARS_UP",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x09F6,
        name: "SET_CHAR_GET_OUT_UPSIDE_DOWN_CAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x09F7,
        name: "REPORT_MISSION_AUDIO_EVENT_AT_CAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x09F8,
        name: "DO_WEAPON_STUFF_AT_START_OF_2P_GAME",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x09FA,
        name: "HAS_GAME_JUST_RETURNED_FROM_FRONTEND",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x09FB,
        name: "GET_CURRENT_LANGUAGE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x09FF,
        name: "SET_RESPAWN_POINT_FOR_DURATION_OF_MISSION",
        shape: OpcodeShape::Fixed(3),
    },
    OpcodeMeta {
        id: 0x0A07,
        name: "SKIP_TO_NEXT_ALLOWED_STATION",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0A09,
        name: "SHUT_CHAR_UP_FOR_SCRIPTED_SPEECH",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0A0A,
        name: "ENABLE_DISABLED_ATTRACTORS_ON_OBJECT",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0A0B,
        name: "LOAD_SCENE_IN_DIRECTION",
        shape: OpcodeShape::Fixed(4),
    },
    OpcodeMeta {
        id: 0x0A0E,
        name: "CLEAR_THIS_PRINT_BIG_NOW",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0A10,
        name: "INCREMENT_INT_STAT_NO_MESSAGE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0A14,
        name: "SET_NO_RESPRAYS",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0A16,
        name: "ATTACH_MISSION_AUDIO_TO_CAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0A17,
        name: "SET_HAS_BEEN_OWNED_FOR_CAR_GENERATOR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0A18,
        name: "SET_UP_CONVERSATION_NODE_WITH_SCRIPTED_SPEECH",
        shape: OpcodeShape::Fixed(6),
    },
    OpcodeMeta {
        id: 0x0A19,
        name: "SET_AREA_NAME",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0A1B,
        name: "IS_CHAR_TOUCHING_CHAR",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0A1C,
        name: "DISABLE_HELI_AUDIO",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0A1E,
        name: "TAKE_PHOTO",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0A1F,
        name: "INCREMENT_FLOAT_STAT_NO_MESSAGE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0A20,
        name: "SET_PLAYER_GROUP_TO_FOLLOW_ALWAYS",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0A24,
        name: "SET_DISABLE_MILITARY_ZONES",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0A25,
        name: "SET_CAMERA_POSITION_UNFIXED",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0A26,
        name: "SET_RADIO_TO_PLAYERS_FAVOURITE_STATION",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x0A27,
        name: "SET_DEATH_WEAPONS_PERSIST",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0A28,
        name: "SET_SWIM_SPEED",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0A29,
        name: "IS_PLAYER_CLIMBING",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0A2B,
        name: "IS_WIDESCREEN_ON_IN_OPTIONS",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x0A2C,
        name: "DRAW_SUBTITLES_BEFORE_FADE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0A2D,
        name: "DRAW_ODDJOB_TITLE_BEFORE_FADE",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0A2F,
        name: "SET_PHOTO_CAMERA_EFFECT",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0A32,
        name: "IS_CHAR_ATTACHED_TO_ANY_CAR",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0A33,
        name: "STORE_CAR_CHAR_IS_ATTACHED_TO_NO_SAVE",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0A39,
        name: "GET_PLAYER_IN_CAR_CAMERA_MODE",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x0A3C,
        name: "SET_UP_CONVERSATION_END_NODE_WITH_SCRIPTED_SPEECH",
        shape: OpcodeShape::Fixed(2),
    },
    OpcodeMeta {
        id: 0x0A3D,
        name: "ACTIVATE_PIMP_CHEAT",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0A43,
        name: "GET_RID_OF_PLAYER_PROSTITUTE",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x0A44,
        name: "DISPLAY_NON_MINIGAME_HELP_MESSAGES",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0A45,
        name: "SET_RAILTRACK_RESISTANCE_MULT",
        shape: OpcodeShape::Fixed(1),
    },
    OpcodeMeta {
        id: 0x0A47,
        name: "FINISH_SETTING_UP_CONVERSATION_NO_SUBTITLES",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x0A49,
        name: "IS_XBOX_VERSION",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x0A4B,
        name: "IS_PC_USING_JOYPAD",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x0A4C,
        name: "IS_MOUSE_USING_VERTICAL_INVERSION",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x0A4D,
        name: "IS_JAPANESE_VERSION",
        shape: OpcodeShape::Fixed(0),
    },
    OpcodeMeta {
        id: 0x0A4E,
        name: "DO_DEBUG_STUFF",
        shape: OpcodeShape::Fixed(0),
    },
];

pub fn lookup_opcode_meta(opcode: u16) -> Option<OpcodeMeta> {
    SA_OPCODE_TABLE
        .binary_search_by_key(&opcode, |m| m.id)
        .ok()
        .map(|idx| SA_OPCODE_TABLE[idx])
}
