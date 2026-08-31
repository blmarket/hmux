use crate::cfg::{cfg_files, cfg_quiet};
use crate::client::client_main;
use crate::compat::BSDgetopt;
use crate::compat::getprogname;
use crate::compat::getptmfd;
use crate::compat::{BSDoptarg, BSDoptind};
use crate::environ::{
    environ_create_box, environ_entry_value, environ_find, environ_process, environ_put,
    environ_set, environ_t,
};
use crate::ffi::{
    access, err, errx, exit, fcntl, fprintf, getcwd, getenv, getpwuid, getuid, nl_langinfo, printf,
    setlocale, stderr, stdout, strcasecmp, strcasestr, strcmp, strerror, strrchr, strstr, tzset,
};
use crate::fmt_args;
use crate::log::{log_add_level, log_debug};
use crate::options::options_table;
use crate::options::{
    options_create_boxed, options_default, options_free, options_set_number, options_set_string,
};
use crate::osdep_linux::osdep_event_init;
use crate::terminfo::tty_add_features;
use crate::text::{utf8_isvalid, utf8_stravis};
pub use crate::types::*;
use crate::xmalloc::xasprintf;
use ::std::ffi::{CStr, CString, OsStr};
use ::std::fs::{self, DirBuilder};
use ::std::io::ErrorKind;
use ::std::os::unix::ffi::{OsStrExt, OsStringExt};
use ::std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use ::std::sync::{LazyLock, OnceLock};
use ::std::time::Instant;
pub type nl_item_value = ::core::ffi::c_uint;
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
pub const OPTIONS_TABLE_COMMAND: options_table_type = 6;
pub const OPTIONS_TABLE_CHOICE: options_table_type = 5;
pub const OPTIONS_TABLE_FLAG: options_table_type = 4;
pub const OPTIONS_TABLE_COLOUR: options_table_type = 3;
pub const OPTIONS_TABLE_KEY: options_table_type = 2;
pub const OPTIONS_TABLE_NUMBER: options_table_type = 1;
pub const OPTIONS_TABLE_STRING: options_table_type = 0;
pub const __S_IREAD: ::core::ffi::c_int = 0o400 as ::core::ffi::c_int;
pub const __S_IWRITE: ::core::ffi::c_int = 0o200 as ::core::ffi::c_int;
pub const __S_IEXEC: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;
pub const __LC_CTYPE: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __LC_TIME: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const O_NONBLOCK: ::core::ffi::c_int = 0o4000 as ::core::ffi::c_int;
pub const F_GETFL: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const F_SETFL: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const S_IRWXU: ::core::ffi::c_int = __S_IREAD | __S_IWRITE | __S_IEXEC;
pub const LC_CTYPE: ::core::ffi::c_int = __LC_CTYPE;
pub const LC_TIME: ::core::ffi::c_int = __LC_TIME;
pub const X_OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const _PATH_BSHELL: &CStr = c"/bin/sh";
pub const VIS_OCTAL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const VIS_CSTYLE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VIS_TAB: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const VIS_NL: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const TMUX_SOCK_PERM: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const MODEKEY_EMACS: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MODEKEY_VI: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const CLIENT_LOGIN: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const CLIENT_NOSTARTSERVER: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const CLIENT_CONTROL: ::core::ffi::c_int = 0x2000 as ::core::ffi::c_int;
pub const CLIENT_CONTROLCONTROL: ::core::ffi::c_int = 0x4000 as ::core::ffi::c_int;
pub const CLIENT_UTF8: ::core::ffi::c_int = 0x10000 as ::core::ffi::c_int;
pub const CLIENT_DEFAULTSOCKET: ::core::ffi::c_int = 0x8000000 as ::core::ffi::c_int;
pub const CLIENT_NOFORK: ::core::ffi::c_int = 0x40000000 as ::core::ffi::c_int;
pub const OPTIONS_TABLE_SERVER: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const OPTIONS_TABLE_SESSION: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const OPTIONS_TABLE_WINDOW: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
/// The option sets and starting environment the server holds for as long as
/// it runs. The four views below are borrowed from here.
static GLOBAL_OPTIONS: crate::tree::GlobalQueue<Box<options>> = crate::tree::GlobalQueue::new();
static GLOBAL_ENVIRON: crate::tree::GlobalQueue<Box<environ_t>> = crate::tree::GlobalQueue::new();

/// Makes the server's option sets and starting environment, and keeps them.
/// The raw views the rest of the crate reads are borrowed from what this
/// holds, so they are good for as long as the server is.
pub unsafe fn global_options_create() {
    unsafe {
        let held = GLOBAL_OPTIONS.queue();
        held.clear();
        for _ in 0..3 {
            held.push_back(options_create_boxed(::core::ptr::null_mut::<options>()));
        }
        global_options = &raw mut **held.front_mut().expect("three sets were just made");
        global_s_options = &raw mut *held[1];
        global_w_options = &raw mut *held[2];

        let held = GLOBAL_ENVIRON.queue();
        held.clear();
        held.push_back(environ_create_box());
        global_environ = &raw mut **held.front_mut().expect("an environment was just made");
    }
}

/// Gives up the option sets and starting environment, which a client process
/// stops needing once it has connected. The views below are left null, since
/// what they borrowed has gone.
pub unsafe fn global_options_free() {
    unsafe {
        global_options = ::core::ptr::null_mut::<options>();
        global_s_options = ::core::ptr::null_mut::<options>();
        global_w_options = ::core::ptr::null_mut::<options>();
        global_environ = ::core::ptr::null_mut::<environ_t>();
        let held = GLOBAL_OPTIONS.queue();
        while let Some(oo) = held.pop_front() {
            options_free(oo);
        }
        GLOBAL_ENVIRON.queue().clear();
    }
}

pub static mut global_options: *mut options = ::core::ptr::null::<options>() as *mut options;
pub static mut global_s_options: *mut options = ::core::ptr::null::<options>() as *mut options;
pub static mut global_w_options: *mut options = ::core::ptr::null::<options>() as *mut options;
pub static mut global_environ: *mut environ_t = ::core::ptr::null::<environ_t>() as *mut environ_t;
pub static mut start_time: timeval = timeval {
    tv_sec: 0,
    tv_usec: 0,
};
/// The socket the client talks to the server over, which is what `-S` and
/// `-L` between them decide.
pub static mut socket_path: Option<CString> = None;
pub static mut ptm_fd: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
/// The command `-c` was given, which the client asks the server to run in a
/// shell instead of attaching.
pub static mut shell_command: Option<CString> = None;
fn usage(mut status: ::core::ffi::c_int) -> ! {
    unsafe {
        fprintf(
        if status != 0 { stderr } else { stdout },
        c"usage: %s [-2CDhlNuVv] [-c shell-command] [-f file] [-L socket-name]\n            [-S socket-path] [-T features] [command [flags]]\n".as_ptr(),
        getprogname().as_ptr(),
    );
        exit(status);
    }
}
/// The shell `default-shell` starts out as: the one `SHELL` names, else the
/// one the password entry gives, else `/bin/sh`.
fn getshell() -> CString {
    unsafe {
        let shell = getenv(c"SHELL".as_ptr());
        if checkshell(shell) != 0 {
            return CStr::from_ptr(shell).to_owned();
        }
        let pw = getpwuid(getuid());
        if !pw.is_null() && checkshell((*pw).pw_shell) != 0 {
            return CStr::from_ptr((*pw).pw_shell).to_owned();
        }
        c"/bin/sh".to_owned()
    }
}
pub unsafe fn checkshell(mut shell: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        if shell.is_null() || *shell as ::core::ffi::c_int != '/' as i32 {
            return 0 as ::core::ffi::c_int;
        }
        if areshell(shell) != 0 {
            return 0 as ::core::ffi::c_int;
        }
        if access(shell, X_OK) != 0 as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        1 as ::core::ffi::c_int
    }
}
unsafe fn areshell(mut shell: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        let mut progname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut ptr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        ptr = strrchr(shell, '/' as i32);
        if !ptr.is_null() {
            ptr = ptr.offset(1);
        } else {
            ptr = shell;
        }
        progname = getprogname().as_ptr();
        if *progname as ::core::ffi::c_int == '-' as i32 {
            progname = progname.offset(1);
        }
        if strcmp(ptr, progname) == 0 as ::core::ffi::c_int {
            return 1 as ::core::ffi::c_int;
        }
        0 as ::core::ffi::c_int
    }
}
unsafe fn expand_path(
    path: *const ::core::ffi::c_char,
    home: *const ::core::ffi::c_char,
) -> Option<CString> {
    unsafe {
        let path = CStr::from_ptr(path).to_bytes();
        if path.starts_with(b"~/") {
            if home.is_null() {
                return None;
            }
            let mut expanded = CStr::from_ptr(home).to_bytes().to_vec();
            expanded.extend_from_slice(&path[1..]);
            return Some(CString::new(expanded).expect("a C string has no interior NUL"));
        }
        if path.first() == Some(&b'$') {
            let slash = path[1..].iter().position(|&byte| byte == b'/');
            let name_end = slash.map_or(path.len(), |at| at + 1);
            let name = CString::new(&path[1..name_end]).expect("a C string has no interior NUL");
            let Some(value) =
                environ_find(&*global_environ, name.as_ptr()).and_then(environ_entry_value)
            else {
                return None;
            };
            let suffix = slash.map_or(&[][..], |at| &path[at + 1..]);
            let mut expanded = value.to_bytes().to_vec();
            expanded.extend_from_slice(suffix);
            return Some(CString::new(expanded).expect("a C string has no interior NUL"));
        }
        Some(CString::new(path).expect("a C string has no interior NUL"))
    }
}
unsafe fn expand_paths(
    s: *const ::core::ffi::c_char,
    no_realpath: ::core::ffi::c_int,
) -> Vec<CString> {
    unsafe {
        let home: *const ::core::ffi::c_char =
            find_home().map_or(::core::ptr::null(), CStr::as_ptr);
        let mut paths: Vec<CString> = Vec::new();
        for next in CStr::from_ptr(s).to_bytes().split(|&byte| byte == b':') {
            let next = CString::new(next).expect("a C string has no interior NUL");
            let Some(expanded) = expand_path(next.as_ptr(), home) else {
                log_debug(
                    c"%s: invalid path: %s".as_ptr(),
                    fmt_args![c"expand_paths".as_ptr(), next.as_ptr()],
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
                            c"%s: realpath(\"%s\") failed: %s".as_ptr(),
                            fmt_args![
                                c"expand_paths".as_ptr(),
                                expanded.as_ptr(),
                                strerror(err.raw_os_error().unwrap_or(0))
                            ],
                        );
                        continue;
                    }
                }
            };
            if paths.contains(&owned) {
                log_debug(
                    c"%s: duplicate path: %s".as_ptr(),
                    fmt_args![c"expand_paths".as_ptr(), owned.as_ptr()],
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
        let paths = expand_paths(c"$TMUX_TMPDIR:/tmp/".as_ptr(), 0 as ::core::ffi::c_int);
        let Some(first) = paths.first() else {
            return Err(xasprintf(c"no suitable socket path".as_ptr(), fmt_args![]));
        };
        let base = xasprintf(
            c"%s/tmux-%ld".as_ptr(),
            fmt_args![first.as_ptr(), uid as ::core::ffi::c_long],
        );
        drop(paths);
        let base_path = OsStr::from_bytes(base.to_bytes());
        let created = DirBuilder::new().mode(S_IRWXU as u32).create(base_path);
        if let Err(err) = created
            && err.kind() != ErrorKind::AlreadyExists
        {
            return Err(xasprintf(
                c"couldn't create directory %s (%s)".as_ptr(),
                fmt_args![base.as_ptr(), strerror(err.raw_os_error().unwrap_or(0))],
            ));
        }
        let sb = match fs::symlink_metadata(base_path) {
            Ok(sb) => sb,
            Err(err) => {
                return Err(xasprintf(
                    c"couldn't read directory %s (%s)".as_ptr(),
                    fmt_args![base.as_ptr(), strerror(err.raw_os_error().unwrap_or(0))],
                ));
            }
        };
        if !sb.file_type().is_dir() {
            Err(xasprintf(
                c"%s is not a directory".as_ptr(),
                fmt_args![base.as_ptr()],
            ))
        } else if sb.uid() != uid || sb.permissions().mode() & TMUX_SOCK_PERM as u32 != 0 {
            Err(xasprintf(
                c"directory %s has unsafe permissions".as_ptr(),
                fmt_args![base.as_ptr()],
            ))
        } else {
            Ok(xasprintf(
                c"%s/%s".as_ptr(),
                fmt_args![base.as_ptr(), label.as_ptr()],
            ))
        }
    }
}
pub unsafe fn shell_argv0(
    mut shell: *const ::core::ffi::c_char,
    mut is_login: ::core::ffi::c_int,
) -> CString {
    unsafe {
        let mut slash: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut name: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        slash = strrchr(shell, '/' as i32);
        if !slash.is_null()
            && *slash.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\0' as i32
        {
            name = slash.offset(1 as ::core::ffi::c_int as isize);
        } else {
            name = shell;
        }
        if is_login != 0 {
            xasprintf(c"-%s".as_ptr(), fmt_args![name])
        } else {
            xasprintf(c"%s".as_ptr(), fmt_args![name])
        }
    }
}
pub fn setblocking(mut fd: ::core::ffi::c_int, mut state: ::core::ffi::c_int) {
    unsafe {
        let mut mode: ::core::ffi::c_int = 0;
        mode = fcntl(fd, F_GETFL);
        if mode != -(1 as ::core::ffi::c_int) {
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
pub unsafe fn clean_name(
    name: *const ::core::ffi::c_char,
    untrusted: ::core::ffi::c_int,
) -> Option<CString> {
    unsafe {
        if utf8_isvalid(name) == 0 {
            return None;
        }
        let mut copy = CStr::from_ptr(name).to_bytes().to_vec();
        if untrusted != 0 {
            for i in 0..copy.len().saturating_sub(1) {
                if copy[i] == b'#' && copy[i + 1] == b'(' {
                    copy[i] = b'_';
                }
            }
        }
        let copy = CString::from_vec_unchecked(copy);
        Some(utf8_stravis(
            &copy,
            VIS_OCTAL | VIS_CSTYLE | VIS_TAB | VIS_NL,
        ))
    }
}
pub unsafe fn check_name(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        if utf8_isvalid(name) == 0 {
            return 0 as ::core::ffi::c_int;
        }
        1 as ::core::ffi::c_int
    }
}
pub fn sig2name(signo: ::core::ffi::c_int) -> CString {
    CString::new(::std::format!("{signo}")).expect("a number has no NUL")
}
/// The working directory, named the way `PWD` names it when that is the same
/// directory, since the shell's spelling of it may keep symbolic links the
/// resolved one has lost.
pub fn find_cwd() -> Option<CString> {
    unsafe {
        let mut buf: [::core::ffi::c_char; 4096] = [0; 4096];
        if getcwd(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>() as size_t,
        )
        .is_null()
        {
            return None;
        }
        let cwd = CStr::from_ptr(&raw const buf as *const ::core::ffi::c_char).to_owned();
        let pwd = getenv(c"PWD".as_ptr());
        if pwd.is_null() || *pwd as ::core::ffi::c_int == '\0' as i32 {
            return Some(cwd);
        }
        let Ok(resolved1) = fs::canonicalize(OsStr::from_bytes(CStr::from_ptr(pwd).to_bytes()))
        else {
            return Some(cwd);
        };
        let Ok(resolved2) = fs::canonicalize(OsStr::from_bytes(cwd.to_bytes())) else {
            return Some(cwd);
        };
        if resolved1 != resolved2 {
            return Some(cwd);
        }
        Some(CStr::from_ptr(pwd).to_owned())
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
        let mut home = getenv(c"HOME".as_ptr());
        if home.is_null() || *home as ::core::ffi::c_int == '\0' as i32 {
            let pw = getpwuid(getuid());
            if pw.is_null() {
                return None;
            }
            home = (*pw).pw_dir;
        }
        Some(
            CACHED_HOME
                .get_or_init(|| CStr::from_ptr(home).to_owned())
                .as_c_str(),
        )
    }
}
pub fn getversion() -> &'static CStr {
    c"3.7b"
}
/// The whole of `main`. `argv` runs one past the arguments, the last slot
/// holding the null terminator the option parser reads.
pub unsafe fn main_0(argv: &mut [*mut ::core::ffi::c_char]) -> ::core::ffi::c_int {
    unsafe {
        let mut argc = argv.len() as ::core::ffi::c_int - 1;
        let mut path: Option<CString> = None;
        let mut label: Option<CString> = None;
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut opt: ::core::ffi::c_int = 0;
        let mut keys: ::core::ffi::c_int = 0;
        let mut feat: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut fflag: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut flags: uint64_t = 0 as uint64_t;
        if setlocale(LC_CTYPE, c"en_US.UTF-8".as_ptr()).is_null()
            && setlocale(LC_CTYPE, c"C.UTF-8".as_ptr()).is_null()
        {
            if setlocale(LC_CTYPE, c"".as_ptr()).is_null() {
                errx(
                    1 as ::core::ffi::c_int,
                    c"invalid LC_ALL, LC_CTYPE or LANG".as_ptr(),
                );
            }
            s = nl_langinfo(CODESET as ::core::ffi::c_int as nl_item);
            if strcasecmp(s, c"UTF-8".as_ptr()) != 0 as ::core::ffi::c_int
                && strcasecmp(s, c"UTF8".as_ptr()) != 0 as ::core::ffi::c_int
            {
                errx(
                    1 as ::core::ffi::c_int,
                    c"need UTF-8 locale (LC_CTYPE) but have %s".as_ptr(),
                    s,
                );
            }
        }
        setlocale(LC_TIME, c"".as_ptr());
        tzset();
        if *argv[0] as ::core::ffi::c_int == '-' as i32 {
            flags = CLIENT_LOGIN as uint64_t;
        }
        global_options_create();
        for var in environ_process() {
            environ_put(global_environ, var.as_ptr(), 0 as ::core::ffi::c_int);
        }
        if let Some(cwd) = find_cwd() {
            environ_set(
                global_environ,
                c"PWD".as_ptr(),
                0 as ::core::ffi::c_int,
                c"%s".as_ptr(),
                fmt_args![cwd.as_ptr()],
            );
        }
        cfg_files = expand_paths(TMUX_CONF.as_ptr(), 1 as ::core::ffi::c_int);
        loop {
            opt = BSDgetopt(argv, c"2c:CDdf:hlL:NqS:T:uUvV".as_ptr());
            if !(opt != -(1 as ::core::ffi::c_int)) {
                break;
            }
            match opt {
                50 => {
                    tty_add_features(&mut feat, c"256".as_ptr(), c":,".as_ptr());
                }
                99 => {
                    shell_command = Some(CStr::from_ptr(BSDoptarg).to_owned());
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
                        fflag = 1 as ::core::ffi::c_int;
                        cfg_files.clear();
                    }
                    cfg_files.push(CStr::from_ptr(BSDoptarg).to_owned());
                    cfg_quiet = 0 as ::core::ffi::c_int;
                }
                104 => {
                    usage(0 as ::core::ffi::c_int);
                }
                86 => {
                    printf(c"tmux %s\n".as_ptr(), getversion().as_ptr());
                    exit(0 as ::core::ffi::c_int);
                }
                108 => {
                    flags |= CLIENT_LOGIN as uint64_t;
                }
                76 => {
                    label = Some(CStr::from_ptr(BSDoptarg).to_owned());
                }
                78 => {
                    flags |= CLIENT_NOSTARTSERVER as uint64_t;
                }
                113 => {}
                83 => {
                    path = Some(CStr::from_ptr(BSDoptarg).to_owned());
                }
                84 => {
                    tty_add_features(&mut feat, BSDoptarg, c":,".as_ptr());
                }
                117 => {
                    flags |= CLIENT_UTF8 as uint64_t;
                }
                118 => {
                    log_add_level();
                }
                _ => {
                    usage(1 as ::core::ffi::c_int);
                }
            }
        }
        argc -= BSDoptind;
        let argv = &argv[BSDoptind as usize..];
        if shell_command.is_some() && argc != 0 as ::core::ffi::c_int {
            usage(1 as ::core::ffi::c_int);
        }
        if flags & CLIENT_NOFORK as uint64_t != 0 && argc != 0 as ::core::ffi::c_int {
            usage(1 as ::core::ffi::c_int);
        }
        ptm_fd = getptmfd();
        if ptm_fd == -(1 as ::core::ffi::c_int) {
            err(1 as ::core::ffi::c_int, c"getptmfd".as_ptr());
        }
        if !getenv(c"TMUX".as_ptr()).is_null() {
            flags |= CLIENT_UTF8 as uint64_t;
        } else {
            s = getenv(c"LC_ALL".as_ptr());
            if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 {
                s = getenv(c"LC_CTYPE".as_ptr());
            }
            if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 {
                s = getenv(c"LANG".as_ptr());
            }
            if s.is_null() || *s as ::core::ffi::c_int == '\0' as i32 {
                s = c"".as_ptr();
            }
            if !strcasestr(s, c"UTF-8".as_ptr()).is_null()
                || !strcasestr(s, c"UTF8".as_ptr()).is_null()
            {
                flags |= CLIENT_UTF8 as uint64_t;
            }
        }
        for oe in &options_table {
            if oe.scope & OPTIONS_TABLE_SERVER != 0 {
                options_default(global_options, oe);
            }
            if oe.scope & OPTIONS_TABLE_SESSION != 0 {
                options_default(global_s_options, oe);
            }
            if oe.scope & OPTIONS_TABLE_WINDOW != 0 {
                options_default(global_w_options, oe);
            }
        }
        let shell = getshell();
        options_set_string(
            global_s_options,
            c"default-shell".as_ptr(),
            0 as ::core::ffi::c_int,
            c"%s".as_ptr(),
            fmt_args![shell.as_c_str()],
        );
        s = getenv(c"VISUAL".as_ptr());
        if !s.is_null() || {
            s = getenv(c"EDITOR".as_ptr());
            !s.is_null()
        } {
            options_set_string(
                global_options,
                c"editor".as_ptr(),
                0 as ::core::ffi::c_int,
                c"%s".as_ptr(),
                fmt_args![s],
            );
            if !strrchr(s, '/' as i32).is_null() {
                s = strrchr(s, '/' as i32).offset(1 as ::core::ffi::c_int as isize);
            }
            if !strstr(s, c"vi".as_ptr()).is_null() {
                keys = MODEKEY_VI;
            } else {
                keys = MODEKEY_EMACS;
            }
            options_set_number(
                global_s_options,
                c"status-keys".as_ptr(),
                keys as ::core::ffi::c_longlong,
            );
            options_set_number(
                global_w_options,
                c"mode-keys".as_ptr(),
                keys as ::core::ffi::c_longlong,
            );
        }
        if path.is_none() && label.is_none() {
            s = getenv(c"TMUX".as_ptr());
            if !s.is_null()
                && *s as ::core::ffi::c_int != '\0' as i32
                && *s as ::core::ffi::c_int != ',' as i32
            {
                let tmux_path = CStr::from_ptr(s).to_bytes();
                let end = tmux_path
                    .iter()
                    .position(|&byte| byte == b',')
                    .unwrap_or(tmux_path.len());
                path =
                    Some(CString::new(&tmux_path[..end]).expect("a C string has no interior NUL"));
            }
        }
        if path.is_none() {
            match make_label(label.as_deref()) {
                Ok(value) => path = Some(value),
                Err(cause) => {
                    fprintf(stderr, c"%s\n".as_ptr(), cause.as_ptr());
                    exit(1 as ::core::ffi::c_int);
                }
            }
            flags |= CLIENT_DEFAULTSOCKET as uint64_t;
        }
        socket_path = Some(path.expect("socket path was selected"));
        let client_argv: Vec<CString> = argv[..argc as usize]
            .iter()
            .map(|arg| CStr::from_ptr(*arg).to_owned())
            .collect();
        let status = client_main(osdep_event_init(), &client_argv, flags, feat);
        crate::reactor::shutdown();
        exit(status);
    }
}
pub const TMUX_VERSION: [::core::ffi::c_char; 5] =
    unsafe { ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b"3.7b\0") };
pub const TMUX_CONF: [::core::ffi::c_char; 85] = unsafe {
    ::core::mem::transmute::<[u8; 85], [::core::ffi::c_char; 85]>(
        *b"/etc/tmux.conf:~/.tmux.conf:$XDG_CONFIG_HOME/tmux/tmux.conf:~/.config/tmux/tmux.conf\0",
    )
};
