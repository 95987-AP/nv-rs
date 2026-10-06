//! Setting the menus up from the game's own files and settings: the
//! screen and safe zone from the INI files, the system colours, the eight
//! fonts (`[Fonts] sFontFile_N`), `menus\globals.xml`, and the game
//! settings' text for `&-sName;`. The caller reads the files (loose or from
//! the archives) and hands over the INI and the settings.

use std::collections::HashMap;

use crate::font::{Font, DEFAULT_FONT_FILES};
use crate::tile::{Screen, SystemColors, Ui};

/// Where `globals.xml` is.
pub const GLOBALS_FILE: &str = "menus\\globals.xml";

/// The exe's own values for the text settings the menus read, used where
/// no plugin sets them (`FalloutNV.esm` overrides only a few game
/// settings; the rest keep the default the exe constructs them with).
/// Read from FalloutNV.exe's static initializers, one per setting (the
/// string pushed before the name; e.g. `sStats` = "STATS", object
/// `011d3fdc`; `sInventoryWeapons` = "Weapons", `011d29b8`).
pub const EXE_TEXT_SETTINGS: &[(&str, &str)] = &[
    // The HUD.
    ("sHitPointsShort", "HP"),
    ("sActionPointsShort", "AP"),
    ("sInventoryAmmo", "AMMO"),
    ("sInventoryCondition", "CND"),
    ("sStatsXP", "XP"),
    ("sLevelUp", "LEVEL UP"),
    // The stats menu (`007da2c0` sets these on its tiles).
    ("sStats", "STATS"),
    ("sStatsLVLAbbrev", "LVL"),
    ("sStatsHP", "HP"),
    ("sStatsAP", "AP"),
    ("sStatsXPMax", "MAX"),
    ("sStatsCNDAbbrev", "CND"),
    ("sStatsRADAbbrev", "RAD"),
    ("sStatsEFFAbbrev", "EFF"),
    ("sStatsH20Abbrev", "H20"),
    ("sStatsFODAbbrev", "FOD"),
    ("sStatsSLPAbbrev", "SLP"),
    ("sStatsCrippled", "CRIPPLED"),
    ("sStatsRadResist", "RAD RESIST"),
    ("sStatsNoEffects", "NO STATUS EFFECTS"),
    ("sStatsStatus", "Status"),
    ("sStatsSpecial", "S.P.E.C.I.A.L."),
    ("sStatsSkills", "Skills"),
    ("sStatsPerks", "Perks"),
    ("sStatsGeneral", "General"),
    ("sStatsBody", "Body"),
    ("sStatsLimbs", "Limbs"),
    ("sStatsAlcohol", "Alcohol"),
    ("sHUDRads", "Rads"),
    ("sMenuDisplayLevelString", "Level"),
    ("sAlignNeutral", "Neutral"),
    ("sAlignGood", "Good"),
    ("sAlignEvil", "Evil"),
    ("sAlignVeryGood", "Very Good"),
    ("sAlignVeryEvil", "Very Evil"),
    // The inventory menu (`0077fc10`) and the item card (`00707e30`).
    ("sInventoryItems", "ITEMS"),
    ("sInventoryWeapons", "Weapons"),
    ("sInventoryApparel", "Apparel"),
    ("sInventoryAid", "Aid"),
    ("sInventoryMisc", "Misc"),
    ("sInventoryAmmoTab", "Ammo"),
    ("sInventoryCaps", "Caps"),
    ("sInventoryWeight", "Wg"),
    ("sInventoryWeightUpper", "WG"),
    ("sInventoryValue", "VAL"),
    ("sInventoryDamage", "DAM"),
    ("sInventoryDamagePerSecond", "DPS"),
    ("sInventoryDamageResistance", "DR"),
    ("sInventoryDamageThreshold", "DT"),
    ("sInventoryStrReq", "STR"),
    ("sInventoryEffects", "EFFECTS"),
    ("sModEffects", "MODS"),
    ("sInventoryEquip", "Equip"),
    ("sInventoryUnequip", "Unequip"),
    ("sInventoryUse", "Use"),
    ("sInventoryDrop", "Drop"),
    ("sInventoryRepair", "Repair"),
    ("sInventoryHotKey", "Hot Key"),
    ("sInventoryMod", "Mod"),
    ("sCancel", "Cancel"),
    // Apparel's weight class on the item card (`00f48ef0` and the two
    // after it).
    ("sArmorWeightLight", "Light"),
    ("sArmorWeightMedium", "Medium"),
    ("sArmorWeightHeavy", "Heavy"),
    // The DATA menu (`00796b90`, `0079a130`).
    ("sData", "DATA"),
    ("sLocalMapTabText", "Local Map"),
    ("sWorldMapTabText", "World Map"),
    ("sQuestsTabText", "Quests"),
    ("sMiscTabText", "Misc"),
    ("sCommsTabText", "Radio"),
    ("sTravel", "Travel"),
    ("sMakeActiveQuest", "Make Active Quest"),
    ("sPlayAudio", "Play Audio"),
    ("sStopAudio", "Stop Audio"),
    ("sTune", "Tune"),
    ("sPlaceMarker", "Place Marker"),
    ("sShowLocation", "Show Location"),
    ("sShowAllNotes", "Show All Notes"),
    ("sShowActiveNotes", "Show Active Quest Notes"),
    ("sNextChallenge", "Next Challenge Type"),
    ("sChallengeToggle", "Challenges"),
    ("sNotesToggle", "Notes"),
    ("sRemaining", "remaining"),
    ("sTravelQuestion", "Do you want to travel to"),
    (
        "sNoFastTravelUndiscovered",
        "You have not discovered this location yet.",
    ),
    ("sYes", "Yes"),
    ("sNo", "No"),
    ("sFullHealth", "You are already at full health."),
    // The keyboard shortcuts written beside the buttons.
    ("sPCMenuHintA", "A)"),
    ("sPCMenuHintE", "E)"),
    ("sPCMenuHintF", "F)"),
    ("sPCMenuHintQ", "Q)"),
    ("sPCMenuHintR", "R)"),
    ("sPCMenuHintS", "S)"),
    ("sPCMenuHintW", "W)"),
    ("sPCMenuHintX", "X)"),
    // Every other text setting the game's menu files name (`&-sName;`), and
    // the ones the container, quantity and other menus' code sets (read
    // from the same initializers; `sOk` is the quantity menu's `sOK`).
    ("sAccept", "Accept"),
    ("sAcceptText", "Accept"),
    ("sActiveMineDescription", "Live"),
    ("sAddText", "Add"),
    ("sAllCardsText", "All Cards"),
    ("sAntePotText", "Ante Pot: "),
    ("sAutoAttempt", "Auto Attempt"),
    ("sAutoMatchText", "Auto Match"),
    ("sBack", "Back"),
    ("sCaravanBiggestAnteText", "Biggest Ante Pot Won: "),
    ("sCaravanLossesText", "Losses to Date: "),
    ("sCaravanWinLossText", "Win/Loss Totals: "),
    ("sCaravanWinningsText", "Winnings to Date: "),
    ("sCardsInDeckText", "Cards in Deck: "),
    ("sCancelBarter", "Cancel transaction?"),
    ("sCauterize", "Cauterizing tool"),
    ("sCloseButton", "Close"),
    (
        "sConfirmWarning",
        "WARNING:  Any unsaved progress will be lost.",
    ),
    ("sContinue", "Continue"),
    ("sCrew", "Credits"),
    ("sCurrentAnteText", "Current Ante: "),
    ("sCurrentObjective", "CURRENT OBJECTIVE"),
    ("sCWheelTitle", "Companion Commands"),
    ("sDevice", "Device"),
    ("sDiscardCardText", "Discard Card"),
    ("sDiscardTrackText", "Discard Track"),
    ("sDone", "Done"),
    ("sEnterName", "Enter character name."),
    ("sExit", "Exit"),
    ("sExplosive", "EXPLOSIVE"),
    ("sForfeitGameText", "Forfeit Game"),
    ("sHackingLockout3", "TERMINAL LOCKED"),
    ("sHackingLockout4", "PLEASE CONTACT AN ADMINISTRATOR"),
    ("sHowMany", "How many?"),
    ("sHowManyWait", "How long would you like to"),
    ("sIngredients", "Ingredients"),
    ("sLevelProgress", "Level Progress"),
    ("sLoad", "Load"),
    ("sLoadSavedGame", "Load Saved Game"),
    ("sLockLevelText", "Lock Level"),
    ("sLockpickSkillText", "Lockpick Skill"),
    ("sMadeAt", "Made at ..."),
    ("sMedicineSkill", "Med skill"),
    (
        "sMissingImage",
        "Data\\Textures\\Interface\\Shared\\missing_image.dds",
    ),
    ("sMorphine", "Morphine"),
    ("sNavigate", "Navigate"),
    ("sNavigateText", "Navigate"),
    ("sNew", "New"),
    ("sNext", "Next"),
    ("sNoItemsToRepair", "No items to repair"),
    ("sOk", "Ok"),
    ("sPCMenuHintRMB", "RMB)"),
    ("sPicksRemainingText", "Bobby Pins"),
    ("sPlayCaravanText", "Play Caravan"),
    ("sPressStart", "Press START"),
    ("sPrevious", "Previous"),
    ("sQuit", "Quit"),
    ("sRaiseText", "Raise"),
    ("sRecipes", "RECIPES"),
    ("sRedeemCode", "Enter Code"),
    ("sRemoveText", "Remove"),
    ("sRepairAllItems", "Repair All (%d)"),
    ("sRepairItem", "Repair"),
    ("sRepairServicesTitle", "Repair Services"),
    ("sRepairSkill", "Repair Skill"),
    // The Pip-Boy's hot keys (read from the settings `00781ba0` loads).
    ("sCantHotkeyItem", "You cannot hotkey that item."),
    ("sCantHotkeyBrokenItem", "You cannot hotkey broken items."),
    ("sReset", "Reset"),
    ("sReturn", "Return"),
    ("sSave", "Save"),
    ("sSaveGame", "Save Game"),
    ("sSelect", "Select"),
    ("sSelectCardText", "Select Card"),
    ("sSettings", "Settings"),
    ("sSkillRequirement", "Skill Requirement"),
    ("sSpace", " "),
    ("sStatsLMBAbbrev", "LMB"),
    ("sStimpak", "Stimpak"),
    ("sSwitchLimbs", "Switch Limbs"),
    ("sTakeAll", "Take All"),
    ("sTotalCardsText", "Total Cards: "),
    ("sTotalFundsText", "Total Funds: "),
    ("sVatsAiming", "Aiming"),
    ("sVatsBodyPart", "Body Part"),
    ("sVatsSelect", "Select"),
    ("sVatsTarget", "Target"),
    ("sVDSGManual", "VDSG MANUAL"),
    ("sVDSGPlate", "VDSG Plate"),
    ("sWinLossText", "Win/Loss: "),
    ("sXPProgressMarker", "^"),
    // The level-up menu (`00784c80`, `00785990`) and a perk's requirements
    // (`005ebac0`).
    ("sLevelUpTitleText", "WELCOME TO LEVEL %d"),
    ("sLevelUpSkillCounter", "ASSIGN 1 SKILL POINT"),
    ("sLevelUpSkillCounterPl", "ASSIGN %d SKILL POINTS"),
    ("sLevelUpPerkCounter", "CHOOSE 1 PERK"),
    ("sLevelUpPerkCounterPl", "CHOOSE %d PERKS"),
    ("sRequirementsText", "Req"),
    ("sLevelAbbrev", "Level"),
    ("sOr", "OR"),
    ("sRanksText", "Ranks"),
    // The trait menu (`007e6990`, `007e71b0`).
    ("sTraitMenuTitleText", "CHOOSE UP TO %d TRAITS"),
    ("sTraitMenuTitleTextSingular", "CHOOSE %d TRAIT"),
    ("sTraitMenuCounter", "%d TRAITS LEFT"),
    ("sTraitMenuCounterSingular", "%d TRAIT LEFT"),
    // The character generation menu (`00753d20`).
    ("sAttributesTitle", "ATTRIBUTES: %d/%d Points Assigned"),
    ("sSkillsTitle", "SKILLS:  %d/%d Selected"),
    ("sAttributesCount", "Distribute %d Point"),
    ("sSkillsCount", "Tag %d Skill"),
    // The sleep/wait menu (`007bfc30`, `007c0000`, `007c03a0`) and the
    // weekday names its time line uses (`00867f10`, the table at
    // `011895b8`).
    ("sSleep", "sleep"),
    ("sWait", "wait"),
    ("sHour", "hour"),
    ("sHours", "hours"),
    ("sAMTime", "AM"),
    ("sPMTime", "PM"),
    ("sDaySunday", "Sunday"),
    ("sDayMonday", "Monday"),
    ("sDayTuesday", "Tuesday"),
    ("sDayWednesday", "Wednesday"),
    ("sDayThursday", "Thursday"),
    ("sDayFriday", "Friday"),
    ("sDaySaturday", "Saturday"),
    // The Pip-Boy's map (`00796fd0` case 0x0c: the player's marker) and
    // ITEMS' Drop (`00780140` case 7).
    ("sSetMarkerQuestion", "Do you want to set your marker?"),
    (
        "sMoveMarkerQuestion",
        "Do you want to move your marker or remove it?",
    ),
    ("sMoveMarker", "Move It"),
    ("sRemoveMarker", "Remove It"),
    ("sLeaveMarker", "Leave It"),
    (
        "sDropQuestItemWarning",
        "You cannot remove Quest Items from your Inventory.",
    ),
    ("sKeyring", "Keyring"),
];

/// A text setting's exe default (see [`EXE_TEXT_SETTINGS`]), name compared
/// without case.
pub fn exe_text_setting(name: &str) -> Option<&'static str> {
    EXE_TEXT_SETTINGS
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, v)| *v)
}

/// The INI files' values: (section, key) to text.
pub type Ini<'a> = &'a dyn Fn(&str, &str) -> Option<String>;

/// The screen: its pixel size, and the safe zone from `[Interface]
/// iSafeZoneXWide`/`YWide` on a screen wider than 4:3, else `iSafeZoneX`/
/// `Y` (`007177c0`, `00717820`; which of the two the code picks by the
/// `&widescreen;` flag is inferred from the names; this install has 15
/// for all four).
pub fn screen(ini: Ini, width_px: u32, height_px: u32) -> Screen {
    let wide = u64::from(width_px) * 3 > u64::from(height_px) * 4;
    let number = |key: &str| {
        ini("Interface", key)
            .and_then(|v| v.trim().parse::<f32>().ok())
            .unwrap_or(0.0)
    };
    let (sx, sy) = if wide {
        (number("iSafeZoneXWide"), number("iSafeZoneYWide"))
    } else {
        (number("iSafeZoneX"), number("iSafeZoneY"))
    };
    Screen {
        width_px,
        height_px,
        safe_x: sx,
        safe_y: sy,
    }
}

/// The system colours, with `[Interface] uHUDColor` and `uPipboyColor`.
pub fn colors(ini: Ini) -> SystemColors {
    let unsigned = |key: &str| ini("Interface", key).and_then(|v| v.trim().parse::<u32>().ok());
    SystemColors::new(unsigned("uHUDColor"), unsigned("uPipboyColor"))
}

/// Where fonts 1 to 8 are, under `Data` (`[Fonts] sFontFile_N`, else the
/// exe's defaults, whose `Data\` is dropped), in lower case.
pub fn font_paths(ini: Ini) -> Vec<String> {
    (1..=8)
        .map(|n| {
            let path = ini("Fonts", &format!("sFontFile_{n}"))
                .unwrap_or_else(|| DEFAULT_FONT_FILES[n - 1].to_string());
            let path = path.trim().replace('/', "\\").to_ascii_lowercase();
            path.strip_prefix("data\\")
                .map_or(path.clone(), str::to_string)
        })
        .collect()
}

/// The menus' starting point: the screen, the colours, the fonts that can
/// be read, `globals.xml`, and the settings' text (names compared without
/// case; the plugins' settings first, then the exe's own defaults,
/// [`EXE_TEXT_SETTINGS`]).
pub fn new_ui(
    read: &mut dyn FnMut(&str) -> Option<Vec<u8>>,
    ini: Ini,
    settings: HashMap<String, String>,
    width_px: u32,
    height_px: u32,
) -> Ui {
    let mut settings: HashMap<String, String> = settings
        .into_iter()
        .map(|(k, v)| (k.to_ascii_lowercase(), v))
        .collect();
    for (name, value) in EXE_TEXT_SETTINGS {
        settings
            .entry(name.to_ascii_lowercase())
            .or_insert_with(|| value.to_string());
    }
    let mut ui = Ui::new(
        screen(ini, width_px, height_px),
        colors(ini),
        Box::new(move |name| settings.get(&name.to_ascii_lowercase()).cloned()),
    );
    for (i, path) in font_paths(ini).iter().enumerate() {
        ui.fonts[i] = read(path).and_then(|b| Font::parse(&b).ok());
    }
    if let Some(globals) = read(GLOBALS_FILE) {
        match ui.load_globals(&globals) {
            Ok(tile) => {
                // `0070adb0`: the menus' background opacity comes from the
                // INI (`[Interface] fMenuBackgroundOpacity`, the setting at
                // `011d3174`, default 0.8) × 255.
                let opacity = ini("Interface", "fMenuBackgroundOpacity")
                    .and_then(|v| v.trim().parse::<f32>().ok())
                    .unwrap_or(MENU_BACKGROUND_OPACITY);
                if let Some(id) = ui.names.lookup_or_add("_background_fill_alpha") {
                    ui.set_number(tile, id, opacity * 255.0);
                }
            }
            Err(e) => ui.warnings.push(format!("globals.xml: {e}")),
        }
    }
    ui
}

/// `[Interface] fMenuBackgroundOpacity`'s default in the exe.
pub const MENU_BACKGROUND_OPACITY: f32 = 0.8;
/// `[Interface] fPopUpBackgroundOpacity`'s default (`011d3bc8`): a message
/// box over another menu.
pub const POPUP_BACKGROUND_OPACITY: f32 = 1.0;

/// A float INI setting, or its default.
pub fn ini_float(ini: Ini, section: &str, key: &str, default: f32) -> f32 {
    ini(section, key)
        .and_then(|v| v.trim().parse::<f32>().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_exes_own_text_settings_fill_in_under_the_plugins() {
        let ini = |_: &str, _: &str| -> Option<String> { None };
        let mut plugin = HashMap::new();
        plugin.insert("sStatsHP".to_string(), "PV".to_string());
        let ui = new_ui(&mut |_| None, &ini, plugin, 1920, 1080);
        // A plugin's setting wins; the rest come from the exe.
        assert_eq!(ui.setting_text("sStatsHP").as_deref(), Some("PV"));
        assert_eq!(ui.setting_text("sStats").as_deref(), Some("STATS"));
        assert_eq!(
            ui.setting_text("SINVENTORYWEAPONS").as_deref(),
            Some("Weapons")
        );
        assert_eq!(exe_text_setting("sQuestsTabText"), Some("Quests"));
        // The container's and quantity menu's.
        assert_eq!(ui.setting_text("sTakeAll").as_deref(), Some("Take All"));
        assert_eq!(ui.setting_text("sOK").as_deref(), Some("Ok"));
        assert_eq!(ui.setting_text("sPCMenuHintA").as_deref(), Some("A)"));
        assert_eq!(ui.setting_text("sNothing"), None);
    }

    #[test]
    fn settings_from_the_ini_files() {
        let ini = |section: &str, key: &str| -> Option<String> {
            match (section, key) {
                ("Interface", "iSafeZoneXWide") => Some("15".into()),
                ("Interface", "iSafeZoneYWide") => Some("12".into()),
                ("Interface", "iSafeZoneX") => Some("5".into()),
                ("Interface", "uHUDColor") => Some("4290134783".into()),
                ("Fonts", "sFontFile_1") => {
                    Some("Textures\\Fonts\\Glow_Monofonto_Large.fnt".into())
                }
                _ => None,
            }
        };
        let s = screen(&ini, 1920, 1080);
        assert_eq!((s.safe_x, s.safe_y), (15.0, 12.0));
        let s = screen(&ini, 1024, 768);
        assert_eq!((s.safe_x, s.safe_y), (5.0, 0.0));
        let paths = font_paths(&ini);
        assert_eq!(paths[0], "textures\\fonts\\glow_monofonto_large.fnt");
        assert_eq!(paths[1], "fonts\\monofonto_large.fnt");
        assert_eq!(
            colors(&ini).get(1),
            Some([1.0, 182.0 / 255.0, 66.0 / 255.0])
        );
    }
}
