# Flag 1 - ROM Reverse Engineering

Value: 4 points  
Solves: 0   :'(  

This flag is hidden inside the ROM.  

The intended ways to find it is either by looking closely at the ROM header or analysing the entrypoint of the ROM. In both case, you could see that the ROM 
is CGB and SGB compatible and actually does stuff according to the hardware it is running on. This can be detected by the value present in registers A and C at the beginning.  

From there, you simply have to run the ROM on another emulator(like BGB) in SGB (Super Game Boy) mode. Gameboy games running on a Super Game Boy (which is an adapter to run GB games on a SNES) 
have access to additionnal functionality, like uploading an graphical border to pad between the SNES and GB's output resolution.  

The ROM uses this functionality and puts the flag in the SGB border.