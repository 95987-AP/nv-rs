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
    ("sAccept", "Accept"),
    ("sAcceptText", "Accept"),
    ("sActionPointsShort", "AP"),
    ("sActiveMineDescription", "Live"),
    ("sAddItemtoInventory", "added"),
    ("sAddText", "Add"),
    ("sAlignEvil", "Evil"),
    ("sAlignGood", "Good"),
    ("sAlignNeutral", "Neutral"),
    ("sAlignVeryEvil", "Very Evil"),
    ("sAlignVeryGood", "Very Good"),
    ("sAllCardsText", "All Cards"),
    ("sAMTime", "AM"),
    ("sAntePotText", "Ante Pot: "),
    ("sArmorWeightHeavy", "Heavy"),
    ("sArmorWeightLight", "Light"),
    ("sArmorWeightMedium", "Medium"),
    ("sAttributesCount", "Distribute %d Point"),
    ("sAttributesTitle", "ATTRIBUTES: %d/%d Points Assigned"),
    ("sAutoAttempt", "Auto Attempt"),
    ("sAutoMatchText", "Auto Match"),
    ("sBack", "Back"),
    ("sCancel", "Cancel"),
    ("sCancelBarter", "Cancel transaction?"),
    ("sCaravanBiggestAnteText", "Biggest Ante Pot Won: "),
    ("sCaravanLossesText", "Losses to Date: "),
    ("sCaravanWinLossText", "Win/Loss Totals: "),
    ("sCaravanWinningsText", "Winnings to Date: "),
    ("sCardsInDeckText", "Cards in Deck: "),
    ("sCauterize", "Cauterizing tool"),
    ("sChallengeToggle", "Challenges"),
    ("sCloseButton", "Close"),
    ("sCommsTabText", "Radio"),
    ("sContinue", "Continue"),
    ("sCrew", "Credits"),
    ("sCurrentAnteText", "Current Ante: "),
    ("sCurrentObjective", "CURRENT OBJECTIVE"),
    ("sCWheelTitle", "Companion Commands"),
    ("sData", "DATA"),
    ("sDayFriday", "Friday"),
    ("sDayMonday", "Monday"),
    ("sDaySaturday", "Saturday"),
    ("sDaySunday", "Sunday"),
    ("sDayThursday", "Thursday"),
    ("sDayTuesday", "Tuesday"),
    ("sDayWednesday", "Wednesday"),
    ("sDevice", "Device"),
    ("sDiscardCardText", "Discard Card"),
    ("sDiscardTrackText", "Discard Track"),
    ("sDone", "Done"),
    ("sEnterName", "Enter character name."),
    ("sExit", "Exit"),
    ("sExplosive", "EXPLOSIVE"),
    ("sForfeitGameText", "Forfeit Game"),
    ("sFullHealth", "You are already at full health."),
    ("sHackingLockout3", "TERMINAL LOCKED"),
    ("sHackingLockout4", "PLEASE CONTACT AN ADMINISTRATOR"),
    ("sHitPointsShort", "HP"),
    ("sHour", "hour"),
    ("sHours", "hours"),
    ("sHowMany", "How many?"),
    ("sHowManyWait", "How long would you like to"),
    ("sHUDRads", "Rads"),
    ("sIngredients", "Ingredients"),
    ("sInventoryAid", "Aid"),
    ("sInventoryAmmo", "AMMO"),
    ("sInventoryAmmoTab", "Ammo"),
    ("sInventoryApparel", "Apparel"),
    ("sInventoryCaps", "Caps"),
    ("sInventoryCondition", "CND"),
    ("sInventoryDamage", "DAM"),
    ("sInventoryDamagePerSecond", "DPS"),
    ("sInventoryDamageResistance", "DR"),
    ("sInventoryDamageThreshold", "DT"),
    ("sInventoryDrop", "Drop"),
    ("sInventoryEffects", "EFFECTS"),
    ("sInventoryEquip", "Equip"),
    ("sInventoryHotKey", "Hot Key"),
    ("sInventoryItems", "ITEMS"),
    ("sInventoryMisc", "Misc"),
    ("sInventoryMod", "Mod"),
    ("sInventoryRepair", "Repair"),
    ("sInventoryStrReq", "STR"),
    ("sInventoryUnequip", "Unequip"),
    ("sInventoryUse", "Use"),
    ("sInventoryValue", "VAL"),
    ("sInventoryWeapons", "Weapons"),
    ("sInventoryWeight", "Wg"),
    ("sInventoryWeightUpper", "WG"),
    ("sLevelAbbrev", "Level"),
    ("sLevelProgress", "Level Progress"),
    ("sLevelUp", "LEVEL UP"),
    ("sLevelUpPerkCounter", "CHOOSE 1 PERK"),
    ("sLevelUpPerkCounterPl", "CHOOSE %d PERKS"),
    ("sLevelUpSkillCounter", "ASSIGN 1 SKILL POINT"),
    ("sLevelUpSkillCounterPl", "ASSIGN %d SKILL POINTS"),
    ("sLevelUpTitleText", "WELCOME TO LEVEL %d"),
    ("sLoad", "Load"),
    ("sLoadSavedGame", "Load Saved Game"),
    ("sLocalMapTabText", "Local Map"),
    ("sLockLevelText", "Lock Level"),
    ("sLockpickSkillText", "Lockpick Skill"),
    ("sMadeAt", "Made at ..."),
    ("sMakeActiveQuest", "Make Active Quest"),
    ("sMedicineSkill", "Med skill"),
    ("sMenuDisplayLevelString", "Level"),
    ("sMiscTabText", "Misc"),
    ("sModEffects", "MODS"),
    ("sMorphine", "Morphine"),
    ("sNavigate", "Navigate"),
    ("sNavigateText", "Navigate"),
    ("sNew", "New"),
    ("sNext", "Next"),
    ("sNextChallenge", "Next Challenge Type"),
    ("sNo", "No"),
    ("sNoItemsToRepair", "No items to repair"),
    ("sNotesToggle", "Notes"),
    ("sOk", "Ok"),
    ("sOr", "OR"),
    ("sPCMenuHintA", "A)"),
    ("sPCMenuHintE", "E)"),
    ("sPCMenuHintF", "F)"),
    ("sPCMenuHintQ", "Q)"),
    ("sPCMenuHintR", "R)"),
    ("sPCMenuHintRMB", "RMB)"),
    ("sPCMenuHintS", "S)"),
    ("sPCMenuHintW", "W)"),
    ("sPCMenuHintX", "X)"),
    ("sPicksRemainingText", "Bobby Pins"),
    ("sPlaceMarker", "Place Marker"),
    ("sPlayAudio", "Play Audio"),
    ("sPlayCaravanText", "Play Caravan"),
    ("sPlural", "(s)"),
    ("sPMTime", "PM"),
    ("sPressStart", "Press START"),
    ("sPrevious", "Previous"),
    ("sQuestsTabText", "Quests"),
    ("sQuit", "Quit"),
    ("sRaiseText", "Raise"),
    ("sRanksText", "Ranks"),
    ("sRecipes", "RECIPES"),
    ("sRedeemCode", "Enter Code"),
    ("sRemaining", "remaining"),
    ("sRemoveText", "Remove"),
    ("sRepairAllItems", "Repair All (%d)"),
    ("sRepairItem", "Repair"),
    ("sRepairServicesTitle", "Repair Services"),
    ("sRepairSkill", "Repair Skill"),
    ("sRequirementsText", "Req"),
    ("sReset", "Reset"),
    ("sReturn", "Return"),
    ("sSave", "Save"),
    ("sSaveGame", "Save Game"),
    ("sSelect", "Select"),
    ("sSelectCardText", "Select Card"),
    ("sSettings", "Settings"),
    ("sShowActiveNotes", "Show Active Quest Notes"),
    ("sShowAllNotes", "Show All Notes"),
    ("sShowLocation", "Show Location"),
    ("sSkillRequirement", "Skill Requirement"),
    ("sSkillsCount", "Tag %d Skill"),
    ("sSkillsTitle", "SKILLS:  %d/%d Selected"),
    ("sSleep", "sleep"),
    ("sSpace", " "),
    ("sStats", "STATS"),
    ("sStatsAlcohol", "Alcohol"),
    ("sStatsAP", "AP"),
    ("sStatsBody", "Body"),
    ("sStatsCNDAbbrev", "CND"),
    ("sStatsCrippled", "CRIPPLED"),
    ("sStatsEFFAbbrev", "EFF"),
    ("sStatsFODAbbrev", "FOD"),
    ("sStatsGeneral", "General"),
    ("sStatsH20Abbrev", "H20"),
    ("sStatsHP", "HP"),
    ("sStatsLimbs", "Limbs"),
    ("sStatsLMBAbbrev", "LMB"),
    ("sStatsLVLAbbrev", "LVL"),
    ("sStatsNoEffects", "NO STATUS EFFECTS"),
    ("sStatsPerks", "Perks"),
    ("sStatsRADAbbrev", "RAD"),
    ("sStatsRadResist", "RAD RESIST"),
    ("sStatsSkills", "Skills"),
    ("sStatsSLPAbbrev", "SLP"),
    ("sStatsSpecial", "S.P.E.C.I.A.L."),
    ("sStatsStatus", "Status"),
    ("sStatsXP", "XP"),
    ("sStatsXPMax", "MAX"),
    ("sStimpak", "Stimpak"),
    ("sStopAudio", "Stop Audio"),
    ("sSwitchLimbs", "Switch Limbs"),
    ("sTakeAll", "Take All"),
    ("sTotalCardsText", "Total Cards: "),
    ("sTotalFundsText", "Total Funds: "),
    ("sTraitMenuCounter", "%d TRAITS LEFT"),
    ("sTraitMenuCounterSingular", "%d TRAIT LEFT"),
    ("sTraitMenuTitleText", "CHOOSE UP TO %d TRAITS"),
    ("sTraitMenuTitleTextSingular", "CHOOSE %d TRAIT"),
    ("sTravel", "Travel"),
    ("sTravelQuestion", "Do you want to travel to"),
    ("sTune", "Tune"),
    ("sVatsAiming", "Aiming"),
    ("sVatsBodyPart", "Body Part"),
    ("sVatsSelect", "Select"),
    ("sVatsTarget", "Target"),
    ("sVDSGManual", "VDSG MANUAL"),
    ("sVDSGPlate", "VDSG Plate"),
    ("sWait", "wait"),
    ("sWinLossText", "Win/Loss: "),
    ("sWorldMapTabText", "World Map"),
    ("sXPProgressMarker", "^"),
    ("sYes", "Yes"),
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
