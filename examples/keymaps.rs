use rustile::{
    core::{Action, KeyBinding},
    mods,
};
use x11rb::NONE;
use xkeysym::Keysym;

pub fn keymaps() -> Vec<KeyBinding> {
    let mut keys = vec![
        KeyBinding {
            modifiers: mods::MOD_4,
            key: Keysym::Return,
            action: Action::Spawn("alacritty".into()),
        },
        KeyBinding {
            modifiers: mods::MOD_4,
            key: Keysym::b,
            action: Action::Spawn("brave-browser".into()),
        },
        KeyBinding {
            modifiers: mods::MOD_4,
            key: Keysym::space,
            action: Action::Spawn("rofi -show drun".into()),
        },
        KeyBinding {
            modifiers: mods::MOD_4,
            key: Keysym::q,
            action: Action::KillClient,
        },
        KeyBinding {
            modifiers: mods::MOD_4,
            key: Keysym::h,
            action: Action::MoveFocus(-1),
        },
        KeyBinding {
            modifiers: mods::MOD_4,
            key: Keysym::l,
            action: Action::MoveFocus(1),
        },
        KeyBinding {
            modifiers: mods::MOD_4 | mods::SHIFT,
            key: Keysym::h,
            action: Action::Swap(-1),
        },
        KeyBinding {
            modifiers: mods::MOD_4 | mods::SHIFT,
            key: Keysym::l,
            action: Action::Swap(1),
        },
        KeyBinding {
            modifiers: mods::MOD_4 | mods::CONTROL,
            key: Keysym::h,
            action: Action::ChangeRatio(0.05),
        },
        KeyBinding {
            modifiers: mods::MOD_4 | mods::CONTROL,
            key: Keysym::l,
            action: Action::ChangeRatio(-0.05),
        },
        KeyBinding {
            modifiers: mods::MOD_4,
            key: Keysym::r,
            action: Action::Restart,
        },
        KeyBinding {
            modifiers: mods::MOD_4,
            key: Keysym::t,
            action: Action::ToggleFloat,
        },
    ];

    let audio = vec![
        KeyBinding {
            modifiers: NONE as u16,
            key: Keysym::XF86_AudioRaiseVolume,
            action: Action::Spawn("volume up".into()),
        },
        KeyBinding {
            modifiers: NONE as u16,
            key: Keysym::XF86_AudioLowerVolume,
            action: Action::Spawn("volume down".into()),
        },
        KeyBinding {
            modifiers: NONE as u16,
            key: Keysym::XF86_AudioMute,
            action: Action::Spawn("volume mute".into()),
        },
    ];

    for a in audio {
        keys.push(a);
    }

    for i in 0..9 {
        let key_val = u32::from(Keysym::_1) + i;
        let k = xkeysym::Keysym::from(key_val);

        keys.push(KeyBinding {
            modifiers: mods::MOD_4,
            key: k,
            action: Action::GoToWorkspace(i as usize),
        });
        keys.push(KeyBinding {
            modifiers: mods::MOD_4 | mods::SHIFT,
            key: k,
            action: Action::MoveToWorkspace(i as usize),
        });
    }

    keys
}
