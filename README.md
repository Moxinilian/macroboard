# macroboard

Utility to turn any keyboard into a macro key board on Linux (via libinput). Still in development, not very much tested yet.

## Features

- Create one-key or key-combination keyboard shortcuts  

- Support an unlimited amount of keyboards  

- Differentiate between all keyboards  

- Run closures on press and release of your shortcuts  

- Works on Linux! (both Wayland and X.Org[^1])  

- Option to keep the keyboard functionality of the macro keyboard while in use  

- No paid license, everything is free software!  

## Planned features before release

- Windows support

## Known issues / contribution opportunities

- Specified key codes map to keys as if the layout was QWERTY. If you use a different layout, you can adapt your key codes so that they map properly (a PR to fix this would be very welcome).

- The key combination detection algorithm is sufficient for a macro key board but is a bit unpolished. I might give it a bit more time, again PRs are welcome! :)

[^1]: Only Wayland has been tested in development so far.