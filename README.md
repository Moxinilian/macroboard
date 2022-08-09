# macroboard

Utility to turn any keyboard into a macro key board on Linux (via libinput).

## Features

- Create one-key or key-combination keyboard shortcuts  

- Support an unlimited amount of keyboards  

- Differentiate between all keyboards  

- Run closures on press and release of your shortcuts  

- Works on Linux! (both Wayland and X.Org[^1])  

- Option to keep the keyboard functionality of the macro keyboard while in use  

- No paid license, everything is free software!  

[^1]: Only Wayland has been tested in development so far.

## Known issues / contribution opportunities

- Specified key codes map to keys as if the layout was QWERTY. If you use a different layout, you can adapt your key codes so that they map properly (a PR to fix this would be very welcome).

- The key combination detection algorithm is sufficient for a macro key board but is a bit unpolished. I might give it a bit more time, again PRs are welcome! :)

## About Windows support

The Windows API is utter trash, there is no good way to achieve the functionality of this project on Windows as open source software.

> If you need a macro key board on Windows using `macroboard`, set up a Linux machine on which you plug your keyboard. Then, network the Linux machine with the Windows machine to send events from Linux to Windows. I know, this is very sad.

### lmao why?

There are two main requirements for an operating system to support `macroboard` in a useful way:

- Provide a way to differentiate keyboard inputs from one keyboard or another. Good news, Windows has a low level API to do that: RawInput.
- Provide a way to block inputs coming from macro board keyboards. Good news-ish, Windows provides a very inefficient but sufficient (I guess) way to do this: global hooks.

Nice. Just one small problem: those are two different APIs that cannot co-operate in a reliable way. The only option left is to write a custom driver to implement Linux-like functionality in the Windows NT kernel, which is hard to do in a secure way and costs multiple hundreds of dollars to publish because of certification requirements. I am not doing that.

I used to have okay feelings about Windows as a platform before this experience. I would now like to quote Torvalds' famous words towards Nvidia and direct them at Microsoft on this one.
