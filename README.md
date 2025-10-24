
# Chip 8 Emulator

Rust implementation of a Chip8 interpreter using Egui for the debug version and SDL2 for release. As for now, only standard CHIP-8 is supported.


## Installation
#### Windows
On windows, both the debug and release versions are avaiable as .exe's.
However, it can also be build from source using cargo. (See below)

#### Linux / MacOS / Manual install
The emulator can be built from source using cargo. You can clone the repository and build either version yourself by carefully setting feature flags
####
For the debug version, simply build normally:
```bash
cargo build
```
For the release version, disable the default features and enable release-ver:
```bash
cargo build --release --no-default-features --features release-ver
```
    
## Usage
### Debug Version
From the bottom row, press 'Insert ROM' to choose a .ch8 file.
On the right side of the file path, there will be a 'Load ROM' button. If everything goes fine, a black screen-like rectangle will appear on the right hand side.
Here you can also:
- Choose the color of ON and OFF pixels;
- Change the frequency of the emulator.
#### Normal view
In the left panel you can use the buttons to interact with the emulator.
- 'Fetch' reads the next OPCODE from memory.
- 'Execute' (only avaiable after fetching first) will execute the fetched instruction;
- 'Step' will perform both one Fetch and one Execute;
- 'Run' will start an asyncronous execution;
- 'Stop' (avaiable after pressing 'Run') will pause the execution;
- 'Exit' will close the emulator.

The keypad grid shows the current status of each key, through the color of the text:
- Grey means the key is not pressed;
- Orange means the key is pressed;
The keypad follows the default structure (for now it's non-modifiable):
-  1 2 3 4
-  q w e r
-  a s d f
-  z x c v

The 'Debug mode' checkbox will switch to the Debug view.
#### Debug view
Here you can also check:
- OPCODE
- Each V register
- PC, SP and I registers
- DT and ST registers
- The content of the Stack
- A view of the memory (consider the bigger the interval, the slower the emulator will run)/

Updates are not done each frame, they have to be 'requested' using the 'Snaphot' button, however, the 'Continuous mode' checkbox can be marked, to ask the emulator to give its status each loop iteration (this will slow down the execution speed a bit though).

### Release Version
Drop any .ch8 file onto the executable and the emulator will start running that game.

Also here the keypad configuration is:
-  1 2 3 4
-  q w e r
-  a s d f
-  z x c v

You can exit by pressing 'Esc' or by hitting the 'X' on the taskbar.
