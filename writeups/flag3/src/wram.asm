STACK_SIZE = $B0

SECTION "WRAM", WRAM0
localVariables::
    DS $20         ; Reserve space for local variables inside functions
.end::
flagLengthRam::
    DB
flagRam::
    DS $10         ; Space where the flag is stored. Note that this is after the local variables so te buffer overflows it and the CTF players nmeeds to fetch it from SRAM
.end::
playerNameLengthRam::
    DB
playerNameRam::
    DS $8         ; Space where the name is stored
.end::
wStack::
	ds STACK_SIZE   ; Define a stack here. I make sure it's after "localVariables" so a buffer overflow can overwrite a function pointer here
wStackBottom::
isCgb::
    DB
isSgb::
    DB
; Used to tell the game in which state it is.
gameState::        
    DB
waitForFrame::
    DB
oldBankNumber::
    DB              ; Used to store bank number to restore it. Useful when needing to jump to ROM0 to access another bank
; Used to stored the joypad state
joypadDpad::
    DB
joypadButtons::
    DB
joypadDpadOld::
    DB
joypadButtonsOld::
    DB
; Used to store the serial state
serialState::
    DB
serialConnectionState::
    DB
serialReceiveData::
    DB
serialSendData::
    DB
serialReceivedNewData::
    DB
otherPlayerNameLength::
    DB
playerNameLengthCounter::
    db
flagExtractCounter::
    db
; From here forward, we can declare state-specific variables and they can overlap
copyingSGBTileDataState::
menuCursorPosition::
characterPositionX::
    DB
characterPositionY::
menuState::
    DB
animationCycleTimer::
    DB
characterDirection::
    DB
mapState::
    DB
menuInputLength::
    DB
npcCursorPosition::
    DB
menuInput::
textboxText::
    DS $24
.end::

; We put this in another section to make sure it's not too large to overflow the stack
SECTION "Text to display", WRAM0
textToDisplay::
    DS $80
.end

SECTION "Shadow", WRAM0
shadowScrollX::
    DB
shadowScrollY::
    DB
shadowWindow::
    DB

SECTION UNION "Shadow OAM", WRAM0, ALIGN[8]
shadowOAM::
    DS $A0
