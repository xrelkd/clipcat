use std::path::PathBuf;

use clap::Args;

#[derive(Args, Clone)]
pub struct RofiConfig {
    #[arg(
        name = "rofi-menu-length",
        long = "rofi-menu-length",
        env = "CLIPCAT_MENU_ROFI_MENU_LENGTH",
        help = "Specify the menu length for Rofi"
    )]
    pub menu_length: Option<usize>,

    #[arg(
        name = "rofi-line-length",
        long = "rofi-line-length",
        env = "CLIPCAT_MENU_ROFI_LINE_LENGTH",
        help = "Specify the length of a line showing on Rofi"
    )]
    pub line_length: Option<usize>,

    #[clap(
        name = "rofi-extra-arguments",
        long = "rofi-extra-arguments",
        env = "CLIPCAT_MENU_ROFI_EXTRA_ARGUMENTS",
        help = "Extra arguments pass to Rofi, use ',' to separate arguments"
    )]
    pub extra_arguments: Option<String>,
}

#[derive(Args, Clone)]
pub struct DmenuConfig {
    #[clap(
        name = "dmenu-menu-length",
        long = "dmenu-menu-length",
        env = "CLIPCAT_MENU_DMENU_MENU_LENGTH",
        help = "Specify the menu length of dmenu"
    )]
    pub menu_length: Option<usize>,

    #[clap(
        name = "dmenu-line-length",
        long = "dmenu-line-length",
        env = "CLIPCAT_MENU_DMENU_LINE_LENGTH",
        help = "Specify the length of a line showing on dmenu"
    )]
    pub line_length: Option<usize>,

    #[clap(
        name = "dmenu-extra-arguments",
        long = "dmenu-extra-arguments",
        env = "CLIPCAT_MENU_DMENU_EXTRA_ARGUMENTS",
        help = "Extra arguments pass to dmenu, use ',' to separate arguments"
    )]
    pub extra_arguments: Option<String>,
}

#[derive(Args, Clone)]
pub struct CustomFinderConfig {
    #[clap(
        name = "custom-finder-program-path",
        long = "custom-finder-program-path",
        env = "CLIPCAT_MENU_CUSTOM_FINDER_PROGRAM_PATH",
        help = "The program path of custom finder"
    )]
    pub program_path: Option<PathBuf>,

    #[clap(
        name = "custom-finder-arguments",
        long = "custom-finder-arguments",
        env = "CLIPCAT_MENU_CUSTOM_FINDER_ARGUMENTS",
        help = "Arguments pass to custom finder, use ',' to separate arguments"
    )]
    pub arguments: Option<String>,
}
