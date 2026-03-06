use rustile::{
    core::{Action, KeyBinding},
    mods, rustile::Rustile,
};
use x11rb::{NONE, connection::Connection};
use xkeysym::Keysym;

pub fn keymaps<C: Connection>() -> Vec<KeyBinding<C>> {
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
            action: Action::Spawn("rofi -show drun -show-icons -theme launchpad".into()),
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
        KeyBinding {
            modifiers: mods::MOD_4,
            key: Keysym::m,
            action: Action::Custom(act),
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

    let win = vec![
        KeyBinding {
            modifiers: mods::MOD_4,
            key: Keysym::Right,
            action: Action::MoveFloating(20, 0),
        },
        KeyBinding {
            modifiers: mods::MOD_4,
            key: Keysym::Left,
            action: Action::MoveFloating(-20, 0),
        },
        KeyBinding {
            modifiers: mods::MOD_4,
            key: Keysym::Up,
            action: Action::MoveFloating(0, -20),
        },
        KeyBinding {
            modifiers: mods::MOD_4,
            key: Keysym::Down,
            action: Action::MoveFloating(0, 20),
        },
        KeyBinding {
            modifiers: mods::MOD_4 | mods::SHIFT,
            key: Keysym::Right,
            action: Action::ResizeFloating(20, 0),
        },
        KeyBinding {
            modifiers: mods::MOD_4 | mods::SHIFT,
            key: Keysym::Left,
            action: Action::ResizeFloating(-20, 0),
        },
        KeyBinding {
            modifiers: mods::MOD_4 | mods::SHIFT,
            key: Keysym::Up,
            action: Action::ResizeFloating(0, -20),
        },
        KeyBinding {
            modifiers: mods::MOD_4 | mods::SHIFT,
            key: Keysym::Down,
            action: Action::ResizeFloating(0, 20),
        },
    ];

    for a in audio {
        keys.push(a);
    }

    for w in win {
        keys.push(w);
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

fn act<C: Connection>(wm: &mut Rustile<C>){

}