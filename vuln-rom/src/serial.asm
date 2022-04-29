INCLUDE "constants.inc"

SECTION FRAGMENT "Serial transfer", ROMX
RunSerialMode::
    ; Disable the PPU
    xor a
    ld [rLCDC], a

    ; We start without any scroll
    ld [shadowScrollX], a
    ld [shadowScrollY], a

    ; Clear Screen
    ld hl, _SCRN0
    ld bc, _SCRN1 - _SCRN0
    call MemSet

    ; Copy the tile map
    ld de, serialTileMap
    ld hl, _SCRN0
    ld bc, serialTileMap.end - serialTileMap
    call CopyToVRAM

    ;ld a, [isCgb]
    ;cp 1
    ;jr nz, .skipAttributeCopy

    ; GDMA the attribute map
    ; Change VRAM bank
    ;ld a, 1
    ;ld [rVBK], a

    ;ld de, menuAttributes
    ;ld hl, _SCRN0
    ;ld bc, menuAttributes.end - menuAttributes
    ;call CopyToVRAM

    ; Reset VRAM bank
    ;ld a, 0
    ;ld [rVBK], a

.skipAttributeCopy
    xor a
    ld [shadowOAM], a
    ld [shadowOAM + 1], a
    ld [shadowOAM + 2], a
    ld [shadowOAM + 3], a
    ; Cursor Y
    ;ld a, 16
    ;ld [shadowOAM], a

    ; Cursor X
    ;ld a, 8
    ;ld [shadowOAM + 1], a 
    
    ; Cursor tile index
    ;ld a, $91
    ;ld [shadowOAM + 2], a

    ; Cursor palette and attribute
    ;ld a, 0
    ;ld [shadowOAM + 3], a 

    ; Turn LDC on
    ld a, LCDC_DEFAULT
    ld [rLCDC], a
    ei

.connectionLoop
    ; We update the joypad state
    call ReadJoypad

    ; We handle the buttons
    ld a, [joypadButtons]
    ld b, a
    ld a, [joypadButtonsOld]

    call GetNewlyPushedButtons

    ; We only check for the a button
    bit 0, a

    ; If a is pressed, start with internal clock
    jr nz, .startWithInternalClock

    ; Else, wait for connection with external clock
    ld a, SERIAL_CONNECTION_STATE_INTERNAL ; Tell the other to connect as internal
    ldh [rSB], a
    xor a
    ld [serialReceiveData], a
    ld a, SCF_START
    ldh [rSC], a

    call WaitVblank

    ; Check if connection
    ld a, [serialConnectionState]
    cp SERIAL_CONNECTION_STATE_EXTERNAL
    jr z, .establishedConnection

    ld a, SERIAL_CONNECTION_STATE_UNCONNECTED
    ld [serialConnectionState], a

    jr .connectionLoop

.startWithInternalClock
    ld a, SERIAL_CONNECTION_STATE_INTERNAL
    ld [serialConnectionState], a

    ld a, SERIAL_CONNECTION_STATE_EXTERNAL ; Tell the other to connect as external
    ldh [rSB], a
    ld a, SCF_START | SCF_SOURCE
    ldh [rSC], a
    call WaitVblank

    ; Wait until the other player has connected
:
    ld a, [serialReceivedNewData]
    and a
    jr z, :-
    ld a, [serialReceiveData]
    and a
    jr nz, .startWithInternalClock

    ; We are good to go
.establishedConnection
    xor a
    call SerialSendByte
    call WaitVblank

    xor a
    call SerialSendByte

    call ExchangeName
    jr .done

.done
    halt
    jr .done

SerialSendByte:
    ld [serialSendData], a
    ld a, [serialConnectionState]
    cp SERIAL_CONNECTION_STATE_INTERNAL
    ret nz
    ld a, SCF_START | SCF_SOURCE
    ldh [rSC], a
    ret

ExchangeName:
    push bc
    push de
    push hl
    call ExchangeNameLength

    ; Get max length, put it in b
    ld a, [playerNameLengthRam]
    ld b, a
    ld a, [otherPlayerNameLength]
    cp b
    jr c, .startExchanging
    ld b, a

.startExchanging
    ld hl, playerNameRam
    ld de, localVariables

.loop
    ; Exchange one byte
    ld a, [hli]
    ld [serialSendData], a
    call SerialSendByte
    call WaitVblank

:
    ld a, [serialReceivedNewData]
    and a
    jr z, :-
    ld a, [serialReceiveData]
    ld c, a

    ; Wait
    call WaitVblank

    ; Store the byte into the local variables
    ld a, c
    ld [de], a
    inc de

    ; Decrease the length counter
    dec b
    jr nz, .loop

    pop hl
    pop de
    pop bc
    ret

ExchangeNameLength:
    ld a, [playerNameLengthRam]
    call SerialSendByte
    call WaitVblank

:
    ld a, [serialReceivedNewData]
    and a
    jr z, :-
    ld a, [serialReceiveData]
    ld [otherPlayerNameLength], a

    ret

WaitVblank:
    ; Lock so we wait for the frame to end
    ld a, 1
    ld [waitForFrame], a;
.waitForFrame
    ; Wait until waitForFrame = 0, which is set by the VBlank handler
    ld a, [waitForFrame]
    cp 0
    jr nz, .waitForFrame
    ret

SECTION FRAGMENT "Serial transfer", ROMX, ALIGN[8]
serialTileMap:
    db "Hello there, press A to continue"
.end
