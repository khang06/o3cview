# o3cview
Cross-platform rewrite of an old tool to view SayoDevice keypads' displays (O3C, O3C++, CM51, etc). **Currently, only 8000hz mode is supported.**

![Screenshot_20250203_004043](https://github.com/user-attachments/assets/13961c65-162e-43ed-9e29-8452e5d340f3)

## Usage
There's two versions: SDL and OBS. The SDL version shows the screen in a window, while the OBS version directly exposes it as a source in OBS.

### SDL
Just open it with your keypad connected. Press `.` to make the window bigger and `,` to make it smaller.

### OBS
Copy the plugin binary to your [plugins folder](https://obsproject.com/kb/plugins-guide) and a new "o3cview" source should be available in OBS.

**IMPORTANT: Creating multiple o3cview sources is currently not supported. You can get around this by copying and pasting the source instead.**
