# Flag 2 - Emulator Reverse Engineering
Value: 6 points  
Solves: 1  

This flag can be fetched from the server emulator by using a backdoor inside the emulator.  

The design philosophy of this specific flag was a needle in a haystack. While the backdoor is very lightly obfuscated, 
there is a lot of code in the executable to reverse until you find it, which reflects the reality of reverse-engineering full-fledged software.  

The backdoor can be found in the link cable implementation in the emulator. There is a state machine that checks all the received bytes and advance the state machine when the right byte is received. 
When the state machine gets to the last state.

To get the flag once the key is send, you simply have to connect to the victim and send the key over the socket. See `flag2.py` for exploit code.
