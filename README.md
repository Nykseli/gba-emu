# Game Boy Advance emulator

Game Boy Advance emulator written in Rust for educational porpuses

## State of the project

Since this project is only used for me to learn ARM instructions and emulation,
it's likely to never become a real functional emulator.

Currently it only supports running the "My first GBA demo" from [tonk](https://gbadev.net/tonc/first.html)
which looks like this:

```c
int main() {
    *(unsigned int*)0x04000000 = 0x0403;

    ((unsigned short*)0x06000000)[120+80*240] = 0x001F;
    ((unsigned short*)0x06000000)[136+80*240] = 0x03E0;
    ((unsigned short*)0x06000000)[120+96*240] = 0x7C00;

    while(1);

    return 0;
}
```

Other programs *might* run, but that's purely by accident.


## Debugger

For debugging programs, there's a very simple debugger that's inspired by `gdb`.
You can "enter" the debugger with `-d` cli argument, or you can run a debugger
script with `-d <scriptfile>`.


### Full list of debugger commends

```sh
# Comments start with '#'

# Use b or break to set break point on any address
b     08000188
break 08000188

# Or use rb or rbreak to set break point on addr relative to 0x08000000
rb     188
rbreak 188

# Print 32bit value in memory
v     03000000
value 03000000

# p/print prints current state of Cpu
p
print

# You can turn on/off logging
logon
logoff

# Stop executing the script
q
quit
exit

# run until next breakpoint, if any is found
r
run

# parse and execute current instruction and go to next instruction
n
next
```

## Acknowledgements

This project has been created using the following resources

* https://github.com/gbadev-org/awesome-gbadev
* https://gbadev.net/tonc/first.html
* https://problemkaputt.de/gbatek.htm
* http://bear.ces.cwru.edu/eecs_382/ARM7-TDMI-manual-pt3.pdf
