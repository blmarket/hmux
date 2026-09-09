use crate::cfg::{cfg_files, cfg_quiet};
use crate::client::client_main;
use crate::compat::BSDgetopt;
use crate::compat::getprogname;
use crate::compat::getptmfd;
use crate::compat::{BSDoptarg, BSDoptind};
use crate::compat::{cstr_eq_ignore_case, error_message};
use crate::environ::EnvironmentStore;
use crate::environ::{
    process_environment, process_environment_value, reset_global_environment,
    with_global_environment, with_global_environment_mut,
};
use crate::ffi::{access, err, errx, fcntl, getcwd, getuid, nl_langinfo, setlocale, tzset};
use crate::fmt_args;
use crate::log::{log_add_level, log_debug};
use crate::options::{OptionsEngine, RustOptionsEngine};
use crate::{UserAccount, UserAccountRecord};

use crate::osdep_linux::osdep_event_init;
use crate::terminfo::{RustTerminalFeatureSet, TerminalFeatureSet};
use crate::text::{RustUtf8VisModel, Utf8VisModel};
pub use crate::types::*;
use crate::xmalloc::xasprintf;
use ::std::ffi::{CStr, CString, OsStr};
use ::std::fs::{self, DirBuilder};
use ::std::io::{ErrorKind, Write};
use ::std::os::unix::ffi::{OsStrExt, OsStringExt};
use ::std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use ::std::sync::{LazyLock, OnceLock};
use ::std::time::Instant;
pub type nl_item_value = core::ffi::c_uint;
pub const _NL_NUM: nl_item_value = 786449;
pub const _NL_NUM_LC_IDENTIFICATION: nl_item_value = 786448;
pub const _NL_IDENTIFICATION_CODESET: nl_item_value = 786447;
pub const _NL_IDENTIFICATION_CATEGORY: nl_item_value = 786446;
pub const _NL_IDENTIFICATION_DATE: nl_item_value = 786445;
pub const _NL_IDENTIFICATION_REVISION: nl_item_value = 786444;
pub const _NL_IDENTIFICATION_ABBREVIATION: nl_item_value = 786443;
pub const _NL_IDENTIFICATION_APPLICATION: nl_item_value = 786442;
pub const _NL_IDENTIFICATION_AUDIENCE: nl_item_value = 786441;
pub const _NL_IDENTIFICATION_TERRITORY: nl_item_value = 786440;
pub const _NL_IDENTIFICATION_LANGUAGE: nl_item_value = 786439;
pub const _NL_IDENTIFICATION_FAX: nl_item_value = 786438;
pub const _NL_IDENTIFICATION_TEL: nl_item_value = 786437;
pub const _NL_IDENTIFICATION_EMAIL: nl_item_value = 786436;
pub const _NL_IDENTIFICATION_CONTACT: nl_item_value = 786435;
pub const _NL_IDENTIFICATION_ADDRESS: nl_item_value = 786434;
pub const _NL_IDENTIFICATION_SOURCE: nl_item_value = 786433;
pub const _NL_IDENTIFICATION_TITLE: nl_item_value = 786432;
pub const _NL_NUM_LC_MEASUREMENT: nl_item_value = 720898;
pub const _NL_MEASUREMENT_CODESET: nl_item_value = 720897;
pub const _NL_MEASUREMENT_MEASUREMENT: nl_item_value = 720896;
pub const _NL_NUM_LC_TELEPHONE: nl_item_value = 655365;
pub const _NL_TELEPHONE_CODESET: nl_item_value = 655364;
pub const _NL_TELEPHONE_INT_PREFIX: nl_item_value = 655363;
pub const _NL_TELEPHONE_INT_SELECT: nl_item_value = 655362;
pub const _NL_TELEPHONE_TEL_DOM_FMT: nl_item_value = 655361;
pub const _NL_TELEPHONE_TEL_INT_FMT: nl_item_value = 655360;
pub const _NL_NUM_LC_ADDRESS: nl_item_value = 589837;
pub const _NL_ADDRESS_CODESET: nl_item_value = 589836;
pub const _NL_ADDRESS_LANG_LIB: nl_item_value = 589835;
pub const _NL_ADDRESS_LANG_TERM: nl_item_value = 589834;
pub const _NL_ADDRESS_LANG_AB: nl_item_value = 589833;
pub const _NL_ADDRESS_LANG_NAME: nl_item_value = 589832;
pub const _NL_ADDRESS_COUNTRY_ISBN: nl_item_value = 589831;
pub const _NL_ADDRESS_COUNTRY_NUM: nl_item_value = 589830;
pub const _NL_ADDRESS_COUNTRY_CAR: nl_item_value = 589829;
pub const _NL_ADDRESS_COUNTRY_AB3: nl_item_value = 589828;
pub const _NL_ADDRESS_COUNTRY_AB2: nl_item_value = 589827;
pub const _NL_ADDRESS_COUNTRY_POST: nl_item_value = 589826;
pub const _NL_ADDRESS_COUNTRY_NAME: nl_item_value = 589825;
pub const _NL_ADDRESS_POSTAL_FMT: nl_item_value = 589824;
pub const _NL_NUM_LC_NAME: nl_item_value = 524295;
pub const _NL_NAME_CODESET: nl_item_value = 524294;
pub const _NL_NAME_NAME_MS: nl_item_value = 524293;
pub const _NL_NAME_NAME_MISS: nl_item_value = 524292;
pub const _NL_NAME_NAME_MRS: nl_item_value = 524291;
pub const _NL_NAME_NAME_MR: nl_item_value = 524290;
pub const _NL_NAME_NAME_GEN: nl_item_value = 524289;
pub const _NL_NAME_NAME_FMT: nl_item_value = 524288;
pub const _NL_NUM_LC_PAPER: nl_item_value = 458755;
pub const _NL_PAPER_CODESET: nl_item_value = 458754;
pub const _NL_PAPER_WIDTH: nl_item_value = 458753;
pub const _NL_PAPER_HEIGHT: nl_item_value = 458752;
pub const _NL_NUM_LC_MESSAGES: nl_item_value = 327685;
pub const _NL_MESSAGES_CODESET: nl_item_value = 327684;
pub const __NOSTR: nl_item_value = 327683;
pub const __YESSTR: nl_item_value = 327682;
pub const __NOEXPR: nl_item_value = 327681;
pub const __YESEXPR: nl_item_value = 327680;
pub const _NL_NUM_LC_NUMERIC: nl_item_value = 65542;
pub const _NL_NUMERIC_CODESET: nl_item_value = 65541;
pub const _NL_NUMERIC_THOUSANDS_SEP_WC: nl_item_value = 65540;
pub const _NL_NUMERIC_DECIMAL_POINT_WC: nl_item_value = 65539;
pub const __GROUPING: nl_item_value = 65538;
pub const THOUSEP: nl_item_value = 65537;
pub const __THOUSANDS_SEP: nl_item_value = 65537;
pub const RADIXCHAR: nl_item_value = 65536;
pub const __DECIMAL_POINT: nl_item_value = 65536;
pub const _NL_NUM_LC_MONETARY: nl_item_value = 262190;
pub const _NL_MONETARY_CODESET: nl_item_value = 262189;
pub const _NL_MONETARY_THOUSANDS_SEP_WC: nl_item_value = 262188;
pub const _NL_MONETARY_DECIMAL_POINT_WC: nl_item_value = 262187;
pub const _NL_MONETARY_CONVERSION_RATE: nl_item_value = 262186;
pub const _NL_MONETARY_DUO_VALID_TO: nl_item_value = 262185;
pub const _NL_MONETARY_DUO_VALID_FROM: nl_item_value = 262184;
pub const _NL_MONETARY_UNO_VALID_TO: nl_item_value = 262183;
pub const _NL_MONETARY_UNO_VALID_FROM: nl_item_value = 262182;
pub const _NL_MONETARY_DUO_INT_N_SIGN_POSN: nl_item_value = 262181;
pub const _NL_MONETARY_DUO_INT_P_SIGN_POSN: nl_item_value = 262180;
pub const _NL_MONETARY_DUO_N_SIGN_POSN: nl_item_value = 262179;
pub const _NL_MONETARY_DUO_P_SIGN_POSN: nl_item_value = 262178;
pub const _NL_MONETARY_DUO_INT_N_SEP_BY_SPACE: nl_item_value = 262177;
pub const _NL_MONETARY_DUO_INT_N_CS_PRECEDES: nl_item_value = 262176;
pub const _NL_MONETARY_DUO_INT_P_SEP_BY_SPACE: nl_item_value = 262175;
pub const _NL_MONETARY_DUO_INT_P_CS_PRECEDES: nl_item_value = 262174;
pub const _NL_MONETARY_DUO_N_SEP_BY_SPACE: nl_item_value = 262173;
pub const _NL_MONETARY_DUO_N_CS_PRECEDES: nl_item_value = 262172;
pub const _NL_MONETARY_DUO_P_SEP_BY_SPACE: nl_item_value = 262171;
pub const _NL_MONETARY_DUO_P_CS_PRECEDES: nl_item_value = 262170;
pub const _NL_MONETARY_DUO_FRAC_DIGITS: nl_item_value = 262169;
pub const _NL_MONETARY_DUO_INT_FRAC_DIGITS: nl_item_value = 262168;
pub const _NL_MONETARY_DUO_CURRENCY_SYMBOL: nl_item_value = 262167;
pub const _NL_MONETARY_DUO_INT_CURR_SYMBOL: nl_item_value = 262166;
pub const __INT_N_SIGN_POSN: nl_item_value = 262165;
pub const __INT_P_SIGN_POSN: nl_item_value = 262164;
pub const __INT_N_SEP_BY_SPACE: nl_item_value = 262163;
pub const __INT_N_CS_PRECEDES: nl_item_value = 262162;
pub const __INT_P_SEP_BY_SPACE: nl_item_value = 262161;
pub const __INT_P_CS_PRECEDES: nl_item_value = 262160;
pub const _NL_MONETARY_CRNCYSTR: nl_item_value = 262159;
pub const __N_SIGN_POSN: nl_item_value = 262158;
pub const __P_SIGN_POSN: nl_item_value = 262157;
pub const __N_SEP_BY_SPACE: nl_item_value = 262156;
pub const __N_CS_PRECEDES: nl_item_value = 262155;
pub const __P_SEP_BY_SPACE: nl_item_value = 262154;
pub const __P_CS_PRECEDES: nl_item_value = 262153;
pub const __FRAC_DIGITS: nl_item_value = 262152;
pub const __INT_FRAC_DIGITS: nl_item_value = 262151;
pub const __NEGATIVE_SIGN: nl_item_value = 262150;
pub const __POSITIVE_SIGN: nl_item_value = 262149;
pub const __MON_GROUPING: nl_item_value = 262148;
pub const __MON_THOUSANDS_SEP: nl_item_value = 262147;
pub const __MON_DECIMAL_POINT: nl_item_value = 262146;
pub const __CURRENCY_SYMBOL: nl_item_value = 262145;
pub const __INT_CURR_SYMBOL: nl_item_value = 262144;
pub const _NL_NUM_LC_CTYPE: nl_item_value = 86;
pub const _NL_CTYPE_EXTRA_MAP_14: nl_item_value = 85;
pub const _NL_CTYPE_EXTRA_MAP_13: nl_item_value = 84;
pub const _NL_CTYPE_EXTRA_MAP_12: nl_item_value = 83;
pub const _NL_CTYPE_EXTRA_MAP_11: nl_item_value = 82;
pub const _NL_CTYPE_EXTRA_MAP_10: nl_item_value = 81;
pub const _NL_CTYPE_EXTRA_MAP_9: nl_item_value = 80;
pub const _NL_CTYPE_EXTRA_MAP_8: nl_item_value = 79;
pub const _NL_CTYPE_EXTRA_MAP_7: nl_item_value = 78;
pub const _NL_CTYPE_EXTRA_MAP_6: nl_item_value = 77;
pub const _NL_CTYPE_EXTRA_MAP_5: nl_item_value = 76;
pub const _NL_CTYPE_EXTRA_MAP_4: nl_item_value = 75;
pub const _NL_CTYPE_EXTRA_MAP_3: nl_item_value = 74;
pub const _NL_CTYPE_EXTRA_MAP_2: nl_item_value = 73;
pub const _NL_CTYPE_EXTRA_MAP_1: nl_item_value = 72;
pub const _NL_CTYPE_NONASCII_CASE: nl_item_value = 71;
pub const _NL_CTYPE_MAP_TO_NONASCII: nl_item_value = 70;
pub const _NL_CTYPE_TRANSLIT_IGNORE: nl_item_value = 69;
pub const _NL_CTYPE_TRANSLIT_IGNORE_LEN: nl_item_value = 68;
pub const _NL_CTYPE_TRANSLIT_DEFAULT_MISSING: nl_item_value = 67;
pub const _NL_CTYPE_TRANSLIT_DEFAULT_MISSING_LEN: nl_item_value = 66;
pub const _NL_CTYPE_TRANSLIT_TO_TBL: nl_item_value = 65;
pub const _NL_CTYPE_TRANSLIT_TO_IDX: nl_item_value = 64;
pub const _NL_CTYPE_TRANSLIT_FROM_TBL: nl_item_value = 63;
pub const _NL_CTYPE_TRANSLIT_FROM_IDX: nl_item_value = 62;
pub const _NL_CTYPE_TRANSLIT_TAB_SIZE: nl_item_value = 61;
pub const _NL_CTYPE_OUTDIGIT9_WC: nl_item_value = 60;
pub const _NL_CTYPE_OUTDIGIT8_WC: nl_item_value = 59;
pub const _NL_CTYPE_OUTDIGIT7_WC: nl_item_value = 58;
pub const _NL_CTYPE_OUTDIGIT6_WC: nl_item_value = 57;
pub const _NL_CTYPE_OUTDIGIT5_WC: nl_item_value = 56;
pub const _NL_CTYPE_OUTDIGIT4_WC: nl_item_value = 55;
pub const _NL_CTYPE_OUTDIGIT3_WC: nl_item_value = 54;
pub const _NL_CTYPE_OUTDIGIT2_WC: nl_item_value = 53;
pub const _NL_CTYPE_OUTDIGIT1_WC: nl_item_value = 52;
pub const _NL_CTYPE_OUTDIGIT0_WC: nl_item_value = 51;
pub const _NL_CTYPE_OUTDIGIT9_MB: nl_item_value = 50;
pub const _NL_CTYPE_OUTDIGIT8_MB: nl_item_value = 49;
pub const _NL_CTYPE_OUTDIGIT7_MB: nl_item_value = 48;
pub const _NL_CTYPE_OUTDIGIT6_MB: nl_item_value = 47;
pub const _NL_CTYPE_OUTDIGIT5_MB: nl_item_value = 46;
pub const _NL_CTYPE_OUTDIGIT4_MB: nl_item_value = 45;
pub const _NL_CTYPE_OUTDIGIT3_MB: nl_item_value = 44;
pub const _NL_CTYPE_OUTDIGIT2_MB: nl_item_value = 43;
pub const _NL_CTYPE_OUTDIGIT1_MB: nl_item_value = 42;
pub const _NL_CTYPE_OUTDIGIT0_MB: nl_item_value = 41;
pub const _NL_CTYPE_INDIGITS9_WC: nl_item_value = 40;
pub const _NL_CTYPE_INDIGITS8_WC: nl_item_value = 39;
pub const _NL_CTYPE_INDIGITS7_WC: nl_item_value = 38;
pub const _NL_CTYPE_INDIGITS6_WC: nl_item_value = 37;
pub const _NL_CTYPE_INDIGITS5_WC: nl_item_value = 36;
pub const _NL_CTYPE_INDIGITS4_WC: nl_item_value = 35;
pub const _NL_CTYPE_INDIGITS3_WC: nl_item_value = 34;
pub const _NL_CTYPE_INDIGITS2_WC: nl_item_value = 33;
pub const _NL_CTYPE_INDIGITS1_WC: nl_item_value = 32;
pub const _NL_CTYPE_INDIGITS0_WC: nl_item_value = 31;
pub const _NL_CTYPE_INDIGITS_WC_LEN: nl_item_value = 30;
pub const _NL_CTYPE_INDIGITS9_MB: nl_item_value = 29;
pub const _NL_CTYPE_INDIGITS8_MB: nl_item_value = 28;
pub const _NL_CTYPE_INDIGITS7_MB: nl_item_value = 27;
pub const _NL_CTYPE_INDIGITS6_MB: nl_item_value = 26;
pub const _NL_CTYPE_INDIGITS5_MB: nl_item_value = 25;
pub const _NL_CTYPE_INDIGITS4_MB: nl_item_value = 24;
pub const _NL_CTYPE_INDIGITS3_MB: nl_item_value = 23;
pub const _NL_CTYPE_INDIGITS2_MB: nl_item_value = 22;
pub const _NL_CTYPE_INDIGITS1_MB: nl_item_value = 21;
pub const _NL_CTYPE_INDIGITS0_MB: nl_item_value = 20;
pub const _NL_CTYPE_INDIGITS_MB_LEN: nl_item_value = 19;
pub const _NL_CTYPE_MAP_OFFSET: nl_item_value = 18;
pub const _NL_CTYPE_CLASS_OFFSET: nl_item_value = 17;
pub const _NL_CTYPE_TOLOWER32: nl_item_value = 16;
pub const _NL_CTYPE_TOUPPER32: nl_item_value = 15;
pub const CODESET: nl_item_value = 14;
pub const _NL_CTYPE_CODESET_NAME: nl_item_value = 14;
pub const _NL_CTYPE_MB_CUR_MAX: nl_item_value = 13;
pub const _NL_CTYPE_WIDTH: nl_item_value = 12;
pub const _NL_CTYPE_MAP_NAMES: nl_item_value = 11;
pub const _NL_CTYPE_CLASS_NAMES: nl_item_value = 10;
pub const _NL_CTYPE_GAP6: nl_item_value = 9;
pub const _NL_CTYPE_GAP5: nl_item_value = 8;
pub const _NL_CTYPE_GAP4: nl_item_value = 7;
pub const _NL_CTYPE_GAP3: nl_item_value = 6;
pub const _NL_CTYPE_CLASS32: nl_item_value = 5;
pub const _NL_CTYPE_GAP2: nl_item_value = 4;
pub const _NL_CTYPE_TOLOWER: nl_item_value = 3;
pub const _NL_CTYPE_GAP1: nl_item_value = 2;
pub const _NL_CTYPE_TOUPPER: nl_item_value = 1;
pub const _NL_CTYPE_CLASS: nl_item_value = 0;
pub const _NL_NUM_LC_COLLATE: nl_item_value = 196627;
pub const _NL_COLLATE_CODESET: nl_item_value = 196626;
pub const _NL_COLLATE_COLLSEQWC: nl_item_value = 196625;
pub const _NL_COLLATE_COLLSEQMB: nl_item_value = 196624;
pub const _NL_COLLATE_SYMB_EXTRAMB: nl_item_value = 196623;
pub const _NL_COLLATE_SYMB_TABLEMB: nl_item_value = 196622;
pub const _NL_COLLATE_SYMB_HASH_SIZEMB: nl_item_value = 196621;
pub const _NL_COLLATE_INDIRECTWC: nl_item_value = 196620;
pub const _NL_COLLATE_EXTRAWC: nl_item_value = 196619;
pub const _NL_COLLATE_WEIGHTWC: nl_item_value = 196618;
pub const _NL_COLLATE_TABLEWC: nl_item_value = 196617;
pub const _NL_COLLATE_GAP3: nl_item_value = 196616;
pub const _NL_COLLATE_GAP2: nl_item_value = 196615;
pub const _NL_COLLATE_GAP1: nl_item_value = 196614;
pub const _NL_COLLATE_INDIRECTMB: nl_item_value = 196613;
pub const _NL_COLLATE_EXTRAMB: nl_item_value = 196612;
pub const _NL_COLLATE_WEIGHTMB: nl_item_value = 196611;
pub const _NL_COLLATE_TABLEMB: nl_item_value = 196610;
pub const _NL_COLLATE_RULESETS: nl_item_value = 196609;
pub const _NL_COLLATE_NRULES: nl_item_value = 196608;
pub const _NL_NUM_LC_TIME: nl_item_value = 131231;
pub const _NL_WABALTMON_12: nl_item_value = 131230;
pub const _NL_WABALTMON_11: nl_item_value = 131229;
pub const _NL_WABALTMON_10: nl_item_value = 131228;
pub const _NL_WABALTMON_9: nl_item_value = 131227;
pub const _NL_WABALTMON_8: nl_item_value = 131226;
pub const _NL_WABALTMON_7: nl_item_value = 131225;
pub const _NL_WABALTMON_6: nl_item_value = 131224;
pub const _NL_WABALTMON_5: nl_item_value = 131223;
pub const _NL_WABALTMON_4: nl_item_value = 131222;
pub const _NL_WABALTMON_3: nl_item_value = 131221;
pub const _NL_WABALTMON_2: nl_item_value = 131220;
pub const _NL_WABALTMON_1: nl_item_value = 131219;
pub const _NL_ABALTMON_12: nl_item_value = 131218;
pub const _NL_ABALTMON_11: nl_item_value = 131217;
pub const _NL_ABALTMON_10: nl_item_value = 131216;
pub const _NL_ABALTMON_9: nl_item_value = 131215;
pub const _NL_ABALTMON_8: nl_item_value = 131214;
pub const _NL_ABALTMON_7: nl_item_value = 131213;
pub const _NL_ABALTMON_6: nl_item_value = 131212;
pub const _NL_ABALTMON_5: nl_item_value = 131211;
pub const _NL_ABALTMON_4: nl_item_value = 131210;
pub const _NL_ABALTMON_3: nl_item_value = 131209;
pub const _NL_ABALTMON_2: nl_item_value = 131208;
pub const _NL_ABALTMON_1: nl_item_value = 131207;
pub const _NL_WALTMON_12: nl_item_value = 131206;
pub const _NL_WALTMON_11: nl_item_value = 131205;
pub const _NL_WALTMON_10: nl_item_value = 131204;
pub const _NL_WALTMON_9: nl_item_value = 131203;
pub const _NL_WALTMON_8: nl_item_value = 131202;
pub const _NL_WALTMON_7: nl_item_value = 131201;
pub const _NL_WALTMON_6: nl_item_value = 131200;
pub const _NL_WALTMON_5: nl_item_value = 131199;
pub const _NL_WALTMON_4: nl_item_value = 131198;
pub const _NL_WALTMON_3: nl_item_value = 131197;
pub const _NL_WALTMON_2: nl_item_value = 131196;
pub const _NL_WALTMON_1: nl_item_value = 131195;
pub const __ALTMON_12: nl_item_value = 131194;
pub const __ALTMON_11: nl_item_value = 131193;
pub const __ALTMON_10: nl_item_value = 131192;
pub const __ALTMON_9: nl_item_value = 131191;
pub const __ALTMON_8: nl_item_value = 131190;
pub const __ALTMON_7: nl_item_value = 131189;
pub const __ALTMON_6: nl_item_value = 131188;
pub const __ALTMON_5: nl_item_value = 131187;
pub const __ALTMON_4: nl_item_value = 131186;
pub const __ALTMON_3: nl_item_value = 131185;
pub const __ALTMON_2: nl_item_value = 131184;
pub const __ALTMON_1: nl_item_value = 131183;
pub const _NL_TIME_CODESET: nl_item_value = 131182;
pub const _NL_W_DATE_FMT: nl_item_value = 131181;
pub const _DATE_FMT: nl_item_value = 131180;
pub const _NL_TIME_TIMEZONE: nl_item_value = 131179;
pub const _NL_TIME_CAL_DIRECTION: nl_item_value = 131178;
pub const _NL_TIME_FIRST_WORKDAY: nl_item_value = 131177;
pub const _NL_TIME_FIRST_WEEKDAY: nl_item_value = 131176;
pub const _NL_TIME_WEEK_1STWEEK: nl_item_value = 131175;
pub const _NL_TIME_WEEK_1STDAY: nl_item_value = 131174;
pub const _NL_TIME_WEEK_NDAYS: nl_item_value = 131173;
pub const _NL_WERA_T_FMT: nl_item_value = 131172;
pub const _NL_WERA_D_T_FMT: nl_item_value = 131171;
pub const _NL_WALT_DIGITS: nl_item_value = 131170;
pub const _NL_WERA_D_FMT: nl_item_value = 131169;
pub const _NL_WERA_YEAR: nl_item_value = 131168;
pub const _NL_WT_FMT_AMPM: nl_item_value = 131167;
pub const _NL_WT_FMT: nl_item_value = 131166;
pub const _NL_WD_FMT: nl_item_value = 131165;
pub const _NL_WD_T_FMT: nl_item_value = 131164;
pub const _NL_WPM_STR: nl_item_value = 131163;
pub const _NL_WAM_STR: nl_item_value = 131162;
pub const _NL_WMON_12: nl_item_value = 131161;
pub const _NL_WMON_11: nl_item_value = 131160;
pub const _NL_WMON_10: nl_item_value = 131159;
pub const _NL_WMON_9: nl_item_value = 131158;
pub const _NL_WMON_8: nl_item_value = 131157;
pub const _NL_WMON_7: nl_item_value = 131156;
pub const _NL_WMON_6: nl_item_value = 131155;
pub const _NL_WMON_5: nl_item_value = 131154;
pub const _NL_WMON_4: nl_item_value = 131153;
pub const _NL_WMON_3: nl_item_value = 131152;
pub const _NL_WMON_2: nl_item_value = 131151;
pub const _NL_WMON_1: nl_item_value = 131150;
pub const _NL_WABMON_12: nl_item_value = 131149;
pub const _NL_WABMON_11: nl_item_value = 131148;
pub const _NL_WABMON_10: nl_item_value = 131147;
pub const _NL_WABMON_9: nl_item_value = 131146;
pub const _NL_WABMON_8: nl_item_value = 131145;
pub const _NL_WABMON_7: nl_item_value = 131144;
pub const _NL_WABMON_6: nl_item_value = 131143;
pub const _NL_WABMON_5: nl_item_value = 131142;
pub const _NL_WABMON_4: nl_item_value = 131141;
pub const _NL_WABMON_3: nl_item_value = 131140;
pub const _NL_WABMON_2: nl_item_value = 131139;
pub const _NL_WABMON_1: nl_item_value = 131138;
pub const _NL_WDAY_7: nl_item_value = 131137;
pub const _NL_WDAY_6: nl_item_value = 131136;
pub const _NL_WDAY_5: nl_item_value = 131135;
pub const _NL_WDAY_4: nl_item_value = 131134;
pub const _NL_WDAY_3: nl_item_value = 131133;
pub const _NL_WDAY_2: nl_item_value = 131132;
pub const _NL_WDAY_1: nl_item_value = 131131;
pub const _NL_WABDAY_7: nl_item_value = 131130;
pub const _NL_WABDAY_6: nl_item_value = 131129;
pub const _NL_WABDAY_5: nl_item_value = 131128;
pub const _NL_WABDAY_4: nl_item_value = 131127;
pub const _NL_WABDAY_3: nl_item_value = 131126;
pub const _NL_WABDAY_2: nl_item_value = 131125;
pub const _NL_WABDAY_1: nl_item_value = 131124;
pub const _NL_TIME_ERA_ENTRIES: nl_item_value = 131123;
pub const _NL_TIME_ERA_NUM_ENTRIES: nl_item_value = 131122;
pub const ERA_T_FMT: nl_item_value = 131121;
pub const ERA_D_T_FMT: nl_item_value = 131120;
pub const ALT_DIGITS: nl_item_value = 131119;
pub const ERA_D_FMT: nl_item_value = 131118;
pub const __ERA_YEAR: nl_item_value = 131117;
pub const ERA: nl_item_value = 131116;
pub const T_FMT_AMPM: nl_item_value = 131115;
pub const T_FMT: nl_item_value = 131114;
pub const D_FMT: nl_item_value = 131113;
pub const D_T_FMT: nl_item_value = 131112;
pub const PM_STR: nl_item_value = 131111;
pub const AM_STR: nl_item_value = 131110;
pub const MON_12: nl_item_value = 131109;
pub const MON_11: nl_item_value = 131108;
pub const MON_10: nl_item_value = 131107;
pub const MON_9: nl_item_value = 131106;
pub const MON_8: nl_item_value = 131105;
pub const MON_7: nl_item_value = 131104;
pub const MON_6: nl_item_value = 131103;
pub const MON_5: nl_item_value = 131102;
pub const MON_4: nl_item_value = 131101;
pub const MON_3: nl_item_value = 131100;
pub const MON_2: nl_item_value = 131099;
pub const MON_1: nl_item_value = 131098;
pub const ABMON_12: nl_item_value = 131097;
pub const ABMON_11: nl_item_value = 131096;
pub const ABMON_10: nl_item_value = 131095;
pub const ABMON_9: nl_item_value = 131094;
pub const ABMON_8: nl_item_value = 131093;
pub const ABMON_7: nl_item_value = 131092;
pub const ABMON_6: nl_item_value = 131091;
pub const ABMON_5: nl_item_value = 131090;
pub const ABMON_4: nl_item_value = 131089;
pub const ABMON_3: nl_item_value = 131088;
pub const ABMON_2: nl_item_value = 131087;
pub const ABMON_1: nl_item_value = 131086;
pub const DAY_7: nl_item_value = 131085;
pub const DAY_6: nl_item_value = 131084;
pub const DAY_5: nl_item_value = 131083;
pub const DAY_4: nl_item_value = 131082;
pub const DAY_3: nl_item_value = 131081;
pub const DAY_2: nl_item_value = 131080;
pub const DAY_1: nl_item_value = 131079;
pub const ABDAY_7: nl_item_value = 131078;
pub const ABDAY_6: nl_item_value = 131077;
pub const ABDAY_5: nl_item_value = 131076;
pub const ABDAY_4: nl_item_value = 131075;
pub const ABDAY_3: nl_item_value = 131074;
pub const ABDAY_2: nl_item_value = 131073;
pub const ABDAY_1: nl_item_value = 131072;
pub use crate::consts::{
    __S_IEXEC, __S_IREAD, __S_IWRITE, _PATH_BSHELL, CLIENT_CONTROL, CLIENT_CONTROLCONTROL,
    CLIENT_DEFAULTSOCKET, CLIENT_LOGIN, CLIENT_NOFORK, CLIENT_NOSTARTSERVER, CLIENT_UTF8,
    MODEKEY_EMACS, MODEKEY_VI, O_NONBLOCK, OPTIONS_TABLE_CHOICE, OPTIONS_TABLE_COLOUR,
    OPTIONS_TABLE_COMMAND, OPTIONS_TABLE_FLAG, OPTIONS_TABLE_KEY, OPTIONS_TABLE_NUMBER,
    OPTIONS_TABLE_SERVER, OPTIONS_TABLE_SESSION, OPTIONS_TABLE_STRING, OPTIONS_TABLE_WINDOW,
    S_IRWXU, VIS_CSTYLE, VIS_NL, VIS_OCTAL, VIS_TAB, X_OK,
};

pub const __LC_CTYPE: core::ffi::c_int = 0 as core::ffi::c_int;
pub const __LC_TIME: core::ffi::c_int = 2 as core::ffi::c_int;

pub const F_GETFL: core::ffi::c_int = 3 as core::ffi::c_int;
pub const F_SETFL: core::ffi::c_int = 4 as core::ffi::c_int;

pub const LC_CTYPE: core::ffi::c_int = __LC_CTYPE;
pub const LC_TIME: core::ffi::c_int = __LC_TIME;

pub const TMUX_SOCK_PERM: core::ffi::c_int = 7 as core::ffi::c_int;

/// Initializes the server's owned option handles and starting environment.
pub unsafe fn global_options_create() {
    unsafe {
        global_options = Some(RustOptionsEngine.create(None));
        global_s_options = Some(RustOptionsEngine.create(None));
        global_w_options = Some(RustOptionsEngine.create(None));

        reset_global_environment();
    }
}

/// Releases the server's option handles and starting environment.
pub unsafe fn global_options_free() {
    unsafe {
        global_options = None;
        global_s_options = None;
        global_w_options = None;
        reset_global_environment();
    }
}

pub static mut global_options: Option<RustOptionsRef> = None;
pub static mut global_s_options: Option<RustOptionsRef> = None;
pub static mut global_w_options: Option<RustOptionsRef> = None;
pub static mut start_time: timeval = timeval {
    tv_sec: 0,
    tv_usec: 0,
};
/// The socket the client talks to the server over, which is what `-S` and
/// `-L` between them decide.
pub static mut socket_path: Option<CString> = None;
pub static mut ptm_fd: core::ffi::c_int = -(1 as core::ffi::c_int);
/// The command `-c` was given, which the client asks the server to run in a
/// shell instead of attaching.
pub static mut shell_command: Option<CString> = None;

fn usage(status: core::ffi::c_int) -> ! {
    let message = [b"usage: ".as_slice(), getprogname().to_bytes(), b" [-2CDhlNuVv] [-c shell-command] [-f file] [-L socket-name]\n            [-S socket-path] [-T features] [command [flags]]\n"].concat();
    if status != 0 {
        let _ = std::io::stderr().lock().write_all(&message);
    } else {
        let mut output = std::io::stdout().lock();
        let _ = output.write_all(&message);
        let _ = output.flush();
    }
    std::process::exit(status);
}

/// The shell `default-shell` starts out as: the one `SHELL` names, else the
/// one the password entry gives, else `/bin/sh`.
fn getshell() -> CString {
    unsafe {
        if let Some(shell) = process_environment_value(c"SHELL")
            && checkshell(Some(&shell)) != 0
        {
            return shell;
        }
        if let Some(account) = UserAccountRecord::lookup_uid(getuid())
            && let Some(shell) = account
                .account_shell()
                .filter(|shell| checkshell(Some(shell)) != 0)
        {
            return shell.to_owned();
        }
        c"/bin/sh".to_owned()
    }
}
pub unsafe fn checkshell(shell: Option<&CStr>) -> core::ffi::c_int {
    unsafe {
        let Some(shell) = shell else {
            return 0 as core::ffi::c_int;
        };
        if shell.to_bytes().first() != Some(&{ b'/' }) {
            return 0 as core::ffi::c_int;
        }
        if areshell(shell) != 0 {
            return 0 as core::ffi::c_int;
        }
        if access(shell.as_ptr(), X_OK) != 0 as core::ffi::c_int {
            return 0 as core::ffi::c_int;
        }
        1 as core::ffi::c_int
    }
}
fn areshell(shell: &CStr) -> core::ffi::c_int {
    let basename = shell
        .to_bytes()
        .rsplit(|&byte| byte == b'/')
        .next()
        .unwrap_or_default();
    let name = getprogname();
    let progname = name.to_bytes();
    let progname = progname.strip_prefix(b"-").unwrap_or(progname);
    (basename == progname) as core::ffi::c_int
}
unsafe fn expand_path(path: &CStr, home: Option<&CStr>) -> Option<CString> {
    {
        let path = path.to_bytes();
        if path.starts_with(b"~/") {
            let mut expanded = home?.to_bytes().to_vec();
            expanded.extend_from_slice(&path[1..]);
            return Some(CString::new(expanded).expect("a C string has no interior NUL"));
        }
        if path.first() == Some(&b'$') {
            let slash = path[1..].iter().position(|&byte| byte == b'/');
            let name_end = slash.map_or(path.len(), |at| at + 1);
            let name = CString::new(&path[1..name_end]).expect("a C string has no interior NUL");
            let mut expanded = with_global_environment(|env| {
                env.find(name.as_c_str())
                    .and_then(|entry| entry.value)
                    .map(|value| value.to_bytes().to_vec())
            })?;
            let suffix = slash.map_or(&[][..], |at| &path[at + 1..]);
            expanded.extend_from_slice(suffix);
            return Some(CString::new(expanded).expect("a C string has no interior NUL"));
        }
        Some(CString::new(path).expect("a C string has no interior NUL"))
    }
}
unsafe fn expand_paths(s: &CStr, no_realpath: core::ffi::c_int) -> Vec<CString> {
    unsafe {
        let home = find_home();
        let mut paths: Vec<CString> = Vec::new();
        for next in s.to_bytes().split(|&byte| byte == b':') {
            let next = CString::new(next).expect("a C string has no interior NUL");
            let Some(expanded) = expand_path(&next, home) else {
                log_debug(
                    c"%s: invalid path: %s",
                    fmt_args![c"expand_paths", next.as_c_str()],
                );
                continue;
            };
            let owned = if no_realpath != 0 {
                expanded
            } else {
                match fs::canonicalize(OsStr::from_bytes(expanded.to_bytes())) {
                    Ok(resolved) => CString::new(resolved.into_os_string().into_vec())
                        .expect("a resolved path has no interior NUL"),
                    Err(err) => {
                        log_debug(
                            c"%s: realpath(\"%s\") failed: %s",
                            fmt_args![
                                c"expand_paths",
                                expanded.as_c_str(),
                                error_message(err.raw_os_error().unwrap_or(0)).as_c_str()
                            ],
                        );
                        continue;
                    }
                }
            };
            if paths.contains(&owned) {
                log_debug(
                    c"%s: duplicate path: %s",
                    fmt_args![c"expand_paths", owned.as_c_str()],
                );
            } else {
                paths.push(owned);
            }
        }
        paths
    }
}
fn make_label(label: Option<&CStr>) -> Result<CString, CString> {
    unsafe {
        let label = label.unwrap_or(c"default");
        let uid = getuid() as uid_t;
        let paths = expand_paths(c"$TMUX_TMPDIR:/tmp/", 0 as core::ffi::c_int);
        let Some(first) = paths.first() else {
            return Err(xasprintf(c"no suitable socket path", fmt_args![]));
        };
        let base = xasprintf(
            c"%s/tmux-%ld",
            fmt_args![first.as_c_str(), uid as core::ffi::c_long],
        );
        drop(paths);
        let base_path = OsStr::from_bytes(base.to_bytes());
        let created = DirBuilder::new().mode(S_IRWXU as u32).create(base_path);
        if let Err(err) = created
            && err.kind() != ErrorKind::AlreadyExists
        {
            return Err(xasprintf(
                c"couldn't create directory %s (%s)",
                fmt_args![
                    base.as_c_str(),
                    error_message(err.raw_os_error().unwrap_or(0)).as_c_str()
                ],
            ));
        }
        let sb = match fs::symlink_metadata(base_path) {
            Ok(sb) => sb,
            Err(err) => {
                return Err(xasprintf(
                    c"couldn't read directory %s (%s)",
                    fmt_args![
                        base.as_c_str(),
                        error_message(err.raw_os_error().unwrap_or(0)).as_c_str()
                    ],
                ));
            }
        };
        if !sb.file_type().is_dir() {
            Err(xasprintf(
                c"%s is not a directory",
                fmt_args![base.as_c_str()],
            ))
        } else if sb.uid() != uid || sb.permissions().mode() & TMUX_SOCK_PERM as u32 != 0 {
            Err(xasprintf(
                c"directory %s has unsafe permissions",
                fmt_args![base.as_c_str()],
            ))
        } else {
            Ok(xasprintf(c"%s/%s", fmt_args![base.as_c_str(), label]))
        }
    }
}
pub unsafe fn shell_argv0(shell: &CStr, is_login: core::ffi::c_int) -> CString {
    {
        let bytes = shell.to_bytes();
        let start = bytes
            .iter()
            .rposition(|byte| *byte == b'/')
            .map(|slash| slash + 1)
            .filter(|start| *start < bytes.len())
            .unwrap_or(0);
        let name = CStr::from_bytes_with_nul(&shell.to_bytes_with_nul()[start..])
            .expect("a suffix of a C string retains its terminator");
        if is_login != 0 {
            xasprintf(c"-%s", fmt_args![name])
        } else {
            xasprintf(c"%s", fmt_args![name])
        }
    }
}
pub fn setblocking(fd: core::ffi::c_int, state: core::ffi::c_int) {
    unsafe {
        let mut mode: core::ffi::c_int;
        mode = fcntl(fd, F_GETFL);
        if mode != -(1 as core::ffi::c_int) {
            if state == 0 {
                mode |= O_NONBLOCK;
            } else {
                mode &= !O_NONBLOCK;
            }
            fcntl(fd, F_SETFL, mode);
        }
    }
}
/// Milliseconds elapsed on a monotonic clock whose epoch is the first call.
pub fn get_timer() -> uint64_t {
    static START: LazyLock<Instant> = LazyLock::new(Instant::now);
    START.elapsed().as_millis() as uint64_t
}
pub unsafe fn clean_name(name: &CStr, untrusted: core::ffi::c_int) -> Option<CString> {
    unsafe {
        if !RustUtf8VisModel.is_valid(name) {
            return None;
        }
        let mut copy = name.to_bytes().to_vec();
        if untrusted != 0 {
            for i in 0..copy.len().saturating_sub(1) {
                if copy[i] == b'#' && copy[i + 1] == b'(' {
                    copy[i] = b'_';
                }
            }
        }
        let copy = CString::from_vec_unchecked(copy);
        Some(
            RustUtf8VisModel
                .encode_utf8(copy.as_bytes(), VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL),
        )
    }
}
pub unsafe fn check_name(name: Option<&CStr>) -> core::ffi::c_int {
    let Some(name) = name else {
        return 0 as core::ffi::c_int;
    };
    if !RustUtf8VisModel.is_valid(name) {
        return 0 as core::ffi::c_int;
    }
    1 as core::ffi::c_int
}
pub fn sig2name(signo: core::ffi::c_int) -> CString {
    CString::new(format!("{signo}")).expect("a number has no NUL")
}
/// The working directory, named the way `PWD` names it when that is the same
/// directory, since the shell's spelling of it may keep symbolic links the
/// resolved one has lost.
pub fn find_cwd() -> Option<CString> {
    unsafe {
        let mut buf = [0u8; 4096];
        if getcwd(buf.as_mut_ptr().cast(), buf.len()).is_null() {
            return None;
        }
        let cwd = CStr::from_bytes_until_nul(&buf)
            .expect("getcwd terminates its successful result")
            .to_owned();
        let Some(pwd) = process_environment_value(c"PWD").filter(|pwd| !pwd.is_empty()) else {
            return Some(cwd);
        };
        let Ok(resolved1) = fs::canonicalize(OsStr::from_bytes(pwd.to_bytes())) else {
            return Some(cwd);
        };
        let Ok(resolved2) = fs::canonicalize(OsStr::from_bytes(cwd.to_bytes())) else {
            return Some(cwd);
        };
        if resolved1 != resolved2 {
            return Some(cwd);
        }
        Some(pwd)
    }
}
/// The home directory, from the environment or from the password file, kept
/// once it has been worked out.
pub fn find_home() -> Option<&'static CStr> {
    unsafe {
        static CACHED_HOME: OnceLock<CString> = OnceLock::new();
        if let Some(home) = CACHED_HOME.get() {
            return Some(home.as_c_str());
        }
        let home = if let Some(home) =
            process_environment_value(c"HOME").filter(|home| !home.is_empty())
        {
            home
        } else {
            UserAccountRecord::lookup_uid(getuid())?
                .account_home()?
                .to_owned()
        };
        Some(CACHED_HOME.get_or_init(|| home).as_c_str())
    }
}
pub fn getversion() -> &'static CStr {
    c"3.7b"
}
/// The whole of `main`, borrowing the owned process arguments.
pub unsafe fn main_0(argv: &mut [CString]) -> core::ffi::c_int {
    unsafe {
        let mut argc = argv.len() as core::ffi::c_int;
        let mut path: Option<CString> = None;
        let mut label: Option<CString> = None;
        let mut opt: core::ffi::c_int;
        let keys: core::ffi::c_int;
        let mut feat: core::ffi::c_int = 0 as core::ffi::c_int;
        let mut fflag: core::ffi::c_int = 0 as core::ffi::c_int;
        let mut flags: uint64_t = 0 as uint64_t;
        if setlocale(LC_CTYPE, c"en_US.UTF-8".as_ptr()).is_null()
            && setlocale(LC_CTYPE, c"C.UTF-8".as_ptr()).is_null()
        {
            if setlocale(LC_CTYPE, c"".as_ptr()).is_null() {
                errx(
                    1 as core::ffi::c_int,
                    c"invalid LC_ALL, LC_CTYPE or LANG".as_ptr(),
                );
            }
            let s = CStr::from_ptr(nl_langinfo(CODESET as core::ffi::c_int as nl_item));
            if !cstr_eq_ignore_case(s, c"UTF-8") && !cstr_eq_ignore_case(s, c"UTF8") {
                errx(
                    1 as core::ffi::c_int,
                    c"need UTF-8 locale (LC_CTYPE) but have %s".as_ptr(),
                    s.as_ptr(),
                );
            }
        }
        setlocale(LC_TIME, c"".as_ptr());
        tzset();
        if argv[0].to_bytes().starts_with(b"-") {
            flags = CLIENT_LOGIN as uint64_t;
        }
        global_options_create();
        for var in process_environment() {
            with_global_environment_mut(|env| env.put(&var, 0));
        }
        if let Some(cwd) = find_cwd() {
            with_global_environment_mut(|env| env.set(c"PWD", 0, &cwd));
        }
        cfg_files = expand_paths(TMUX_CONF, 1 as core::ffi::c_int);
        loop {
            opt = BSDgetopt(argv, c"2c:CDdf:hlL:NqS:T:uUvV");
            if !(opt != -(1 as core::ffi::c_int)) {
                break;
            }
            match opt {
                50 => {
                    feat = RustTerminalFeatureSet.add(feat, c"256", c":,");
                }
                99 => {
                    shell_command = Some(
                        BSDoptarg(argv)
                            .expect("this option requires an argument")
                            .to_owned(),
                    );
                }
                68 => {
                    flags |= CLIENT_NOFORK as uint64_t;
                }
                67 => {
                    if flags & CLIENT_CONTROL as uint64_t != 0 {
                        flags |= CLIENT_CONTROLCONTROL as uint64_t;
                    } else {
                        flags |= CLIENT_CONTROL as uint64_t;
                    }
                }
                102 => {
                    if fflag == 0 {
                        fflag = 1 as core::ffi::c_int;
                        cfg_files.clear();
                    }
                    cfg_files.push(
                        BSDoptarg(argv)
                            .expect("this option requires an argument")
                            .to_owned(),
                    );
                    cfg_quiet = 0 as core::ffi::c_int;
                }
                104 => {
                    usage(0 as core::ffi::c_int);
                }
                86 => {
                    let _ = std::io::stdout()
                        .lock()
                        .write_all(&[b"tmux ".as_slice(), getversion().to_bytes(), b"\n"].concat());
                    {
                        let __status = 0 as core::ffi::c_int;
                        std::process::exit(__status)
                    };
                }
                108 => {
                    flags |= CLIENT_LOGIN as uint64_t;
                }
                76 => {
                    label = Some(
                        BSDoptarg(argv)
                            .expect("this option requires an argument")
                            .to_owned(),
                    );
                }
                78 => {
                    flags |= CLIENT_NOSTARTSERVER as uint64_t;
                }
                113 => {}
                83 => {
                    path = Some(
                        BSDoptarg(argv)
                            .expect("this option requires an argument")
                            .to_owned(),
                    );
                }
                84 => {
                    feat = RustTerminalFeatureSet.add(
                        feat,
                        BSDoptarg(argv).expect("this option requires an argument"),
                        c":,",
                    );
                }
                117 => {
                    flags |= CLIENT_UTF8 as uint64_t;
                }
                118 => {
                    log_add_level();
                }
                _ => {
                    usage(1 as core::ffi::c_int);
                }
            }
        }
        argc -= BSDoptind;
        let argv = &argv[BSDoptind as usize..];
        if shell_command.is_some() && argc != 0 as core::ffi::c_int {
            usage(1 as core::ffi::c_int);
        }
        if flags & CLIENT_NOFORK as uint64_t != 0 && argc != 0 as core::ffi::c_int {
            usage(1 as core::ffi::c_int);
        }
        ptm_fd = getptmfd();
        if ptm_fd == -(1 as core::ffi::c_int) {
            err(1 as core::ffi::c_int, c"getptmfd".as_ptr());
        }
        if process_environment_value(c"TMUX").is_some() {
            flags |= CLIENT_UTF8 as uint64_t;
        } else {
            let locale = [c"LC_ALL", c"LC_CTYPE", c"LANG"]
                .into_iter()
                .find_map(|name| process_environment_value(name).filter(|value| !value.is_empty()));
            let locale = locale.as_deref().unwrap_or(c"");
            if cstr_has_nocase(locale, c"UTF-8") || cstr_has_nocase(locale, c"UTF8") {
                flags |= CLIENT_UTF8 as uint64_t;
            }
        }
        for oe in RustOptionsEngine.table() {
            if oe.scope & OPTIONS_TABLE_SERVER != 0 {
                (global_options
                    .as_ref()
                    .expect("global options are initialized"))
                .set_default(oe);
            }
            if oe.scope & OPTIONS_TABLE_SESSION != 0 {
                (global_s_options
                    .as_ref()
                    .expect("global options are initialized"))
                .set_default(oe);
            }
            if oe.scope & OPTIONS_TABLE_WINDOW != 0 {
                (global_w_options
                    .as_ref()
                    .expect("global options are initialized"))
                .set_default(oe);
            }
        }
        let shell = getshell();
        (global_s_options
            .as_ref()
            .expect("global options are initialized"))
        .set_string(
            c"default-shell",
            0 as core::ffi::c_int,
            c"%s",
            fmt_args![shell.as_c_str()],
        );
        let editor =
            process_environment_value(c"VISUAL").or_else(|| process_environment_value(c"EDITOR"));
        if let Some(editor) = editor {
            (global_options
                .as_ref()
                .expect("global options are initialized"))
            .set_string(
                c"editor",
                0 as core::ffi::c_int,
                c"%s",
                fmt_args![editor.as_c_str()],
            );
            let basename = editor
                .to_bytes()
                .rsplit(|byte| *byte == b'/')
                .next()
                .unwrap_or_default();
            if basename.windows(2).any(|pair| pair == b"vi") {
                keys = MODEKEY_VI;
            } else {
                keys = MODEKEY_EMACS;
            }
            (global_s_options
                .as_ref()
                .expect("global options are initialized"))
            .set_number(c"status-keys", keys as core::ffi::c_longlong);
            (global_w_options
                .as_ref()
                .expect("global options are initialized"))
            .set_number(c"mode-keys", keys as core::ffi::c_longlong);
        }
        if path.is_none()
            && label.is_none()
            && let Some(tmux) = process_environment_value(c"TMUX")
            && !tmux.is_empty()
            && tmux.to_bytes().first() != Some(&b',')
        {
            let tmux_path = tmux.to_bytes();
            let end = tmux_path
                .iter()
                .position(|&byte| byte == b',')
                .unwrap_or(tmux_path.len());
            path = Some(CString::new(&tmux_path[..end]).expect("a C string has no interior NUL"));
        }
        if path.is_none() {
            match make_label(label.as_deref()) {
                Ok(value) => path = Some(value),
                Err(cause) => {
                    let _ = std::io::stderr()
                        .lock()
                        .write_all(&[cause.to_bytes(), b"\n"].concat());
                    {
                        let __status = 1 as core::ffi::c_int;
                        std::process::exit(__status)
                    };
                }
            }
            flags |= CLIENT_DEFAULTSOCKET as uint64_t;
        }
        socket_path = Some(path.expect("socket path was selected"));
        let status = client_main(osdep_event_init(), argv, flags, feat);
        crate::reactor::shutdown();
        std::process::exit(status);
    }
}
pub const TMUX_CONF: &CStr =
    c"/etc/tmux.conf:~/.tmux.conf:$XDG_CONFIG_HOME/tmux/tmux.conf:~/.config/tmux/tmux.conf";

/// Returns the current global session options handle, preserving absence before
/// initialization and without exposing the global storage slot.
///
/// # Safety
/// Run on the server thread without concurrent global-option replacement.
pub(crate) unsafe fn global_session_options() -> Option<RustOptionsRef> {
    unsafe { global_s_options.clone() }
}

/// Retains the server, session and window global option contexts, in that order.
///
/// # Safety
/// Call on the server thread without concurrent global-context replacement.
pub unsafe fn global_option_contexts() -> [Option<RustOptionsRef>; 3] {
    unsafe {
        [
            global_options.clone(),
            global_s_options.clone(),
            global_w_options.clone(),
        ]
    }
}
