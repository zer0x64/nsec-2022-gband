include "hardware.inc"
include "sio.inc"

; ------------------------------
; RAM variables
; ------------------------------
def PAD equ _RAM ; Current state of the joypad
;def SIOTYPE equ _RAM+1 ; Running as MASTER or SLAVE?
;def TD equ _RAM+2
;def RD equ _RAM+3

; ------------------------------
; Main executable code
; ------------------------------
section "Main", rom0[$0100]
	nop
	jp main

	ds $150 - @, 0 ; Make room for the header

main:
	di
	ld sp, $ffff

	;ld hl, SIOTYPE

.wait_input:
	call read_joypad

	ld a, [PAD]	
	and PADF_START
	jr nz, .master

	ld a, [PAD]
	and PADF_SELECT
	jr nz, .slave

	call delay
	jr .wait_input

.master:
	;ld [hl], MASTER_MODE
	call init_sio_master

:
	call read_joypad
	ld a, [PAD]
	call sio_master_transfer

	ld a, 10
	call sio_master_transfer

	call delay
	jr :-

.slave:
	;ld [hl], SLAVE_MODE
	call init_sio_slave

:
	ld a, 69
	call sio_slave_transfer
	ld a, 10
	call sio_slave_transfer

	call delay
	jr :-


; ------------------------------
; Get pressed buttons on the joypad
;
; Stores the result in `PAD`
; ------------------------------
read_joypad:
	push bc

	ld a, P1F_GET_DPAD
	ld [rP1], a

	; Read the state of the dpad, with bouncing protection
	ld a, [rP1]
	ld a, [rP1]
	ld a, [rP1]
	ld a, [rP1]

	and $0F
	swap a
	ld b, a

	ld a, P1F_GET_BTN
	ld [rP1], a

	; Read the state of the buttons, with bouncing protection
	ld a, [rP1]
	ld a, [rP1]
	ld a, [rP1]
	ld a, [rP1]

	and $0F
	or b

	cpl
	ld [PAD], a

	pop bc

	ret

; ------------------------------
; Wait for 60000+15 cycles (~1.4ms)
; ------------------------------
delay:
	push de
	ld de, 6000
:
	dec de
	ld a, d
	or e
	jr z, :+
	nop
	jr :-
:
	pop de
	ret




;;;
; .loop:
; 	ld a, [SIOTYPE]
; 	or a
; 	jr nz, .start

; 	call read_joypad
; 	ld a, [PAD]
; 	and PADF_START
; 	jp z, .skip

; 	call init_sio_master

; .run:
; 	call read_joypad
; 	ld a, [PAD]
; 	ld [TD], a

; 	ld a, [SIOTYPE]
; 	cp MASTER_MODE
; 	call z, init_sio_master

; .skip:
; 	ld a, [RD]
; 	;???

; .done:
; 	call delay
; 	jr .loop

;; .slave:
;; 	jr .done

;; .master:
;; 	ld a, [PAD]
;; 	ld [rSB], a

;; 	ld a, SCF_START | SCF_SOURCE
;; 	ld [rSC], a
;; 	jr .done



;;;
; ld a, [PAD]
; ld [rSB], a

; Temp write a newline for log
; ld a, 10
; ld [rSB], a

; ld a, SCF_START | SCF_SOURCE
; ld [rSC], a

;; ld a, [PAD]
;; and PADF_SELECT
;; jr nz, .slave
;; ld a, [PAD]
;; and PADF_START
;; jr z, .master
